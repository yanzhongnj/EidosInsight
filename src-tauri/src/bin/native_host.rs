//! EidosInsight Native Messaging Host
//!
//! 这是一个独立的可执行文件，由 Chrome 浏览器直接启动（不是 Tauri 主进程）。
//! 它通过标准输入/输出与 Chrome Extension 通信，通过 HTTP/IPC 与 Tauri 主进程通信。
//!
//! # Native Messaging 协议
//! Chrome 发送消息前先写 4 字节小端序消息长度，然后是 JSON payload。
//! 响应同理：4 字节小端序长度 + JSON。
//!
//! # 通信架构
//! Chrome Extension ←→ stdin/stdout ←→ NativeHost ←→ HTTP ←→ Tauri App (localhost:PORT)

use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

// ─────────────────────────────────────────────
// 协议类型
// ─────────────────────────────────────────────

/// 从 Chrome Extension 收到的消息
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum IncomingMessage {
    /// 心跳检测
    Ping {
        #[serde(default)]
        request_id: String,
    },
    /// 保存论文请求
    SavePaper {
        #[serde(default)]
        request_id: String,
        url: String,
        metadata: serde_json::Value,
    },
    /// 打开桌面应用窗口
    OpenApp {
        #[serde(default)]
        request_id: String,
    },
    /// 获取应用状态
    GetStatus {
        #[serde(default)]
        request_id: String,
    },
}

/// 发送给 Chrome Extension 的响应
#[derive(Debug, Serialize)]
struct OutgoingMessage {
    request_id: String,
    #[serde(rename = "type")]
    msg_type: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    paper_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl OutgoingMessage {
    fn ok(request_id: impl Into<String>, msg_type: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            msg_type: msg_type.into(),
            success: true,
            paper_id: None,
            metadata: None,
            error: None,
            message: None,
        }
    }

    fn err(request_id: impl Into<String>, msg_type: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            msg_type: msg_type.into(),
            success: false,
            paper_id: None,
            metadata: None,
            error: Some(error.into()),
            message: None,
        }
    }
}

// ─────────────────────────────────────────────
// Native Messaging I/O
// ─────────────────────────────────────────────

/// 从 stdin 读取一条 Native Messaging 格式消息
/// 格式：[4 字节小端序长度][JSON payload]
fn read_message() -> io::Result<Option<serde_json::Value>> {
    let mut len_buf = [0u8; 4];
    match io::stdin().read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let len = u32::from_le_bytes(len_buf) as usize;
    if len == 0 || len > 1024 * 1024 {
        // 消息为空或超过 1MB，拒绝
        return Ok(None);
    }

    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf)?;

    let value: serde_json::Value = serde_json::from_slice(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(Some(value))
}

/// 向 stdout 写入一条 Native Messaging 格式响应
fn write_message(msg: &OutgoingMessage) -> io::Result<()> {
    let json = serde_json::to_vec(msg)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let len = json.len() as u32;
    let mut stdout = io::stdout().lock();
    stdout.write_all(&len.to_le_bytes())?;
    stdout.write_all(&json)?;
    stdout.flush()?;
    Ok(())
}

// ─────────────────────────────────────────────
// Tauri App HTTP Bridge
// ─────────────────────────────────────────────

/// EidosInsight Tauri 主进程监听的 localhost 端口。
/// 主进程启动时会把端口号写到一个固定位置的文件，Host 读取它。
const DEFAULT_PORT: u16 = 47832;

fn get_app_port() -> u16 {
    // 尝试从 app data dir 读取端口号
    if let Some(data_dir) = dirs_port() {
        let port_file = data_dir.join("native_host_port");
        if let Ok(content) = std::fs::read_to_string(&port_file) {
            if let Ok(port) = content.trim().parse::<u16>() {
                return port;
            }
        }
    }
    DEFAULT_PORT
}

fn dirs_port() -> Option<std::path::PathBuf> {
    // Windows: %APPDATA%\com.eidosinsight.desktop\
    // macOS:   ~/Library/Application Support/com.eidosinsight.desktop/
    // Linux:   ~/.local/share/com.eidosinsight.desktop/
    dirs::data_dir().map(|d| d.join("com.eidosinsight.desktop"))
}

/// 向 Tauri 主进程发送 HTTP 请求（主进程暴露一个内部 HTTP 端点）
fn call_tauri(endpoint: &str, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
    let port = get_app_port();
    let url = format!("http://127.0.0.1:{}/native/{}", port, endpoint);

    // 使用阻塞 HTTP（native host 是同步的）
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp = client
        .post(&url)
        .json(payload)
        .send()
        .map_err(|e| {
            if e.is_connect() {
                "EidosInsight 桌面应用未启动或未响应".to_string()
            } else {
                format!("HTTP error: {}", e)
            }
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("App returned {}: {}", status, body));
    }

    resp.json::<serde_json::Value>()
        .map_err(|e| format!("JSON parse error: {}", e))
}

// ─────────────────────────────────────────────
// 消息处理
// ─────────────────────────────────────────────

fn handle_message(raw: serde_json::Value) -> OutgoingMessage {
    // 提前取 request_id，避免移动后无法访问
    let request_id = raw
        .get("request_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let msg: IncomingMessage = match serde_json::from_value(raw) {
        Ok(m) => m,
        Err(e) => {
            return OutgoingMessage::err(&request_id, "error", format!("Invalid message: {}", e));
        }
    };

    match msg {
        IncomingMessage::Ping { request_id } => {
            let mut resp = OutgoingMessage::ok(&request_id, "pong");
            resp.message = Some("pong".to_string());
            resp
        }

        IncomingMessage::GetStatus { request_id } => {
            // 尝试 ping Tauri 主进程
            let ping_payload = serde_json::json!({});
            match call_tauri("ping", &ping_payload) {
                Ok(_) => {
                    let mut resp = OutgoingMessage::ok(&request_id, "status");
                    resp.message = Some("running".to_string());
                    resp
                }
                Err(_) => OutgoingMessage::err(&request_id, "status", "app_not_running"),
            }
        }

        IncomingMessage::SavePaper { request_id, url, metadata } => {
            let payload = serde_json::json!({
                "url": url,
                "metadata": metadata,
            });

            match call_tauri("import_from_browser", &payload) {
                Ok(result) => {
                    let paper_id = result
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    let meta = result.get("metadata").cloned();
                    let mut resp = OutgoingMessage::ok(&request_id, "save_result");
                    resp.paper_id = paper_id;
                    resp.metadata = meta.or(Some(result));
                    resp
                }
                Err(e) => OutgoingMessage::err(&request_id, "save_result", e),
            }
        }

        IncomingMessage::OpenApp { request_id } => {
            let payload = serde_json::json!({});
            match call_tauri("focus_window", &payload) {
                Ok(_) => OutgoingMessage::ok(&request_id, "open_app_result"),
                Err(e) => OutgoingMessage::err(&request_id, "open_app_result", e),
            }
        }
    }
}

// ─────────────────────────────────────────────
// 主循环
// ─────────────────────────────────────────────

fn main() {
    // Native Host 会被 Chrome 以子进程启动，stdin/stdout 是通信通道
    // 所有日志必须写到 stderr（不能污染 stdout）
    eprintln!("[EidosInsight NativeHost] Started, PID={}", std::process::id());

    loop {
        match read_message() {
            Ok(Some(raw)) => {
                let response = handle_message(raw);
                if let Err(e) = write_message(&response) {
                    eprintln!("[EidosInsight NativeHost] Write error: {}", e);
                    break;
                }
            }
            Ok(None) => {
                // EOF — Chrome 关闭了连接
                eprintln!("[EidosInsight NativeHost] EOF, exiting");
                break;
            }
            Err(e) => {
                eprintln!("[EidosInsight NativeHost] Read error: {}", e);
                break;
            }
        }
    }
}
