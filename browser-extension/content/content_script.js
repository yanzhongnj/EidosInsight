/**
 * EidosInsight Content Script
 * 负责从当前页面提取论文元数据，响应来自 popup/background 的消息
 */

'use strict';

// ─────────────────────────────────────────────
// 工具函数
// ─────────────────────────────────────────────

function trimText(el) {
  return el ? el.textContent.trim().replace(/\s+/g, ' ') : null;
}

function metaContent(name) {
  const el =
    document.querySelector(`meta[name="${name}"]`) ||
    document.querySelector(`meta[property="${name}"]`) ||
    document.querySelector(`meta[name="${name.toLowerCase()}"]`);
  return el ? el.getAttribute('content')?.trim() || null : null;
}

function allMetaContent(name) {
  const els = document.querySelectorAll(`meta[name="${name}"], meta[property="${name}"]`);
  return Array.from(els).map(el => el.getAttribute('content')?.trim()).filter(Boolean);
}

// ─────────────────────────────────────────────
// 站点专属提取器
// ─────────────────────────────────────────────

const extractors = {

  // ── arXiv ──────────────────────────────────
  arxiv() {
    const meta = {};
    const url = window.location.href;

    // 解析 arxiv ID
    const absMatch = url.match(/arxiv\.org\/abs\/([0-9]{4}\.[0-9]{4,5}(?:v\d+)?)/);
    const pdfMatch = url.match(/arxiv\.org\/pdf\/([0-9]{4}\.[0-9]{4,5}(?:v\d+)?)/);
    const arxivId = (absMatch || pdfMatch)?.[1];

    if (arxivId) {
      meta.arxiv_id = arxivId;
      meta.pdf_url = `https://arxiv.org/pdf/${arxivId}.pdf`;
      meta.abstract_url = `https://arxiv.org/abs/${arxivId}`;
    }

    // 标题
    meta.title =
      trimText(document.querySelector('h1.title')) ||
      trimText(document.querySelector('#abs h1')) ||
      metaContent('og:title') ||
      trimText(document.querySelector('title'));
    if (meta.title?.startsWith('Title:')) {
      meta.title = meta.title.replace(/^Title:\s*/, '');
    }

    // 作者
    const authorsEl = document.querySelector('.authors');
    if (authorsEl) {
      meta.authors = Array.from(authorsEl.querySelectorAll('a'))
        .map(a => a.textContent.trim())
        .filter(Boolean)
        .join(', ');
    }
    if (!meta.authors) {
      meta.authors = metaContent('citation_author');
    }

    // 摘要
    meta.abstract =
      trimText(document.querySelector('blockquote.abstract')) ||
      trimText(document.querySelector('#abs blockquote'));
    if (meta.abstract?.startsWith('Abstract:')) {
      meta.abstract = meta.abstract.replace(/^Abstract:\s*/, '');
    }

    // 日期
    meta.published_date =
      trimText(document.querySelector('.dateline'))?.match(/Submitted[^;]*;\s*(.*)/)?.[1] ||
      metaContent('citation_date') ||
      metaContent('citation_online_date');

    // 分类
    const subjectEl = document.querySelector('.primary-subject');
    if (subjectEl) meta.subjects = trimText(subjectEl);

    meta.source = 'arxiv';
    return meta;
  },

  // ── ACM Digital Library ────────────────────
  acm() {
    const meta = {};
    meta.title = metaContent('og:title') || trimText(document.querySelector('h1.citation__title'));
    meta.authors = allMetaContent('citation_author').join(', ') ||
      Array.from(document.querySelectorAll('.author-name span')).map(trimText).join(', ');
    meta.abstract = metaContent('og:description') || trimText(document.querySelector('.abstractSection p'));
    meta.doi = metaContent('citation_doi') ||
      document.querySelector('.issue-item__doi a')?.textContent?.trim();
    meta.published_date = metaContent('citation_publication_date') || metaContent('citation_online_date');
    meta.journal = metaContent('citation_journal_title') || metaContent('citation_conference_title');
    meta.source = 'acm';
    return meta;
  },

  // ── IEEE Xplore ────────────────────────────
  ieee() {
    const meta = {};
    meta.title = metaContent('og:title') || trimText(document.querySelector('h1.document-title'));
    meta.authors = Array.from(document.querySelectorAll('.authors-info .text-base'))
      .map(trimText).filter(Boolean).join(', ') ||
      allMetaContent('citation_author').join(', ');
    meta.abstract = metaContent('og:description') ||
      trimText(document.querySelector('.abstract-text p'));
    meta.doi = metaContent('citation_doi');
    meta.published_date = metaContent('citation_publication_date') || metaContent('citation_online_date');
    meta.journal = metaContent('citation_journal_title') || metaContent('citation_conference_title');
    meta.source = 'ieee';
    return meta;
  },

  // ── Semantic Scholar ───────────────────────
  semanticscholar() {
    const meta = {};
    meta.title = metaContent('og:title') || trimText(document.querySelector('h1[data-test-id="paper-detail-title"]'));
    meta.authors = Array.from(document.querySelectorAll('[data-test-id="author-list"] .author-list__link'))
      .map(trimText).filter(Boolean).join(', ');
    meta.abstract = trimText(document.querySelector('[data-test-id="paper-abstract"]')) ||
      metaContent('og:description');
    meta.doi = trimText(document.querySelector('[data-test-id="doi-link"]'));
    meta.source = 'semanticscholar';
    return meta;
  },

  // ── OpenReview ─────────────────────────────
  openreview() {
    const meta = {};
    meta.title = metaContent('og:title') || trimText(document.querySelector('h2.card-title'));
    meta.authors = allMetaContent('citation_author').join(', ');
    meta.abstract = metaContent('og:description');
    meta.source = 'openreview';
    return meta;
  },

  // ── NeurIPS Proceedings ────────────────────
  neurips() {
    const meta = {};
    meta.title = trimText(document.querySelector('h4')) || metaContent('og:title');
    meta.authors = Array.from(document.querySelectorAll('p.author i'))
      .map(trimText).filter(Boolean).join(', ');
    meta.abstract = trimText(document.querySelector('p.abstract'));
    meta.source = 'neurips';
    return meta;
  },

  // ── ACL Anthology ──────────────────────────
  aclanthology() {
    const meta = {};
    meta.title = trimText(document.querySelector('h2#title')) || metaContent('og:title');
    meta.authors = Array.from(document.querySelectorAll('.lead a'))
      .map(trimText).filter(Boolean).join(', ') ||
      allMetaContent('citation_author').join(', ');
    meta.abstract = trimText(document.querySelector('div.card-body p'));
    meta.doi = metaContent('citation_doi');
    meta.source = 'aclanthology';
    return meta;
  },

  // ── PubMed ────────────────────────────────
  pubmed() {
    const meta = {};
    meta.title = trimText(document.querySelector('h1.heading-title')) || metaContent('og:title');
    meta.authors = Array.from(document.querySelectorAll('.authors-list .full-name'))
      .map(trimText).filter(Boolean).join(', ');
    meta.abstract = trimText(document.querySelector('#abstract p')) ||
      trimText(document.querySelector('.abstract-content'));
    meta.doi = document.querySelector('.identifier.doi')?.textContent?.replace('doi:', '')?.trim();
    meta.source = 'pubmed';
    return meta;
  },

  // ── Springer Link ─────────────────────────
  springer() {
    const meta = {};
    meta.title = metaContent('citation_title') || trimText(document.querySelector('h1.c-article-title'));
    meta.authors = allMetaContent('citation_author').join(', ');
    meta.abstract = trimText(document.querySelector('#Abs1-content p')) ||
      metaContent('og:description');
    meta.doi = metaContent('citation_doi');
    meta.published_date = metaContent('citation_publication_date') || metaContent('citation_online_date');
    meta.journal = metaContent('citation_journal_title');
    meta.source = 'springer';
    return meta;
  },

  // ── ScienceDirect / Elsevier ──────────────
  sciencedirect() {
    const meta = {};
    meta.title = metaContent('citation_title') || trimText(document.querySelector('span.title-text'));
    meta.authors = allMetaContent('citation_author').join(', ');
    meta.abstract = trimText(document.querySelector('.abstract p')) || metaContent('og:description');
    meta.doi = metaContent('citation_doi');
    meta.published_date = metaContent('citation_publication_date') || metaContent('citation_online_date');
    meta.journal = metaContent('citation_journal_title');
    meta.source = 'sciencedirect';
    return meta;
  },

  // ── 通用 fallback ──────────────────────────
  generic() {
    const meta = {};
    // 优先使用 citation_* 系列 meta（大多数学术网站遵循此标准）
    meta.title =
      metaContent('citation_title') ||
      metaContent('og:title') ||
      metaContent('dc.title') ||
      trimText(document.querySelector('h1')) ||
      document.title;

    const authorsMeta = allMetaContent('citation_author');
    meta.authors = authorsMeta.length > 0
      ? authorsMeta.join(', ')
      : metaContent('author') || metaContent('dc.creator');

    meta.abstract =
      metaContent('citation_abstract') ||
      metaContent('og:description') ||
      metaContent('dc.description') ||
      metaContent('description');

    meta.doi = metaContent('citation_doi') || metaContent('dc.identifier');
    meta.published_date =
      metaContent('citation_publication_date') ||
      metaContent('citation_online_date') ||
      metaContent('dc.date');
    meta.journal =
      metaContent('citation_journal_title') ||
      metaContent('citation_conference_title');

    // 尝试从 PDF 链接中获取
    const pdfLink = document.querySelector('a[href$=".pdf"]');
    if (pdfLink) meta.pdf_url = pdfLink.href;

    meta.source = 'generic';
    return meta;
  },
};

// ─────────────────────────────────────────────
// 站点路由
// ─────────────────────────────────────────────

function detectExtractor() {
  const host = window.location.hostname;
  if (host.includes('arxiv.org')) return extractors.arxiv;
  if (host.includes('dl.acm.org')) return extractors.acm;
  if (host.includes('ieeexplore.ieee.org')) return extractors.ieee;
  if (host.includes('semanticscholar.org')) return extractors.semanticscholar;
  if (host.includes('openreview.net')) return extractors.openreview;
  if (host.includes('papers.nips.cc') || host.includes('proceedings.neurips.cc')) return extractors.neurips;
  if (host.includes('aclanthology.org')) return extractors.aclanthology;
  if (host.includes('pubmed.ncbi.nlm.nih.gov')) return extractors.pubmed;
  if (host.includes('link.springer.com')) return extractors.springer;
  if (host.includes('sciencedirect.com')) return extractors.sciencedirect;
  return extractors.generic;
}

function extractMetadata() {
  const extractor = detectExtractor();
  const meta = extractor();

  // 补充通用字段
  meta.url = window.location.href;
  meta.page_title = document.title;
  meta.extracted_at = new Date().toISOString();

  // 清理空字段
  return Object.fromEntries(
    Object.entries(meta).filter(([, v]) => v != null && v !== '')
  );
}

// ─────────────────────────────────────────────
// 消息监听
// ─────────────────────────────────────────────

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === 'EXTRACT_METADATA') {
    try {
      const metadata = extractMetadata();
      sendResponse({ success: true, metadata });
    } catch (err) {
      sendResponse({ success: false, error: err.message });
    }
    return true; // keep channel open for async
  }

  if (message.type === 'CHECK_PAPER_PAGE') {
    const isPaper = isPaperPage();
    sendResponse({ isPaper });
    return true;
  }
});

// 判断当前页面是否为论文页面
function isPaperPage() {
  const host = window.location.hostname;
  const path = window.location.pathname;

  // 已知学术域名直接返回 true
  const academicDomains = [
    'arxiv.org', 'dl.acm.org', 'ieeexplore.ieee.org',
    'semanticscholar.org', 'openreview.net', 'papers.nips.cc',
    'aclanthology.org', 'pubmed.ncbi.nlm.nih.gov',
    'link.springer.com', 'sciencedirect.com', 'proceedings.mlr.press',
  ];
  if (academicDomains.some(d => host.includes(d))) return true;

  // 通用检测：有 citation_title meta 标签
  if (metaContent('citation_title')) return true;

  // URL 中含 doi
  if (path.includes('/doi/') || host.includes('doi.org')) return true;

  return false;
}
