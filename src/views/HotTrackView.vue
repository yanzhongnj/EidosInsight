<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useCollectionStore } from '../stores/collection'
import { useAutomationStore } from '../stores/automation'
import { useAppStore } from '../stores/app'
import type { AiResult, TaskExecution } from '../types'

const router = useRouter()
const collectionStore = useCollectionStore()
const automationStore = useAutomationStore()
const appStore = useAppStore()

const activeTab = ref<'ai' | 'auto'>('ai')

// Track which task histories we've loaded
const loadedTaskHistories = ref<Record<string, TaskExecution[]>>({})

onMounted(async () => {
  await collectionStore.fetchCollections()
  await collectionStore.fetchAiResults()
  await automationStore.fetchTasks()
  // Load execution history for all enabled tasks
  for (const task of automationStore.tasks.filter(t => t.is_enabled)) {
    try {
      await automationStore.fetchHistory(task.id)
      loadedTaskHistories.value[task.id] = [...automationStore.executionHistory]
    } catch {
      // skip
    }
  }
})

// AI results sorted by date (newest first)
const sortedAiResults = computed(() => {
  return [...collectionStore.aiResults].sort((a, b) =>
    new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  )
})

// Collections that have AI results
const collectionsWithResults = computed(() => {
  const collectionIdsWithResults = new Set(
    collectionStore.aiResults
      .map(r => r.collection_id)
      .filter((id): id is string => id !== null)
  )
  return collectionStore.collections.filter(c => collectionIdsWithResults.has(c.id))
})

// All task executions flattened and sorted (newest first)
const allExecutions = computed(() => {
  const all: (TaskExecution & { task_name: string })[] = []
  for (const task of automationStore.tasks) {
    const history = loadedTaskHistories.value[task.id] ?? []
    for (const exec of history) {
      all.push({ ...exec, task_name: task.name })
    }
  }
  return all.sort((a, b) =>
    new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
  )
})

// Recent successful executions with results
const recentAutoResults = computed(() => {
  return allExecutions.value.filter(e => e.status === 'success' && e.result)
})

// Recent failed executions
const recentFailures = computed(() => {
  return allExecutions.value.filter(e => e.status === 'failed')
})

// Running tasks
const runningTasks = computed(() => {
  return allExecutions.value.filter(e => e.status === 'running')
})

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}

function formatDuration(ms: number | null): string {
  if (!ms) return '-'
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  return `${(ms / 60000).toFixed(1)}min`
}

function resultTypeLabel(type: string): string {
  switch (type) {
    case 'review': return '文献综述'
    case 'comparison': return '对比分析'
    case 'baseline': return '基线表格'
    default: return type
  }
}

function resultTypeIcon(type: string): string {
  switch (type) {
    case 'review': return '📑'
    case 'comparison': return '⚖️'
    case 'baseline': return '📊'
    default: return '🤖'
  }
}

function statusIcon(status: string): string {
  switch (status) {
    case 'running': return '⏳'
    case 'success': return '✅'
    case 'failed': return '❌'
    default: return '❓'
  }
}

function statusLabel(status: string): string {
  switch (status) {
    case 'running': return '运行中'
    case 'success': return '成功'
    case 'failed': return '失败'
    default: return status
  }
}

function openResult(id: string) {
  router.push(`/ai-result/${id}`)
}

function openCollection(id: string) {
  router.push(`/collection/${id}`)
}
</script>

<template>
  <div class="hot-track-view">
    <div class="view-header">
      <h1 class="page-title">🔥 热点跟踪</h1>
      <p class="page-desc">AI 分析结果与自动化任务产出</p>
    </div>

    <!-- Tabs -->
    <div class="tab-bar">
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'ai' }"
        @click="activeTab = 'ai'"
      >
        🤖 AI 分析
      </button>
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'auto' }"
        @click="activeTab = 'auto'"
      >
        ⏰ 自动化产出
      </button>
    </div>

    <!-- AI Analysis Results -->
    <div v-if="activeTab === 'ai'" class="tab-content">
      <!-- Recent AI results -->
      <div v-if="sortedAiResults.length > 0" class="result-list">
        <div
          v-for="result in sortedAiResults"
          :key="result.id"
          class="result-card"
          @click="openResult(result.id)"
        >
          <div class="result-icon">{{ resultTypeIcon(result.result_type) }}</div>
          <div class="result-info">
            <h3 class="result-title">{{ result.title || resultTypeLabel(result.result_type) }}</h3>
            <div class="result-meta">
              <span class="result-type-badge">{{ resultTypeLabel(result.result_type) }}</span>
              <span class="result-date">{{ formatDate(result.created_at) }}</span>
            </div>
          </div>
          <span class="result-arrow">›</span>
        </div>
      </div>

      <!-- Collections with results -->
      <div v-if="collectionsWithResults.length > 0" class="section">
        <h2 class="section-title">相关论文集</h2>
        <div class="collection-grid">
          <div
            v-for="col in collectionsWithResults"
            :key="col.id"
            class="collection-card"
            @click="openCollection(col.id)"
          >
            <span class="col-icon">🔬</span>
            <div class="col-info">
              <h4>{{ col.name }}</h4>
              <span class="col-count">{{ col.paper_count ?? 0 }} 篇文献</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="sortedAiResults.length === 0 && collectionsWithResults.length === 0" class="empty-state">
        <div class="empty-icon">🤖</div>
        <h2>暂无 AI 分析结果</h2>
        <p>在论文集中使用 AI 分析功能后，结果将显示在这里</p>
        <button class="btn btn-primary" @click="router.push('/collections')">
          前往论文集
        </button>
      </div>
    </div>

    <!-- Automation output tab -->
    <div v-if="activeTab === 'auto'" class="tab-content">
      <!-- Running tasks -->
      <div v-if="runningTasks.length > 0" class="section">
        <h2 class="section-title">⏳ 正在运行</h2>
        <div class="result-list">
          <div
            v-for="exec in runningTasks"
            :key="exec.id"
            class="result-card execution-card running"
          >
            <div class="result-icon">{{ statusIcon(exec.status) }}</div>
            <div class="result-info">
              <h3 class="result-title">{{ exec.task_name }}</h3>
              <div class="result-meta">
                <span class="result-type-badge running-badge">{{ statusLabel(exec.status) }}</span>
                <span class="result-date">开始于 {{ formatDate(exec.started_at) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Recent successful results -->
      <div v-if="recentAutoResults.length > 0" class="section">
        <h2 class="section-title">✅ 最近产出</h2>
        <div class="result-list">
          <div
            v-for="exec in recentAutoResults.slice(0, 20)"
            :key="exec.id"
            class="result-card execution-card"
          >
            <div class="result-icon">{{ statusIcon(exec.status) }}</div>
            <div class="result-info">
              <h3 class="result-title">{{ exec.task_name }}</h3>
              <div class="result-meta">
                <span class="result-type-badge success-badge">成功</span>
                <span class="result-date">{{ formatDate(exec.finished_at ?? exec.started_at) }}</span>
                <span v-if="exec.duration_ms" class="result-duration">{{ formatDuration(exec.duration_ms) }}</span>
              </div>
              <p v-if="exec.result" class="result-preview">{{ exec.result.slice(0, 120) }}{{ exec.result.length > 120 ? '...' : '' }}</p>
            </div>
            <span class="result-arrow">›</span>
          </div>
        </div>
      </div>

      <!-- Recent failures -->
      <div v-if="recentFailures.length > 0" class="section">
        <h2 class="section-title">❌ 失败记录</h2>
        <div class="result-list">
          <div
            v-for="exec in recentFailures.slice(0, 10)"
            :key="exec.id"
            class="result-card execution-card failed"
          >
            <div class="result-icon">{{ statusIcon(exec.status) }}</div>
            <div class="result-info">
              <h3 class="result-title">{{ exec.task_name }}</h3>
              <div class="result-meta">
                <span class="result-type-badge failed-badge">失败</span>
                <span class="result-date">{{ formatDate(exec.finished_at ?? exec.started_at) }}</span>
              </div>
              <p v-if="exec.error" class="result-preview error-text">{{ exec.error.slice(0, 120) }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="runningTasks.length === 0 && recentAutoResults.length === 0 && recentFailures.length === 0" class="empty-state">
        <div class="empty-icon">⏰</div>
        <h2>自动化产出</h2>
        <p>定时任务和自动化技能的执行结果将显示在这里</p>
        <button class="btn btn-primary" @click="router.push('/automation')">
          设置自动化任务
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hot-track-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
  min-height: 100%;
}

.view-header {
  margin-bottom: 20px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.page-desc {
  font-size: 14px;
  color: var(--text-tertiary);
}

/* Tabs */
.tab-bar {
  display: flex;
  gap: 4px;
  margin-bottom: 20px;
  border-bottom: 1px solid var(--border-primary);
  padding-bottom: 0;
}

.tab-btn {
  padding: 8px 16px;
  background: none;
  border: none;
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: all 0.15s;
}

.tab-btn:hover {
  color: var(--text-primary);
}

.tab-btn.active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
  font-weight: 500;
}

.tab-content {
  padding-top: 4px;
}

/* Result list */
.result-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.result-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all 0.15s;
}

.result-card:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-sm);
}

.result-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.result-type-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 9999px;
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary);
  font-weight: 500;
}

.result-date {
  font-size: 12px;
  color: var(--text-tertiary);
}

.result-arrow {
  font-size: 18px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

/* Section */
.section {
  margin-top: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.collection-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.collection-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s;
}

.collection-card:hover {
  border-color: var(--color-primary);
}

.col-icon {
  font-size: 20px;
}

.col-info h4 {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.col-count {
  font-size: 12px;
  color: var(--text-tertiary);
}

/* Empty state */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state h2 {
  font-size: 20px;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.empty-state p {
  margin-bottom: 24px;
  font-size: 14px;
}

/* Buttons */
.btn {
  padding: 8px 20px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.btn-primary:hover {
  filter: brightness(1.1);
}

/* Execution card */
.execution-card {
  transition: all 0.15s;
}

.execution-card.running {
  border-left: 3px solid #f59e0b;
}

.execution-card.failed {
  border-left: 3px solid #ef4444;
}

.result-preview {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.error-text {
  color: #ef4444;
}

.running-badge {
  background: color-mix(in srgb, #f59e0b 12%, transparent);
  color: #d97706;
}

.success-badge {
  background: color-mix(in srgb, #22c55e 12%, transparent);
  color: #16a34a;
}

.failed-badge {
  background: color-mix(in srgb, #ef4444 12%, transparent);
  color: #ef4444;
}

.result-duration {
  font-size: 11px;
  color: var(--text-tertiary);
  font-family: var(--font-mono, monospace);
}
</style>
