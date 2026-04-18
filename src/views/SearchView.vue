<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useSearch } from '../composables/useSearch'

const router = useRouter()
const {
  query, searchField, readingStatusFilter, yearFrom, yearTo, favoritesOnly,
  showAdvanced, results, searching, searchError, currentPage, totalPages,
  suggestions, suggestVisible, history, hasActiveFilters,
  doSearch, goToPage, selectSuggestion, useHistoryItem, removeHistoryItem, clearHistory, resetFilters,
} = useSearch()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    doSearch(0)
  } else if (e.key === 'Escape') {
    suggestVisible.value = false
  }
}

function navigateToPaper(id: string) {
  router.push(`/paper/${id}`)
}

/** Sanitize: only allow <mark> and </mark> tags in snippets (from our own backend). */
function safeHighlight(html: string): string {
  return html
    .replace(/&/g, '&amp;')
    .replace(/</g, '<')
    .replace(/>/g, '>')
    // Restore safe <mark> tags
    .replace(/&lt;mark&gt;/g, '<mark>')
    .replace(/&lt;\/mark&gt;/g, '</mark>')
}
</script>

<template>
  <div class="search-view">
    <!-- ── Header ── -->
    <div class="search-header">
      <h1 class="page-title">搜索</h1>
      <p class="page-subtitle">全文检索论文、摘要、笔记</p>
    </div>

    <!-- ── Search bar ── -->
    <div class="search-bar-wrap">
      <div class="search-input-row">
        <!-- Field scope selector -->
        <select v-model="searchField" class="field-select">
          <option value="all">全文</option>
          <option value="title">标题</option>
          <option value="abstract">摘要</option>
          <option value="authors">作者</option>
        </select>

        <!-- Main input with suggestion dropdown -->
        <div class="input-wrap" @click.stop>
          <span class="search-icon">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="6.5" cy="6.5" r="5" stroke="currentColor" stroke-width="1.5"/>
              <path d="M10.5 10.5L14 14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </span>
          <input
            v-model="query"
            type="text"
            class="search-input"
            placeholder="输入关键词 — 支持 AND、OR、NOT、&quot;精确短语&quot;"
            autocomplete="off"
            spellcheck="false"
            @keydown="handleKeydown"
            @focus="suggestVisible = suggestions?.items.length ? true : false"
            @click.stop
          />
          <button v-if="query" class="clear-btn" title="清除" @click="query = ''; suggestVisible = false">✕</button>

          <!-- Suggestion dropdown -->
          <Transition name="suggest">
            <div v-if="suggestVisible" class="suggest-panel" @click.stop>
              <div
                v-for="item in suggestions?.items"
                :key="item.id"
                class="suggest-item"
                @mousedown.prevent="selectSuggestion(item, navigateToPaper)"
              >
                <div class="suggest-title">{{ item.title }}</div>
                <div v-if="item.authors" class="suggest-meta">
                  <span>{{ item.authors.split(',').slice(0, 2).join(', ') }}</span>
                  <span v-if="item.publication_date">· {{ item.publication_date.slice(0, 4) }}</span>
                </div>
              </div>
            </div>
          </Transition>
        </div>

        <!-- Search button -->
        <button
          class="btn btn-primary search-btn"
          :disabled="searching || !query.trim()"
          @click="doSearch(0)"
        >
          <span v-if="searching" class="spin-icon">⟳</span>
          <span v-else>搜索</span>
        </button>

        <!-- Advanced toggle -->
        <button
          class="btn-adv-toggle"
          :class="{ active: showAdvanced || hasActiveFilters }"
          :title="showAdvanced ? '收起高级筛选' : '展开高级筛选'"
          @click="showAdvanced = !showAdvanced"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <line x1="2" y1="4" x2="14" y2="4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="4" y1="8" x2="12" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="6" y1="12" x2="10" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <span v-if="hasActiveFilters" class="filter-dot" />
        </button>
      </div>

      <!-- ── Advanced filters ── -->
      <Transition name="expand">
        <div v-if="showAdvanced" class="advanced-filters">
          <div class="filter-row">
            <div class="filter-group">
              <label>阅读状态</label>
              <select v-model="readingStatusFilter" class="filter-select">
                <option value="">全部</option>
                <option value="unread">未读</option>
                <option value="reading">阅读中</option>
                <option value="read">已读</option>
              </select>
            </div>
            <div class="filter-group">
              <label>发布年份</label>
              <div class="year-range">
                <input v-model="yearFrom" type="text" class="year-input" placeholder="从 2020" maxlength="4" />
                <span class="range-dash">—</span>
                <input v-model="yearTo" type="text" class="year-input" placeholder="到 2024" maxlength="4" />
              </div>
            </div>
            <div class="filter-group filter-checkbox-group">
              <label>
                <input v-model="favoritesOnly" type="checkbox" />
                仅收藏
              </label>
            </div>
            <button v-if="hasActiveFilters" class="btn-reset-filters" @click="resetFilters">
              重置筛选
            </button>
          </div>
          <p class="filter-hint">
            提示：支持 FTS5 语法 —
            <code>neural AND network</code>、
            <code>"attention mechanism"</code>、
            <code>transformer NOT RNN</code>、
            <code>transform*</code>（前缀匹配）
          </p>
        </div>
      </Transition>
    </div>

    <!-- ── Error ── -->
    <div v-if="searchError" class="search-error">
      <span>⚠️ {{ searchError }}</span>
    </div>

    <!-- ── Results ── -->
    <div v-if="results" class="results-section">
      <!-- Summary bar -->
      <div class="results-summary-bar">
        <span class="results-count">
          找到 <strong>{{ results.total }}</strong> 篇相关论文
          <span v-if="results.query" class="query-badge">{{ results.query }}</span>
        </span>
        <span class="results-time" v-if="results.papers.length">
          显示第 {{ currentPage * 20 + 1 }}–{{ Math.min((currentPage + 1) * 20, results.total) }} 篇
        </span>
      </div>

      <!-- Result items -->
      <div v-if="results.papers.length" class="results-list">
        <div
          v-for="item in results.papers"
          :key="item.paper.id"
          class="result-item"
          @click="navigateToPaper(item.paper.id)"
        >
          <!-- Left: content -->
          <div class="result-content">
            <!-- Title (with possible title highlight) -->
            <h3 class="result-title">{{ item.paper.title }}</h3>

            <!-- Authors -->
            <p v-if="item.paper.authors" class="result-authors">
              {{ item.paper.authors.split(',').slice(0, 4).join(', ') }}
              <span v-if="item.paper.authors.split(',').length > 4" class="more-authors">
                等 {{ item.paper.authors.split(',').length }} 人
              </span>
            </p>

            <!-- Snippet highlight (HTML from FTS5 snippet()) -->
            <p
              v-if="item.highlight"
              class="result-snippet"
              v-html="safeHighlight(item.highlight)"
            />
            <!-- Fallback: first 200 chars of abstract -->
            <p v-else-if="item.paper.abstract_text" class="result-snippet result-snippet--plain">
              {{ item.paper.abstract_text.slice(0, 220) }}{{ item.paper.abstract_text.length > 220 ? '…' : '' }}
            </p>

            <!-- Meta row -->
            <div class="result-meta">
              <span v-if="item.paper.journal" class="meta-chip meta-journal">
                {{ item.paper.journal }}
              </span>
              <span v-if="item.paper.publication_date" class="meta-chip meta-year">
                {{ item.paper.publication_date.slice(0, 4) }}
              </span>
              <span class="meta-chip meta-status" :class="`status--${item.paper.reading_status}`">
                {{ item.paper.reading_status === 'read' ? '已读'
                   : item.paper.reading_status === 'reading' ? '阅读中'
                   : '未读' }}
              </span>
              <span v-if="item.paper.is_favorite" class="meta-chip meta-fav">★ 收藏</span>
              <span v-if="item.paper.doi" class="meta-chip meta-doi">DOI</span>
            </div>
          </div>

          <!-- Right: score bar (relative relevance) -->
          <div class="result-score-bar" :title="`相关度得分: ${Math.abs(item.score).toFixed(3)}`">
            <div
              class="score-fill"
              :style="{ height: `${Math.min(100, Math.abs(item.score) * 20)}%` }"
            />
          </div>
        </div>
      </div>

      <!-- Empty results -->
      <div v-else class="no-results">
        <div class="no-results-icon">🔍</div>
        <h3>未找到相关论文</h3>
        <p>尝试调整关键词，或在高级筛选中放宽条件</p>
        <div class="search-tips">
          <p>搜索技巧：</p>
          <ul>
            <li>使用 <code>*</code> 做前缀匹配，如 <code>transform*</code></li>
            <li>用 <code>"..."</code> 搜索精确短语，如 <code>"large language model"</code></li>
            <li>用 <code>AND</code>/<code>OR</code>/<code>NOT</code> 组合词，如 <code>BERT NOT GPT</code></li>
            <li>切换"搜索范围"到"标题"可快速精确定位</li>
          </ul>
        </div>
      </div>

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="pagination">
        <button class="page-btn" :disabled="currentPage === 0" @click="goToPage(0)">«</button>
        <button class="page-btn" :disabled="currentPage === 0" @click="goToPage(currentPage - 1)">‹</button>
        <span class="page-info">{{ currentPage + 1 }} / {{ totalPages }}</span>
        <button class="page-btn" :disabled="currentPage >= totalPages - 1" @click="goToPage(currentPage + 1)">›</button>
        <button class="page-btn" :disabled="currentPage >= totalPages - 1" @click="goToPage(totalPages - 1)">»</button>
      </div>
    </div>

    <!-- ── Empty / History state (before any search) ── -->
    <div v-else-if="!searching" class="search-idle">
      <!-- Search history -->
      <div v-if="history.length" class="history-section">
        <div class="history-header">
          <span class="history-title">最近搜索</span>
          <button class="btn-clear-history" @click="clearHistory">清除全部</button>
        </div>
        <div class="history-list">
          <button
            v-for="h in history"
            :key="h"
            class="history-item"
          >
            <span class="history-icon">🕐</span>
            <span class="history-text" @click="useHistoryItem(h)">{{ h }}</span>
            <span class="history-remove" @click.stop="removeHistoryItem(h)">✕</span>
          </button>
        </div>
      </div>

      <!-- Empty state when no history -->
      <div v-else class="empty-state">
        <div class="empty-icon">
          <svg width="56" height="56" viewBox="0 0 56 56" fill="none">
            <circle cx="24" cy="24" r="18" stroke="var(--text-tertiary)" stroke-width="2"/>
            <path d="M37 37L50 50" stroke="var(--text-tertiary)" stroke-width="2" stroke-linecap="round"/>
            <path d="M18 24h12M24 18v12" stroke="var(--color-primary)" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </div>
        <h3>全文检索论文库</h3>
        <p>搜索标题、摘要、作者，快速定位目标论文</p>
        <div class="quick-tips">
          <span class="tip-chip">试试：<code>attention mechanism</code></span>
          <span class="tip-chip">或：<code>"neural network" AND image</code></span>
        </div>
      </div>
    </div>

    <!-- ── Loading state ── -->
    <div v-if="searching" class="searching-state">
      <div class="loading-ring" />
      <span>正在检索…</span>
    </div>
  </div>
</template>

<style scoped>
/* ── Layout ── */
.search-view {
  max-width: 860px;
  margin: 0 auto;
  padding-bottom: 40px;
}

.search-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.page-subtitle {
  font-size: 13px;
  color: var(--text-tertiary);
}

/* ── Search bar ── */
.search-bar-wrap {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 12px 14px;
  margin-bottom: 20px;
}

.search-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-select {
  height: 38px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  flex-shrink: 0;
  min-width: 70px;
}

.input-wrap {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--text-tertiary);
  display: flex;
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 38px;
  padding: 0 32px 0 34px;
  background: var(--bg-primary);
  border: 1.5px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  transition: border-color 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.clear-btn {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 11px;
  padding: 4px;
  border-radius: var(--radius-sm);
  transition: color 0.15s;
}

.clear-btn:hover {
  color: var(--text-secondary);
}

.search-btn {
  height: 38px;
  padding: 0 18px;
  flex-shrink: 0;
  min-width: 68px;
}

.spin-icon {
  display: inline-block;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Advanced toggle */
.btn-adv-toggle {
  position: relative;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-tertiary);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.btn-adv-toggle:hover,
.btn-adv-toggle.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.filter-dot {
  position: absolute;
  top: 5px;
  right: 5px;
  width: 6px;
  height: 6px;
  background: var(--color-primary);
  border-radius: 50%;
}

/* ── Suggestion dropdown ── */
.suggest-panel {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  z-index: 200;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.suggest-item {
  padding: 9px 14px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-subtle, var(--border-color));
  transition: background 0.1s;
}

.suggest-item:last-child {
  border-bottom: none;
}

.suggest-item:hover {
  background: var(--bg-hover, var(--bg-tertiary));
}

.suggest-title {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.suggest-meta {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 2px;
  display: flex;
  gap: 4px;
}

.suggest-enter-active,
.suggest-leave-active {
  transition: opacity 0.12s, transform 0.12s;
}

.suggest-enter-from,
.suggest-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* ── Advanced filters ── */
.advanced-filters {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.filter-row {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-group label {
  font-size: 11px;
  color: var(--text-tertiary);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.filter-select {
  height: 32px;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  min-width: 100px;
}

.year-range {
  display: flex;
  align-items: center;
  gap: 6px;
}

.year-input {
  width: 70px;
  height: 32px;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
}

.range-dash {
  color: var(--text-tertiary);
  font-size: 12px;
}

.filter-checkbox-group label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  padding-top: 4px;
  text-transform: none;
  letter-spacing: 0;
}

.btn-reset-filters {
  height: 32px;
  padding: 0 12px;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-tertiary);
  font-size: 12px;
  cursor: pointer;
  align-self: flex-end;
  transition: all 0.15s;
}

.btn-reset-filters:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.filter-hint {
  margin-top: 10px;
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.6;
}

.filter-hint code {
  background: var(--bg-tertiary);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--color-primary);
}

.expand-enter-active,
.expand-leave-active {
  transition: opacity 0.2s, transform 0.2s;
  transform-origin: top;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  transform: scaleY(0.9);
}

/* ── Error ── */
.search-error {
  background: color-mix(in srgb, #f87171 10%, var(--bg-secondary));
  border: 1px solid color-mix(in srgb, #f87171 30%, transparent);
  border-radius: var(--radius-md);
  padding: 10px 14px;
  font-size: 13px;
  color: #f87171;
  margin-bottom: 16px;
}

/* ── Results ── */
.results-summary-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
  font-size: 13px;
  color: var(--text-tertiary);
}

.results-count strong {
  color: var(--text-primary);
  font-weight: 600;
}

.query-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 8px;
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary);
  border-radius: var(--radius-full, 999px);
  font-size: 12px;
  font-weight: 500;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* ── Result item ── */
.result-item {
  display: flex;
  gap: 10px;
  align-items: stretch;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 14px 16px;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s, box-shadow 0.15s;
}

.result-item:hover {
  border-color: var(--color-primary);
  background: var(--bg-hover, var(--bg-secondary));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.result-content {
  flex: 1;
  min-width: 0;
}

.result-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 4px;
  line-height: 1.4;
}

.result-authors {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.more-authors {
  color: var(--text-tertiary);
}

/* FTS5 snippet highlight */
.result-snippet {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin-bottom: 8px;
}

.result-snippet--plain {
  color: var(--text-tertiary);
}

/* Global: FTS5 <mark> highlight style */
:deep(mark) {
  background: color-mix(in srgb, var(--color-primary) 20%, transparent);
  color: var(--text-primary);
  border-radius: 2px;
  padding: 0 1px;
  font-weight: 600;
}

/* Meta chips */
.result-meta {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 4px;
}

.meta-chip {
  display: inline-block;
  padding: 1px 8px;
  border-radius: var(--radius-full, 999px);
  font-size: 11px;
  font-weight: 500;
}

.meta-journal {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.meta-year {
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
}

.meta-status.status--unread {
  background: color-mix(in srgb, #94a3b8 12%, transparent);
  color: #94a3b8;
}

.meta-status.status--reading {
  background: color-mix(in srgb, #60a5fa 15%, transparent);
  color: #60a5fa;
}

.meta-status.status--read {
  background: color-mix(in srgb, #34d399 15%, transparent);
  color: #34d399;
}

.meta-fav {
  background: color-mix(in srgb, #fbbf24 15%, transparent);
  color: #fbbf24;
}

.meta-doi {
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
  border: 1px solid var(--border-color);
}

/* Score bar */
.result-score-bar {
  width: 4px;
  border-radius: 2px;
  background: var(--bg-tertiary);
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
  min-height: 40px;
  align-self: stretch;
}

.score-fill {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: var(--color-primary);
  border-radius: 2px;
  opacity: 0.6;
  transition: height 0.3s;
}

/* ── No results ── */
.no-results {
  text-align: center;
  padding: 48px 20px;
}

.no-results-icon {
  font-size: 40px;
  margin-bottom: 12px;
  opacity: 0.5;
}

.no-results h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.no-results p {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-bottom: 20px;
}

.search-tips {
  display: inline-block;
  text-align: left;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 14px 18px;
  font-size: 13px;
}

.search-tips p {
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.search-tips ul {
  margin: 0;
  padding-left: 16px;
  color: var(--text-tertiary);
  line-height: 2;
}

.search-tips code {
  background: var(--bg-tertiary);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--color-primary);
}

/* ── Pagination ── */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 24px;
}

.page-btn {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  transition: all 0.15s;
}

.page-btn:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.page-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-info {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 60px;
  text-align: center;
}

/* ── Idle / History ── */
.search-idle {
  margin-top: 8px;
}

.history-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 16px;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.history-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.btn-clear-history {
  background: none;
  border: none;
  font-size: 12px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  transition: color 0.15s;
}

.btn-clear-history:hover {
  color: var(--text-secondary);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  background: none;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s;
  width: 100%;
}

.history-item:hover {
  background: var(--bg-tertiary);
}

.history-icon {
  font-size: 13px;
  flex-shrink: 0;
  color: var(--text-tertiary);
}

.history-text {
  flex: 1;
  font-size: 13px;
  color: var(--text-secondary);
}

.history-remove {
  font-size: 11px;
  color: var(--text-tertiary);
  padding: 2px 5px;
  border-radius: var(--radius-sm);
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
}

.history-item:hover .history-remove {
  opacity: 1;
}

.history-remove:hover {
  color: var(--text-primary);
}

/* ── Empty state ── */
.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.7;
}

.empty-state h3 {
  font-size: 17px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.empty-state p {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-bottom: 24px;
}

.quick-tips {
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}

.tip-chip {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-full, 999px);
  padding: 5px 14px;
  font-size: 13px;
  color: var(--text-tertiary);
}

.tip-chip code {
  color: var(--color-primary);
  background: none;
  font-size: 13px;
}

/* ── Loading ── */
.searching-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 60px 20px;
  color: var(--text-tertiary);
  font-size: 14px;
}

.loading-ring {
  width: 32px;
  height: 32px;
  border: 3px solid var(--bg-tertiary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.75s linear infinite;
}
</style>
