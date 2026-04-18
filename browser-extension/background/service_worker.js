/**
 * EidosInsight Background Service Worker
 * 负责 Native Messaging 通信、消息路由、状态管理
 */

'use strict';

// ─────────────────────────────────────────────
// 常量
// ─────────────────────────────────────────────

const NATIVE_HOST_ID = 'com.eidosinsight.desktop';
const PING_INTERVAL_MS = 15000; // 15s 心跳检查连接
const CONNECT_TIMEOUT_MS = 5000;

// ─────────────────────────────────────────────
// 状态
// ─────────────────────────────────────────────

let nativePort = null;
let connectionStatus = 'disconnected'; // 'connected' | 'disconnected' | 'error'
let pendingRequests = new Map(); // requestId → { resolve, reject, timer }
let requestCounter = 0;
let pingTimer = null;

// ─────────────────────────────────────────────
// Native Messaging 连接管理
// ─────────────────────────────────────────────

function connect() {
  if (nativePort) return;

  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST_ID);
    connectionStatus = 'connected';

    nativePort.onMessage.addListener(onNativeMessage);

    nativePort.onDisconnect.addListener(() => {
      const err = chrome.runtime.lastError;
      console.warn('[EidosInsight] Native host disconnected:', err?.message || 'unknown');
      nativePort = null;
      connectionStatus = err ? 'error' : 'disconnected';
      stopPing();
      // 拒绝所有挂起请求
      for (const [id, pending] of pendingRequests) {
        clearTimeout(pending.timer);
        pending.reject(new Error('Native host disconnected'));
      }
      pendingRequests.clear();
      broadcastStatus();
    });

    startPing();
    broadcastStatus();
    console.log('[EidosInsight] Connected to native host');
  } catch (err) {
    console.error('[EidosInsight] Failed to connect native host:', err);
    connectionStatus = 'error';
    broadcastStatus();
  }
}

function disconnect() {
  stopPing();
  if (nativePort) {
    nativePort.disconnect();
    nativePort = null;
  }
  connectionStatus = 'disconnected';
}

// ─────────────────────────────────────────────
// 心跳
// ─────────────────────────────────────────────

function startPing() {
  stopPing();
  pingTimer = setInterval(async () => {
    try {
      await sendNativeRequest({ type: 'ping' });
    } catch (_) {
      // ping 失败时连接断开回调会处理
    }
  }, PING_INTERVAL_MS);
}

function stopPing() {
  if (pingTimer) {
    clearInterval(pingTimer);
    pingTimer = null;
  }
}

// ─────────────────────────────────────────────
// Native Messaging I/O
// ─────────────────────────────────────────────

function onNativeMessage(message) {
  const { request_id, type } = message;

  // 响应挂起请求
  if (request_id && pendingRequests.has(request_id)) {
    const pending = pendingRequests.get(request_id);
    clearTimeout(pending.timer);
    pendingRequests.delete(request_id);

    if (message.error) {
      pending.reject(new Error(message.error));
    } else {
      pending.resolve(message);
    }
    return;
  }

  // 主动推送消息（如桌面端发起通知）
  if (type === 'status_update') {
    broadcastStatus();
  }
}

function sendNativeRequest(payload) {
  return new Promise((resolve, reject) => {
    if (!nativePort) {
      connect();
      // 再次检查
      if (!nativePort) {
        reject(new Error('Not connected to EidosInsight desktop app'));
        return;
      }
    }

    const requestId = `req_${++requestCounter}_${Date.now()}`;
    const message = { ...payload, request_id: requestId };

    const timer = setTimeout(() => {
      pendingRequests.delete(requestId);
      reject(new Error('Request timeout'));
    }, CONNECT_TIMEOUT_MS);

    pendingRequests.set(requestId, { resolve, reject, timer });
    nativePort.postMessage(message);
  });
}

// ─────────────────────────────────────────────
// 状态广播（通知所有 popup）
// ─────────────────────────────────────────────

function broadcastStatus() {
  chrome.runtime.sendMessage({
    type: 'CONNECTION_STATUS',
    status: connectionStatus,
  }).catch(() => { /* popup 可能未开启 */ });
}

// ─────────────────────────────────────────────
// 核心操作：保存论文
// ─────────────────────────────────────────────

async function savePaper(tabId, metadata) {
  // 获取页面 metadata（如果外部没有传入）
  let finalMetadata = metadata;

  if (!finalMetadata) {
    try {
      const result = await chrome.tabs.sendMessage(tabId, { type: 'EXTRACT_METADATA' });
      if (!result.success) throw new Error(result.error || 'Metadata extraction failed');
      finalMetadata = result.metadata;
    } catch (err) {
      throw new Error(`Cannot extract page metadata: ${err.message}`);
    }
  }

  // 确保连接
  if (!nativePort) connect();

  // 发送到 Native Host
  const response = await sendNativeRequest({
    type: 'save_paper',
    url: finalMetadata.url,
    metadata: finalMetadata,
  });

  return response;
}

// ─────────────────────────────────────────────
// 消息路由（来自 popup 或 content script）
// ─────────────────────────────────────────────

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  const tabId = sender.tab?.id;

  switch (message.type) {

    case 'GET_STATUS':
      // popup 查询连接状态
      sendResponse({ status: connectionStatus });
      // 同时尝试（重新）连接
      if (connectionStatus !== 'connected') {
        connect();
        // 100ms 后反馈新状态
        setTimeout(() => {
          chrome.runtime.sendMessage({
            type: 'CONNECTION_STATUS',
            status: connectionStatus,
          }).catch(() => {});
        }, 100);
      }
      return true;

    case 'SAVE_PAPER':
      // popup 请求保存
      savePaper(message.tabId, message.metadata)
        .then(result => sendResponse({ success: true, result }))
        .catch(err => sendResponse({ success: false, error: err.message }));
      return true; // async

    case 'CHECK_PAPER_PAGE':
      // popup 查询当前 tab 是否是论文页
      if (tabId) {
        chrome.tabs.sendMessage(tabId, { type: 'CHECK_PAPER_PAGE' })
          .then(result => sendResponse(result))
          .catch(() => sendResponse({ isPaper: false }));
      } else if (message.tabId) {
        chrome.tabs.sendMessage(message.tabId, { type: 'CHECK_PAPER_PAGE' })
          .then(result => sendResponse(result))
          .catch(() => sendResponse({ isPaper: false }));
      } else {
        sendResponse({ isPaper: false });
      }
      return true;

    case 'EXTRACT_METADATA_FROM_TAB':
      // popup 请求从指定 tab 提取
      chrome.tabs.sendMessage(message.tabId, { type: 'EXTRACT_METADATA' })
        .then(result => sendResponse(result))
        .catch(err => sendResponse({ success: false, error: err.message }));
      return true;

    default:
      break;
  }
});

// ─────────────────────────────────────────────
// 扩展安装 / 启动
// ─────────────────────────────────────────────

chrome.runtime.onInstalled.addListener(() => {
  console.log('[EidosInsight] Extension installed');
  connect();
});

chrome.runtime.onStartup.addListener(() => {
  connect();
});

// 初始连接
connect();
