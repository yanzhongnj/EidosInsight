//! Browser extension command handlers
//!
//! 处理来自 Chrome Extension (通过 Native Messaging Host) 的请求

use crate::db::{DbState, PaperRepo};
use crate::errors::{AppError, Result};
use crate::models::{CreatePaperInput, Paper};
use crate::services::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ─────────────────────────────────────────────
// 协议类型
// ─────────────────────────────────────────────

/// 浏览器端发送过来的元数据（由 content_script.js 提取）
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BrowserMetadata {
    pub title: Option<String>,
    pub authors: Option<String>,
    pub abstract_text: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_alt: Option<String>,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub pdf_url: Option<String>,
    pub arxiv_id: Option<String>,
    pub published_date: Option<String>,
    pub journal: Option<String>,
    pub subjects: Option<String>,
    pub source: Option<String>,
    pub page_title: Option<String>,
}

impl BrowserMetadata {
    /// 获取摘要（兼容两个字段名）
    pub fn get_abstract(&self) -> Option<&str> {
        self.abstract_text
            .as_deref()
            .or(self.abstract_alt.as_deref())
    }
}

/// import_from_browser 命令的参数
#[derive(Debug, Deserialize)]
pub struct ImportFromBrowserInput {
    pub url: String,
    pub metadata: BrowserMetadata,
}

// ─────────────────────────────────────────────
// Tauri Commands
// ─────────────────────────────────────────────

/// 从浏览器扩展接收并保存论文元数据
///
/// 由 Native Messaging Host 转发的 Chrome Extension 请求触发。
/// 也可由 Tauri 前端直接调用（用于 URL 导入功能）。
#[tauri::command]
pub fn import_from_browser(
    input: ImportFromBrowserInput,
    db: State<'_, DbState>,
    _app_state: State<'_, AppState>,
) -> Result<Paper> {
    let meta = &input.metadata;

    // 标题必须有，否则拒绝
    let title = meta.title.clone().unwrap_or_else(|| {
        meta.page_title
            .clone()
            .unwrap_or_else(|| "Untitled Paper".to_string())
    });

    if title.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Cannot import paper without a title".to_string(),
        ));
    }

    // 构建 CreatePaperInput（字段名与 paper.rs 中保持一致）
    let create_input = CreatePaperInput {
        title,
        authors: meta.authors.clone(),
        abstract_text: meta.get_abstract().map(str::to_string),
        doi: meta.doi.clone(),
        arxiv_id: meta.arxiv_id.clone(),
        pdf_url: meta.pdf_url.clone(),
        pdf_path: None,   // 浏览器端只有 URL，无本地 PDF 路径
        publication_date: meta.published_date.clone(),
        journal: meta.journal.clone(),
        volume: None,
        issue: None,
        pages: None,
        file_type: None,
        folder_id: None,
    };

    let db = db.0.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let paper = PaperRepo::insert(db.conn(), &create_input)?;

    tracing::info!(
        paper_id = %paper.id,
        title = %paper.title,
        source = %meta.source.as_deref().unwrap_or("unknown"),
        "Paper imported from browser extension"
    );

    Ok(paper)
}

/// 获取浏览器扩展连接状态（供前端展示安装引导）
#[tauri::command]
pub fn get_extension_status() -> ExtensionStatus {
    // 检查 native host manifest 是否已注册
    let registered = check_native_host_registered();
    ExtensionStatus { registered }
}

#[derive(Debug, Serialize)]
pub struct ExtensionStatus {
    pub registered: bool,
}

/// 注册 Native Messaging Host（写入 Chrome 注册表 / manifest 文件）
#[tauri::command]
pub fn register_native_host(app: tauri::AppHandle) -> Result<()> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::Internal(format!("Cannot get exe path: {}", e)))?;

    // 替换主进程可执行文件名为 native host 二进制名
    let host_exe = exe_path
        .parent()
        .ok_or_else(|| AppError::Internal("Cannot get exe dir".to_string()))?
        .join(if cfg!(windows) {
            "eidosinsight-native-host.exe"
        } else {
            "eidosinsight-native-host"
        });

    let manifest = serde_json::json!({
        "name": "com.eidosinsight.desktop",
        "description": "EidosInsight Native Messaging Host",
        "path": host_exe.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": [
            // 正式发布后替换为真实扩展 ID
            "chrome-extension://*/",
        ]
    });

    do_register_native_host(&manifest, &app)?;

    tracing::info!("Native Messaging Host registered, exe: {:?}", host_exe);
    Ok(())
}

// ─────────────────────────────────────────────
// 平台注册实现
// ─────────────────────────────────────────────

fn check_native_host_registered() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 检查注册表项是否存在
        use std::process::Command;
        let output = Command::new("reg")
            .args(["query", r"HKCU\Software\Google\Chrome\NativeMessagingHosts\com.eidosinsight.desktop"])
            .output();
        matches!(output, Ok(o) if o.status.success())
    }

    #[cfg(target_os = "macos")]
    {
        let path = dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Google/Chrome/NativeMessagingHosts/com.eidosinsight.desktop.json"));
        path.map(|p| p.exists()).unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    {
        let path = dirs::home_dir()
            .map(|h| h.join(".config/google-chrome/NativeMessagingHosts/com.eidosinsight.desktop.json"));
        path.map(|p| p.exists()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    { false }
}

fn do_register_native_host(manifest: &serde_json::Value, _app: &tauri::AppHandle) -> Result<()> {
    let manifest_str = serde_json::to_string_pretty(manifest)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    #[cfg(target_os = "windows")]
    {
        // 1. 写 manifest JSON 到 AppData
        let manifest_path = dirs::data_local_dir()
            .ok_or_else(|| AppError::Internal("Cannot find AppData/Local".to_string()))?
            .join("EidosInsight")
            .join("native_host_manifest.json");

        std::fs::create_dir_all(manifest_path.parent().unwrap())
            .map_err(|e| AppError::Io(e))?;
        std::fs::write(&manifest_path, &manifest_str)
            .map_err(|e| AppError::Io(e))?;

        // 2. 写注册表
        let reg_key = r"HKCU\Software\Google\Chrome\NativeMessagingHosts\com.eidosinsight.desktop";
        let reg_value = manifest_path.to_string_lossy().to_string();

        let status = std::process::Command::new("reg")
            .args(["add", reg_key, "/ve", "/t", "REG_SZ", "/d", &reg_value, "/f"])
            .status()
            .map_err(|e| AppError::Internal(format!("reg add failed: {}", e)))?;

        if !status.success() {
            return Err(AppError::Internal("Failed to write registry".to_string()));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let dir = dirs::home_dir()
            .ok_or_else(|| AppError::Internal("Cannot find home dir".to_string()))?
            .join("Library/Application Support/Google/Chrome/NativeMessagingHosts");
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(e))?;
        std::fs::write(dir.join("com.eidosinsight.desktop.json"), &manifest_str)
            .map_err(|e| AppError::Io(e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let dir = dirs::home_dir()
            .ok_or_else(|| AppError::Internal("Cannot find home dir".to_string()))?
            .join(".config/google-chrome/NativeMessagingHosts");
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(e))?;
        std::fs::write(dir.join("com.eidosinsight.desktop.json"), &manifest_str)
            .map_err(|e| AppError::Io(e))?;
    }

    Ok(())
}
