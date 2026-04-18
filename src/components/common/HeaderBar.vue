<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { usePaperStore } from '../../stores/paper'
import { useAppStore } from '../../stores/app'
import ImportDialog from '../paper/ImportDialog.vue'

const router = useRouter()
const paperStore = usePaperStore()
const appStore = useAppStore()
const searchQuery = ref('')
const showImportDialog = ref(false)

const llmStatusColor = computed(() => {
  switch (appStore.llmStatus.status) {
    case 'local_available': return 'var(--color-ai-local, #22C55E)'
    case 'cloud_available': return 'var(--color-ai-cloud, #3B82F6)'
    default: return 'var(--color-ai-off, #9CA3AF)'
  }
})

const llmStatusLabel = computed(() => {
  switch (appStore.llmStatus.status) {
    case 'local_available': return '本地 AI 可用'
    case 'cloud_available': return '云端 AI 可用'
    default: return 'AI 未配置'
  }
})

async function handleSearch() {
  if (!searchQuery.value.trim()) return
  try {
    await paperStore.fetchPapers({ query: searchQuery.value })
  } catch (error) {
    console.error('Search failed:', error)
  }
}

function clearSearch() {
  searchQuery.value = ''
  paperStore.fetchPapers({})
}

function openImport() {
  showImportDialog.value = true
}

function closeImport() {
  showImportDialog.value = false
}

function goToSettings() {
  router.push('/settings')
}
</script>

<template>
  <header class="header-bar">
    <div class="search-box">
      <span class="search-icon">&#x1F50D;</span>
      <input
        v-model="searchQuery"
        type="text"
        class="input search-input"
        placeholder="搜索论文、作者、关键词..."
        @keyup.enter="handleSearch"
      />
      <button v-if="searchQuery" class="btn-ghost clear-btn" @click="clearSearch">&#x2715;</button>
    </div>
    <div class="header-actions">
      <!-- LLM Status indicator -->
      <button class="llm-status-btn" @click="goToSettings" :title="llmStatusLabel">
        <span class="status-dot" :style="{ backgroundColor: llmStatusColor }"></span>
        <span class="status-text">{{ llmStatusLabel }}</span>
      </button>
      <button class="btn btn-primary" @click="openImport">
        + 导入文献
      </button>
    </div>

    <ImportDialog v-if="showImportDialog" @close="closeImport" />
  </header>
</template>

<style scoped>
.header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-primary);
  gap: 16px;
  height: var(--header-height);
}

.search-box {
  flex: 1;
  max-width: 600px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-lg);
  padding: 4px 12px;
}

.search-icon {
  font-size: 14px;
  opacity: 0.6;
}

.search-input {
  border: none;
  background: transparent;
  box-shadow: none;
  padding: 6px 4px;
}

.search-input:focus {
  border: none;
  outline: none;
}

.clear-btn {
  padding: 2px 6px;
  font-size: 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--text-tertiary);
}

.clear-btn:hover {
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.llm-status-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
  transition: border-color 0.2s;
}

.llm-status-btn:hover {
  border-color: var(--color-info);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-text {
  white-space: nowrap;
}
</style>
