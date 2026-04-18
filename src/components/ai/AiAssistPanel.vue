<script setup lang="ts">
/**
 * AiAssistPanel — AI Assistant side panel
 *
 * Three tabs:
 *   - 术语解释 (explain)  : text input + optional context text
 *   - 摘要生成 (summarize): style picker (brief/structured/eli5)
 *   - Insight 提取 (insights): focus picker (all/contributions/methodology/gaps)
 *
 * All actions stream output via Tauri events.
 * The panel is self-contained and accepts a paper prop for context injection.
 */

import { ref, watch, computed, onMounted } from 'vue'
import { useAiAssist } from '../../composables/useAiAssist'
import type { AiTask, AiSummaryStyle, AiInsightFocus, Paper } from '../../types'

interface Props {
  paper?: Paper | null
  /** If true, the panel renders in compact/inline mode (no outer chrome) */
  inline?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  paper: null,
  inline: false,
})

// ── State ─────────────────────────────────────────────────────────────────────

type Tab = 'explain' | 'summarize' | 'insights'
const activeTab = ref<Tab>('summarize')

const ai = useAiAssist()

// Per-tab last task
const explainTask = ref<AiTask | null>(null)
const summarizeTask = ref<AiTask | null>(null)
const insightsTask = ref<AiTask | null>(null)

// LLM availability
const availability = ref<{ available: boolean; provider: string } | null>(null)

// ── Explain tab ───────────────────────────────────────────────────────────────
const termInput = ref('')
const contextInput = ref('')
const lang = ref<'zh' | 'en'>('zh')

async function runExplain() {
  if (!termInput.value.trim()) return
  const task = await ai.explainTerm({
    term: termInput.value.trim(),
    paper_id: props.paper?.id,
    context_text: contextInput.value.trim() || undefined,
    lang: lang.value,
  })
  explainTask.value = task
}

// ── Summarize tab ─────────────────────────────────────────────────────────────
const summaryStyle = ref<AiSummaryStyle>('structured')

async function runSummarize() {
  if (!props.paper?.id) return
  const task = await ai.summarizePaper({
    paper_id: props.paper.id,
    style: summaryStyle.value,
    lang: lang.value,
  })
  summarizeTask.value = task
}

// ── Insights tab ──────────────────────────────────────────────────────────────
const insightFocus = ref<AiInsightFocus>('all')

async function runInsights() {
  if (!props.paper?.id) return
  const task = await ai.extractInsights({
    paper_id: props.paper.id,
    focus: insightFocus.value,
    lang: lang.value,
  })
  insightsTask.value = task
}

// ── Active task for current tab ───────────────────────────────────────────────
const currentTask = computed<AiTask | null>(() => {
  if (activeTab.value === 'explain') return explainTask.value
  if (activeTab.value === 'summarize') return summarizeTask.value
  return insightsTask.value
})

const currentTaskLive = computed<AiTask | null>(() => {
  if (!currentTask.value) return null
  // Get live reactive version from taskMap
  return ai.getTask(currentTask.value.task_id) ?? currentTask.value
})

function clearCurrent() {
  if (currentTask.value) {
    ai.clearTask(currentTask.value.task_id)
    if (activeTab.value === 'explain') explainTask.value = null
    else if (activeTab.value === 'summarize') summarizeTask.value = null
    else insightsTask.value = null
  }
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  availability.value = await ai.checkAvailability().catch(() => ({
    available: false,
    provider: 'none',
  }))
})

// Reset tasks when paper changes
watch(() => props.paper?.id, () => {
  explainTask.value = null
  summarizeTask.value = null
  insightsTask.value = null
  ai.clearAllTasks()
})

// ── Markdown-like render (simple; full md renderer is overkill here) ──────────
// Convert **bold**, *italic*, and \n\n paragraphs to basic HTML for display
function renderMarkdown(text: string): string {
  if (!text) return ''
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    // Bold
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    // Italic
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // Code
    .replace(/`(.+?)`/g, '<code>$1</code>')
    // Paragraph breaks
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br>')
    // Wrap everything in a paragraph
    .replace(/^/, '<p>')
    .replace(/$/, '</p>')
}

// Provider label
const providerLabel = computed(() => {
  if (!availability.value) return '检测中…'
  if (!availability.value.available) return '未配置 AI'
  return availability.value.provider === 'ollama' ? 'Ollama (本地)' : '云端 API'
})

const providerClass = computed(() => {
  if (!availability.value) return 'provider-checking'
  if (!availability.value.available) return 'provider-unavailable'
  return availability.value.provider === 'ollama'
    ? 'provider-local'
    : 'provider-cloud'
})
</script>

<template>
  <div class="ai-panel" :class="{ 'ai-panel--inline': inline }">
    <!-- Header -->
    <div class="ai-panel-header">
      <div class="ai-panel-title">
        <svg class="ai-icon" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd"
            d="M9.664 1.319a.75.75 0 01.672 0 41.059 41.059 0 018.198 5.424.75.75 0 01-.254 1.285 31.372 31.372 0 00-7.86 3.83.75.75 0 01-.84 0 31.508 31.508 0 00-2.08-1.287V9.482a31.516 31.516 0 00-5.774-2.458.75.75 0 01-.254-1.285 41.059 41.059 0 018.198-5.424zM6 12.459v3.116a.75.75 0 00.894.734 31.39 31.39 0 005.332-1.76A.75.75 0 0013 13.85v-3.391a33.016 33.016 0 00-7 2z"
            clip-rule="evenodd" />
        </svg>
        <span>AI 助理</span>
      </div>
      <div class="provider-badge" :class="providerClass">{{ providerLabel }}</div>
    </div>

    <!-- No LLM warning -->
    <div v-if="availability && !availability.available" class="ai-unavailable">
      <p>未配置 AI 后端。请前往 <strong>设置 → AI 配置</strong> 连接 Ollama 或云端 API。</p>
    </div>

    <template v-else>
      <!-- Tabs -->
      <div class="ai-tabs">
        <button
          class="ai-tab"
          :class="{ active: activeTab === 'summarize' }"
          @click="activeTab = 'summarize'"
        >摘要</button>
        <button
          class="ai-tab"
          :class="{ active: activeTab === 'insights' }"
          @click="activeTab = 'insights'"
        >Insights</button>
        <button
          class="ai-tab"
          :class="{ active: activeTab === 'explain' }"
          @click="activeTab = 'explain'"
        >术语解释</button>
      </div>

      <!-- ── Summarize tab ── -->
      <div v-if="activeTab === 'summarize'" class="tab-content">
        <div v-if="!paper" class="no-paper-hint">
          请先打开一篇文献再使用 AI 摘要功能。
        </div>
        <template v-else>
          <div class="form-row">
            <label class="form-label">摘要风格</label>
            <div class="style-pills">
              <button
                v-for="s in (['structured', 'brief', 'eli5'] as const)"
                :key="s"
                class="pill"
                :class="{ active: summaryStyle === s }"
                @click="summaryStyle = s"
              >
                {{ s === 'structured' ? '结构化' : s === 'brief' ? '简短' : '通俗' }}
              </button>
            </div>
          </div>
          <div class="form-row">
            <label class="form-label">语言</label>
            <div class="lang-pills">
              <button class="pill" :class="{ active: lang === 'zh' }" @click="lang = 'zh'">中文</button>
              <button class="pill" :class="{ active: lang === 'en' }" @click="lang = 'en'">English</button>
            </div>
          </div>
          <div class="action-row">
            <button
              class="btn btn-primary btn-ai"
              :disabled="!!(currentTaskLive?.streaming)"
              @click="runSummarize"
            >
              <span v-if="currentTaskLive?.streaming" class="btn-spinner"></span>
              <span v-else>生成摘要</span>
            </button>
            <button v-if="summarizeTask" class="btn-ghost btn-sm" @click="clearCurrent">清除</button>
          </div>
        </template>
      </div>

      <!-- ── Insights tab ── -->
      <div v-if="activeTab === 'insights'" class="tab-content">
        <div v-if="!paper" class="no-paper-hint">
          请先打开一篇文献再提取 Insights。
        </div>
        <template v-else>
          <div class="form-row">
            <label class="form-label">聚焦方向</label>
            <div class="style-pills flex-wrap">
              <button
                v-for="f in (['all', 'contributions', 'methodology', 'gaps'] as const)"
                :key="f"
                class="pill"
                :class="{ active: insightFocus === f }"
                @click="insightFocus = f"
              >
                {{ { all: '全面', contributions: '贡献', methodology: '方法', gaps: '局限' }[f] }}
              </button>
            </div>
          </div>
          <div class="form-row">
            <label class="form-label">语言</label>
            <div class="lang-pills">
              <button class="pill" :class="{ active: lang === 'zh' }" @click="lang = 'zh'">中文</button>
              <button class="pill" :class="{ active: lang === 'en' }" @click="lang = 'en'">English</button>
            </div>
          </div>
          <div class="action-row">
            <button
              class="btn btn-primary btn-ai"
              :disabled="!!(currentTaskLive?.streaming)"
              @click="runInsights"
            >
              <span v-if="currentTaskLive?.streaming" class="btn-spinner"></span>
              <span v-else>提取 Insights</span>
            </button>
            <button v-if="insightsTask" class="btn-ghost btn-sm" @click="clearCurrent">清除</button>
          </div>
        </template>
      </div>

      <!-- ── Explain tab ── -->
      <div v-if="activeTab === 'explain'" class="tab-content">
        <div class="form-row">
          <label class="form-label">术语</label>
          <input
            v-model="termInput"
            class="input"
            placeholder="输入要解释的专业术语…"
            @keydown.enter="runExplain"
          />
        </div>
        <div class="form-row">
          <label class="form-label">上下文 <span class="optional">可选</span></label>
          <textarea
            v-model="contextInput"
            class="input textarea-sm"
            placeholder="粘贴含有该术语的句子，帮助 AI 理解语境…"
            rows="2"
          ></textarea>
        </div>
        <div class="form-row">
          <label class="form-label">语言</label>
          <div class="lang-pills">
            <button class="pill" :class="{ active: lang === 'zh' }" @click="lang = 'zh'">中文</button>
            <button class="pill" :class="{ active: lang === 'en' }" @click="lang = 'en'">English</button>
          </div>
        </div>
        <div class="action-row">
          <button
            class="btn btn-primary btn-ai"
            :disabled="!termInput.trim() || !!(currentTaskLive?.streaming)"
            @click="runExplain"
          >
            <span v-if="currentTaskLive?.streaming" class="btn-spinner"></span>
            <span v-else>解释术语</span>
          </button>
          <button v-if="explainTask" class="btn-ghost btn-sm" @click="clearCurrent">清除</button>
        </div>
      </div>

      <!-- ── Output area ── -->
      <div v-if="currentTaskLive" class="ai-output-area">
        <!-- Streaming indicator -->
        <div v-if="currentTaskLive.streaming" class="streaming-bar">
          <div class="streaming-dots">
            <span></span><span></span><span></span>
          </div>
          <span class="streaming-label">AI 生成中…</span>
        </div>

        <!-- Error state -->
        <div v-if="currentTaskLive.error" class="ai-error-box">
          <svg viewBox="0 0 16 16" fill="currentColor" width="14" height="14">
            <path d="M8 1a7 7 0 100 14A7 7 0 008 1zm.75 4.75a.75.75 0 00-1.5 0v2.5a.75.75 0 001.5 0v-2.5zM8 10.5a.75.75 0 100 1.5.75.75 0 000-1.5z"/>
          </svg>
          <span>{{ currentTaskLive.error }}</span>
        </div>

        <!-- Content output -->
        <div
          v-if="currentTaskLive.content"
          class="ai-content"
          v-html="renderMarkdown(currentTaskLive.content)"
        ></div>

        <!-- Streaming cursor -->
        <span v-if="currentTaskLive.streaming && currentTaskLive.content" class="cursor-blink">▋</span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.ai-panel {
  display: flex;
  flex-direction: column;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  overflow: hidden;
  height: 100%;
  min-height: 0;
}

.ai-panel--inline {
  border: none;
  border-radius: 0;
  background: transparent;
}

/* Header */
.ai-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-primary);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.ai-panel-title {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.ai-icon {
  width: 16px;
  height: 16px;
  color: var(--color-primary);
}

.provider-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.provider-local {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.provider-cloud {
  background: rgba(99, 102, 241, 0.15);
  color: var(--color-primary);
}

.provider-unavailable {
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
}

.provider-checking {
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
}

/* Unavailable */
.ai-unavailable {
  padding: 20px 16px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}

/* Tabs */
.ai-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
  background: var(--bg-primary);
}

.ai-tab {
  flex: 1;
  padding: 9px 4px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
}

.ai-tab:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.ai-tab.active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
  background: transparent;
}

/* Tab content */
.tab-content {
  padding: 14px 16px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.no-paper-hint {
  font-size: 13px;
  color: var(--text-tertiary);
  padding: 8px 0;
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.form-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.optional {
  font-size: 11px;
  color: var(--text-tertiary);
  font-weight: 400;
}

.input {
  width: 100%;
  padding: 7px 10px;
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  transition: border-color var(--transition-fast);
  box-sizing: border-box;
}

.input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.textarea-sm {
  resize: vertical;
  min-height: 50px;
  font-family: inherit;
  line-height: 1.5;
}

/* Pills */
.style-pills,
.lang-pills {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.pill {
  padding: 4px 12px;
  border-radius: var(--radius-full);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border-primary);
  background: var(--bg-primary);
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.pill:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.pill.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: var(--text-inverse);
}

/* Action row */
.action-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn {
  padding: 7px 16px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all var(--transition-fast);
}

.btn-primary {
  background: var(--color-primary);
  color: var(--text-inverse);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-ai {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 90px;
  justify-content: center;
}

.btn-ghost {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  transition: all var(--transition-fast);
}

.btn-ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

/* Spinner */
.btn-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.4);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* Output area */
.ai-output-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 16px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* Streaming bar */
.streaming-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: rgba(99, 102, 241, 0.08);
  border-radius: var(--radius-md);
  border-left: 3px solid var(--color-primary);
}

.streaming-dots {
  display: flex;
  gap: 3px;
}

.streaming-dots span {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--color-primary);
  animation: bounce 1.2s infinite;
}

.streaming-dots span:nth-child(2) { animation-delay: 0.2s; }
.streaming-dots span:nth-child(3) { animation-delay: 0.4s; }

@keyframes bounce {
  0%, 60%, 100% { transform: translateY(0); }
  30% { transform: translateY(-4px); }
}

.streaming-label {
  font-size: 12px;
  color: var(--color-primary);
  font-weight: 500;
}

/* Error */
.ai-error-box {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--radius-md);
  font-size: 12px;
  color: var(--color-danger);
  line-height: 1.5;
}

.ai-error-box svg {
  flex-shrink: 0;
  margin-top: 1px;
}

/* Content */
.ai-content {
  font-size: 13px;
  line-height: 1.75;
  color: var(--text-secondary);
}

.ai-content :deep(p) {
  margin: 0 0 10px;
}

.ai-content :deep(p:last-child) {
  margin-bottom: 0;
}

.ai-content :deep(strong) {
  color: var(--text-primary);
  font-weight: 600;
}

.ai-content :deep(em) {
  font-style: italic;
}

.ai-content :deep(code) {
  font-family: var(--font-mono);
  font-size: 12px;
  background: var(--bg-tertiary);
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--color-primary);
}

/* Streaming cursor */
.cursor-blink {
  display: inline;
  color: var(--color-primary);
  font-weight: 700;
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  50% { opacity: 0; }
}
</style>
