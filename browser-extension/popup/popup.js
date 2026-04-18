/**
 * EidosInsight Popup Logic
 */

'use strict';

// ─────────────────────────────────────────────
// DOM refs
// ─────────────────────────────────────────────

const $ = id => document.getElementById(id);

const statusDot    = $('statusDot');
const statusText   = $('statusText');
const notPaperNotice = $('notPaperNotice');
const metaCard     = $('metaCard');
const actionsArea  = $('actionsArea');
const successCard  = $('successCard');
const errorCard    = $('errorCard');
const disconnCard  = $('disconnCard');
const loadingOverlay = $('loadingOverlay');

const sourceBadge  = $('sourceBadge');
const fieldTitle   = $('fieldTitle');
const fieldAuthors = $('fieldAuthors');
const fieldDate    = $('fieldDate');
const fieldDoi     = $('fieldDoi');
const fieldAbstract = $('fieldAbstract');
const abstractField = $('abstractField');
const toggleAbstract = $('toggleAbstract');

const btnSave       = $('btnSave');
const btnSaveText   = $('btnSaveText');
const btnRefresh    = $('btnRefresh');
const btnRetry      = $('btnRetry');
const btnOpenApp    = $('btnOpenApp');
const btnInstallHost = $('btnInstallHost');
const successHint   = $('successHint');
const errorMsg      = $('errorMsg');
const loadingText   = $('loadingText');

// ─────────────────────────────────────────────
// State
// ─────────────────────────────────────────────

let currentMetadata = null;
let currentTab = null;
let isConnected = false;

// ─────────────────────────────────────────────
// UI helpers
// ─────────────────────────────────────────────

function showOnly(...toShow) {
  const all = [notPaperNotice, metaCard, actionsArea, successCard, errorCard, disconnCard];
  all.forEach(el => el.style.display = 'none');
  toShow.forEach(el => { if (el) el.style.display = ''; });
}

function showLoading(text = '正在保存...') {
  loadingText.textContent = text;
  loadingOverlay.style.display = 'flex';
}

function hideLoading() {
  loadingOverlay.style.display = 'none';
}

function setStatus(status) {
  statusDot.className = 'status-dot';
  if (status === 'connected') {
    statusDot.classList.add('connected');
    statusText.textContent = '已连接';
    isConnected = true;
  } else if (status === 'error') {
    statusDot.classList.add('error');
    statusText.textContent = '连接错误';
    isConnected = false;
  } else if (status === 'checking') {
    statusDot.classList.add('checking');
    statusText.textContent = '检查中...';
    isConnected = false;
  } else {
    statusDot.classList.add('disconnected');
    statusText.textContent = '未连接';
    isConnected = false;
  }
}

function renderMetadata(meta) {
  currentMetadata = meta;

  // Source badge
  const sourceLabels = {
    arxiv: 'arXiv', acm: 'ACM DL', ieee: 'IEEE Xplore',
    semanticscholar: 'Semantic Scholar', openreview: 'OpenReview',
    neurips: 'NeurIPS', aclanthology: 'ACL', pubmed: 'PubMed',
    springer: 'Springer', sciencedirect: 'Elsevier', generic: '学术页面',
  };
  sourceBadge.textContent = sourceLabels[meta.source] || meta.source || '未知来源';

  fieldTitle.textContent = meta.title || '（未能提取标题）';
  fieldAuthors.textContent = meta.authors || '（未知作者）';
  fieldDate.textContent = meta.published_date || '–';
  fieldDoi.textContent = meta.doi || '–';

  if (meta.abstract) {
    fieldAbstract.textContent = meta.abstract;
    abstractField.style.display = '';
  } else {
    abstractField.style.display = 'none';
  }
}

// ─────────────────────────────────────────────
// Core flow
// ─────────────────────────────────────────────

async function init() {
  setStatus('checking');

  // 获取当前 tab
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  currentTab = tab;

  if (!tab) {
    showOnly(notPaperNotice);
    return;
  }

  // 查询连接状态
  chrome.runtime.sendMessage({ type: 'GET_STATUS' }, response => {
    setStatus(response?.status || 'disconnected');
  });

  // 检测是否论文页面
  showLoading('正在分析页面...');

  let isPaper = false;
  try {
    const result = await chrome.tabs.sendMessage(tab.id, { type: 'CHECK_PAPER_PAGE' });
    isPaper = result?.isPaper || false;
  } catch (_) {
    // content script 未注入（非匹配页面）
    isPaper = false;
  }

  if (!isPaper) {
    hideLoading();
    showOnly(notPaperNotice);
    return;
  }

  // 提取元数据
  await extractAndRender();
}

async function extractAndRender() {
  showLoading('正在提取元数据...');
  try {
    const result = await chrome.tabs.sendMessage(currentTab.id, { type: 'EXTRACT_METADATA' });
    hideLoading();

    if (!result?.success || !result.metadata) {
      showOnly(notPaperNotice);
      return;
    }

    renderMetadata(result.metadata);

    if (isConnected) {
      showOnly(metaCard, actionsArea);
    } else {
      showOnly(metaCard, disconnCard);
    }
  } catch (err) {
    hideLoading();
    showOnly(notPaperNotice);
  }
}

async function doSave() {
  if (!currentMetadata) return;

  // 读取用户可能编辑过的字段
  const meta = {
    ...currentMetadata,
    title: fieldTitle.textContent.trim() || currentMetadata.title,
    authors: fieldAuthors.textContent.trim() || currentMetadata.authors,
  };

  btnSave.disabled = true;
  btnSaveText.textContent = '保存中...';
  showLoading('正在保存到 EidosInsight...');

  chrome.runtime.sendMessage(
    { type: 'SAVE_PAPER', tabId: currentTab.id, metadata: meta },
    (response) => {
      hideLoading();
      btnSave.disabled = false;
      btnSaveText.textContent = '保存到 EidosInsight';

      if (response?.success) {
        successHint.textContent = meta.title
          ? `"${meta.title.slice(0, 40)}${meta.title.length > 40 ? '...' : ''}" 已保存`
          : '文献已保存到您的知识库';
        showOnly(successCard);
      } else {
        errorMsg.textContent = response?.error || '未知错误，请重试';
        showOnly(errorCard);
      }
    }
  );
}

// ─────────────────────────────────────────────
// 事件绑定
// ─────────────────────────────────────────────

btnSave.addEventListener('click', doSave);

btnRefresh.addEventListener('click', async () => {
  await extractAndRender();
});

btnRetry.addEventListener('click', async () => {
  // 重新检查连接并尝试
  setStatus('checking');
  chrome.runtime.sendMessage({ type: 'GET_STATUS' }, response => {
    setStatus(response?.status || 'disconnected');
    if (response?.status === 'connected') {
      doSave();
    } else {
      showOnly(disconnCard);
    }
  });
});

btnOpenApp.addEventListener('click', () => {
  // 通知桌面端打开 app 窗口（桌面端收到后调用 focus/show）
  chrome.runtime.sendMessage({ type: 'OPEN_APP' });
  window.close();
});

btnInstallHost.addEventListener('click', () => {
  // 打开安装说明页面
  chrome.tabs.create({
    url: 'https://github.com/eidosinsight/eidosinsight#native-host-installation',
  });
});

toggleAbstract.addEventListener('click', () => {
  const expanded = fieldAbstract.classList.toggle('expanded');
  toggleAbstract.textContent = expanded ? '收起' : '展开';
});

// 监听来自 background 的状态更新
chrome.runtime.onMessage.addListener((message) => {
  if (message.type === 'CONNECTION_STATUS') {
    setStatus(message.status);
    // 如果刚连上，且当前显示的是断连卡片，切换到操作区
    if (message.status === 'connected' && currentMetadata) {
      showOnly(metaCard, actionsArea);
    } else if (message.status !== 'connected' && currentMetadata) {
      showOnly(metaCard, disconnCard);
    }
  }
});

// ─────────────────────────────────────────────
// 启动
// ─────────────────────────────────────────────

init().catch(err => {
  console.error('[EidosInsight Popup] Init error:', err);
  hideLoading();
  showOnly(notPaperNotice);
});
