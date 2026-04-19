<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useCollectionStore } from '../stores/collection'
import { usePaperStore } from '../stores/paper'

const route = useRoute()
const router = useRouter()
const collectionStore = useCollectionStore()
const paperStore = usePaperStore()

const collectionId = computed(() => route.params.id as string)
const showAiDialog = ref(false)
const aiDialogType = ref<'review' | 'comparison' | 'baseline'>('review')
const generating = ref(false)
const showAddPapers = ref(false)
const searchQuery = ref('')

const selectedPapersToAdd = ref<Set<string>>(new Set())

const comparisonDimensions = ref<string[]>([
  '方法', '数据集', '实验设计', '评估指标', '局限性', '创新点', '可复现性'
])

onMounted(async () => {
  await collectionStore.fetchCollection(collectionId.value)
  await collectionStore.fetchCollectionPapers(collectionId.value)
  await collectionStore.fetchCollectionAiResults(collectionId.value)
})

function openReader(id: string) {
  router.push(`/reader/${id}`)
}

function openPaper(id: string) {
  router.push(`/paper/${id}`)
}

async function removePaper(paperId: string) {
  await collectionStore.removePaper(collectionId.value, paperId)
}

async function generateAi() {
  generating.value = true
  try {
    if (aiDialogType.value === 'review') {
      await collectionStore.doGenerateReview({ collection_id: collectionId.value })
    } else if (aiDialogType.value === 'comparison') {
      await collectionStore.doGenerateComparison({
        collection_id: collectionId.value,
        dimensions: comparisonDimensions.value,
      })
    } else {
      await collectionStore.doGenerateBaseline({ collection_id: collectionId.value })
    }
    showAiDialog.value = false
    await collectionStore.fetchCollectionAiResults(collectionId.value)
  } catch (e) {
    console.error('AI generation failed:', e)
  } finally {
    generating.value = false
  }
}

function viewAiResult(id: string) {
  router.push(`/ai-result/${id}`)
}

// Add papers from library
const libraryPapers = computed(() => {
  return paperStore.papers.filter(p => !collectionStore.collectionPapers.find(cp => cp.id === p.id))
})

const filteredLibraryPapers = computed(() => {
  if (!searchQuery.value) return libraryPapers.value.slice(0, 20)
  const q = searchQuery.value.toLowerCase()
  return libraryPapers.value.filter(p => p.title.toLowerCase().includes(q)).slice(0, 20)
})

async function addSelectedPapers() {
  if (selectedPapersToAdd.value.size === 0) return
  await collectionStore.addPapersBatch(collectionId.value, Array.from(selectedPapersToAdd.value))
  selectedPapersToAdd.value.clear()
  showAddPapers.value = false
  await collectionStore.fetchCollectionPapers(collectionId.value)
}

function togglePaperSelection(paperId: string) {
  if (selectedPapersToAdd.value.has(paperId)) {
    selectedPapersToAdd.value.delete(paperId)
  } else {
    selectedPapersToAdd.value.add(paperId)
  }
}
</script>

<template>
  <div class="collection-view">
    <div class="collection-header" v-if="collectionStore.currentCollection">
      <div class="header-left">
        <button class="btn-back" @click="router.push('/')">← 返回</button>
        <div class="collection-info">
          <span class="collection-dot" :style="{ backgroundColor: collectionStore.currentCollection.color || '#6366f1' }"></span>
          <h1 class="collection-title">{{ collectionStore.currentCollection.name }}</h1>
          <span class="paper-count">{{ collectionStore.currentCollection.paper_count || 0 }} 篇论文</span>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary" @click="showAddPapers = true">+ 添加论文</button>
        <button class="btn btn-primary" @click="showAiDialog = true">🤖 AI 分析</button>
      </div>
    </div>

    <p v-if="collectionStore.currentCollection?.description" class="collection-desc">
      {{ collectionStore.currentCollection.description }}
    </p>

    <!-- Papers list -->
    <div v-if="collectionStore.collectionPapers.length" class="papers-list">
      <div v-for="paper in collectionStore.collectionPapers" :key="paper.id" class="paper-row">
        <div class="paper-main" @click="openPaper(paper.id)">
          <span class="paper-icon">📄</span>
          <div class="paper-info">
            <span class="paper-title">{{ paper.title }}</span>
            <span class="paper-authors">{{ paper.authors }}</span>
          </div>
          <span :class="['status-badge', `status-${paper.reading_status}`]">
            {{ paper.reading_status === 'unread' ? '未读' : paper.reading_status === 'reading' ? '在读' : '已读' }}
          </span>
        </div>
        <div class="paper-actions">
          <button v-if="paper.pdf_path" class="btn btn-sm btn-secondary" @click="openReader(paper.id)">阅读</button>
          <button class="btn btn-sm btn-ghost" @click="removePaper(paper.id)">移除</button>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <p>暂无论文，点击"添加论文"开始构建文集</p>
    </div>

    <!-- AI Results section -->
    <div v-if="collectionStore.aiResults.length" class="ai-results-section">
      <h2 class="section-title">AI 分析结果</h2>
      <div class="ai-results-list">
        <div v-for="result in collectionStore.aiResults" :key="result.id" class="ai-result-card" @click="viewAiResult(result.id)">
          <span class="result-type-badge" :class="result.result_type">
            {{ result.result_type === 'review' ? '综述' : result.result_type === 'comparison' ? '对比' : '基线' }}
          </span>
          <span class="result-title">{{ result.title }}</span>
          <span class="result-date">{{ result.created_at.slice(0, 10) }}</span>
        </div>
      </div>
    </div>

    <!-- AI Dialog -->
    <div v-if="showAiDialog" class="dialog-overlay" @click.self="showAiDialog = false">
      <div class="dialog">
        <h3>AI 分析</h3>
        <div class="ai-options">
          <label :class="{ active: aiDialogType === 'review' }" @click="aiDialogType = 'review'">
            <span class="option-icon">📝</span>
            <strong>文献综述</strong>
            <small>生成结构化文献综述，便于写引言</small>
          </label>
          <label :class="{ active: aiDialogType === 'comparison' }" @click="aiDialogType = 'comparison'">
            <span class="option-icon">📊</span>
            <strong>对比研究</strong>
            <small>从方法、数据集等维度对比论文</small>
          </label>
          <label :class="{ active: aiDialogType === 'baseline' }" @click="aiDialogType = 'baseline'">
            <span class="option-icon">📈</span>
            <strong>基线对比表</strong>
            <small>提取关键实验指标构建对比表</small>
          </label>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-ghost" @click="showAiDialog = false">取消</button>
          <button class="btn btn-primary" :disabled="generating" @click="generateAi">
            {{ generating ? '生成中...' : '开始分析' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Add Papers Dialog -->
    <div v-if="showAddPapers" class="dialog-overlay" @click.self="showAddPapers = false">
      <div class="dialog wide">
        <h3>添加论文到文集</h3>
        <input v-model="searchQuery" placeholder="搜索论文..." class="search-input" />
        <div class="paper-select-list">
          <label v-for="paper in filteredLibraryPapers" :key="paper.id" class="paper-select-item">
            <input type="checkbox" :checked="selectedPapersToAdd.has(paper.id)" @change="togglePaperSelection(paper.id)" />
            <span class="paper-select-title">{{ paper.title }}</span>
            <span class="paper-select-authors">{{ paper.authors }}</span>
          </label>
          <p v-if="!filteredLibraryPapers.length" class="no-papers">没有可添加的论文</p>
        </div>
        <div class="dialog-actions">
          <span class="selection-count">已选 {{ selectedPapersToAdd.size }} 篇</span>
          <button class="btn btn-ghost" @click="showAddPapers = false">取消</button>
          <button class="btn btn-primary" :disabled="selectedPapersToAdd.size === 0" @click="addSelectedPapers">
            添加 {{ selectedPapersToAdd.size }} 篇
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.collection-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
}

.collection-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-back {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 14px;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
}

.btn-back:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.collection-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.collection-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.collection-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
}

.paper-count {
  font-size: 13px;
  color: var(--text-tertiary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.collection-desc {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 20px;
}

/* Papers list */
.papers-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 24px;
}

.paper-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  transition: background 0.15s;
}

.paper-row:hover {
  background: var(--bg-hover);
}

.paper-main {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.paper-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.paper-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.paper-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.paper-authors {
  font-size: 12px;
  color: var(--text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.paper-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.status-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 9999px;
  font-weight: 500;
  flex-shrink: 0;
}

.status-unread { background: color-mix(in srgb, var(--color-primary) 15%, transparent); color: var(--color-primary); }
.status-reading { background: color-mix(in srgb, #f59e0b 15%, transparent); color: #d97706; }
.status-read { background: color-mix(in srgb, #22c55e 15%, transparent); color: #16a34a; }

/* AI Results */
.ai-results-section {
  border-top: 1px solid var(--border-primary);
  padding-top: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.ai-results-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ai-result-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s;
}

.ai-result-card:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-sm);
}

.result-type-badge {
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 500;
}

.result-type-badge.review { background: color-mix(in srgb, #6366f1 15%, transparent); color: #6366f1; }
.result-type-badge.comparison { background: color-mix(in srgb, #f59e0b 15%, transparent); color: #d97706; }
.result-type-badge.baseline { background: color-mix(in srgb, #10b981 15%, transparent); color: #10b981; }

.result-title {
  flex: 1;
  font-size: 14px;
  color: var(--text-primary);
}

.result-date {
  font-size: 12px;
  color: var(--text-tertiary);
}

/* Dialogs */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.dialog {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: 24px;
  min-width: 400px;
  max-width: 90vw;
  max-height: 80vh;
  overflow-y: auto;
}

.dialog.wide {
  min-width: 600px;
}

.dialog h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}

.ai-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
}

.ai-options label {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border: 2px solid var(--border-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s;
}

.ai-options label:hover {
  border-color: var(--color-primary-light);
}

.ai-options label.active {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 5%, transparent);
}

.option-icon {
  font-size: 20px;
}

.ai-options label strong {
  display: block;
  font-size: 14px;
  color: var(--text-primary);
}

.ai-options label small {
  display: block;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  align-items: center;
}

.selection-count {
  flex: 1;
  font-size: 13px;
  color: var(--text-tertiary);
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-md);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
  margin-bottom: 12px;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.paper-select-list {
  max-height: 300px;
  overflow-y: auto;
}

.paper-select-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.paper-select-item:hover {
  background: var(--bg-hover);
}

.paper-select-title {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.paper-select-authors {
  font-size: 12px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.no-papers {
  text-align: center;
  color: var(--text-tertiary);
  padding: 20px;
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

.btn:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
.btn-secondary { background: var(--bg-tertiary); color: var(--text-secondary); border-color: var(--border-secondary); }
.btn-secondary:hover:not(:disabled) { background: var(--bg-active); color: var(--color-primary); }
.btn-ghost { background: transparent; color: var(--text-secondary); border-color: var(--border-secondary); }
.btn-ghost:hover:not(:disabled) { background: var(--bg-tertiary); color: var(--text-primary); }
.btn-sm { padding: 4px 12px; font-size: 12px; }
</style>
