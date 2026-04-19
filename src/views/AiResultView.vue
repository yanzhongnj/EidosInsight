<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useCollectionStore } from '../stores/collection'
import MarkdownIt from 'markdown-it'

const route = useRoute()
const router = useRouter()
const collectionStore = useCollectionStore()

const resultId = computed(() => route.params.id as string)

const md = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true,
})

onMounted(async () => {
  await collectionStore.fetchAiResult(resultId.value)
})

function goBack() {
  router.back()
}

async function deleteResult() {
  await collectionStore.deleteAiResult(resultId.value)
  router.back()
}

function typeLabel(type: string): string {
  switch (type) {
    case 'review': return '文献综述'
    case 'comparison': return '对比研究'
    case 'baseline': return '基线对比'
    default: return type
  }
}

function renderMarkdown(content: string): string {
  return md.render(content)
}

function formatJson(str: string): string {
  try {
    return JSON.stringify(JSON.parse(str), null, 2)
  } catch {
    return str
  }
}
</script>

<template>
  <div class="ai-result-view" v-if="collectionStore.currentAiResult">
    <div class="result-header">
      <button class="btn-back" @click="goBack">← 返回</button>
      <div class="result-info">
        <span class="type-badge" :class="collectionStore.currentAiResult.result_type">
          {{ typeLabel(collectionStore.currentAiResult.result_type) }}
        </span>
        <h1 class="result-title">{{ collectionStore.currentAiResult.title }}</h1>
        <span class="result-meta">
          {{ collectionStore.currentAiResult.created_at.slice(0, 10) }}
          <span v-if="collectionStore.currentAiResult.model_used"> | 模型: {{ collectionStore.currentAiResult.model_used }}</span>
        </span>
      </div>
      <button class="btn btn-ghost btn-sm" @click="deleteResult">删除</button>
    </div>

    <div class="result-content">
      <div class="markdown-content" v-html="renderMarkdown(collectionStore.currentAiResult.content)"></div>
    </div>

    <!-- Structured data for baseline -->
    <div v-if="collectionStore.currentAiResult.structured_data" class="structured-section">
      <h3>结构化数据</h3>
      <pre class="structured-json">{{ formatJson(collectionStore.currentAiResult.structured_data) }}</pre>
    </div>
  </div>

  <div v-else class="loading-state">
    <div class="spinner"></div>
    <p>加载中...</p>
  </div>
</template>

<style scoped>
.ai-result-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
}

.result-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 24px;
  flex-wrap: wrap;
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

.result-info {
  flex: 1;
  min-width: 0;
}

.type-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 500;
  margin-bottom: 6px;
}

.type-badge.review { background: color-mix(in srgb, #6366f1 15%, transparent); color: #6366f1; }
.type-badge.comparison { background: color-mix(in srgb, #f59e0b 15%, transparent); color: #d97706; }
.type-badge.baseline { background: color-mix(in srgb, #10b981 15%, transparent); color: #10b981; }

.result-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
}

.result-meta {
  font-size: 13px;
  color: var(--text-tertiary);
}

.result-content {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: 24px;
  line-height: 1.7;
  font-size: 15px;
  color: var(--text-primary);
}

.markdown-content :deep(h1) { font-size: 20px; font-weight: 700; margin: 20px 0 10px; }
.markdown-content :deep(h2) { font-size: 18px; font-weight: 600; margin: 16px 0 8px; }
.markdown-content :deep(h3) { font-size: 16px; font-weight: 600; margin: 12px 0 6px; }
.markdown-content :deep(strong) { font-weight: 600; }
.markdown-content :deep(code) { background: var(--bg-tertiary); padding: 1px 4px; border-radius: 3px; font-size: 13px; }
.markdown-content :deep(pre) { background: var(--bg-tertiary); padding: 12px; border-radius: var(--radius-md); overflow-x: auto; margin: 8px 0; }
.markdown-content :deep(pre code) { background: none; padding: 0; }
.markdown-content :deep(ul), .markdown-content :deep(ol) { margin-left: 20px; margin-bottom: 8px; }
.markdown-content :deep(li) { margin-bottom: 2px; }
.markdown-content :deep(p) { margin-bottom: 8px; }
.markdown-content :deep(table) { width: 100%; border-collapse: collapse; margin: 12px 0; font-size: 14px; }
.markdown-content :deep(th) { background: var(--bg-tertiary); font-weight: 600; text-align: left; padding: 8px 12px; border: 1px solid var(--border-secondary); }
.markdown-content :deep(td) { padding: 8px 12px; border: 1px solid var(--border-secondary); }
.markdown-content :deep(tr:hover) { background: var(--bg-hover); }
.markdown-content :deep(blockquote) { border-left: 3px solid var(--color-primary); margin: 8px 0; padding: 4px 12px; color: var(--text-secondary); }

.structured-section {
  margin-top: 24px;
  border-top: 1px solid var(--border-primary);
  padding-top: 16px;
}

.structured-section h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 8px;
}

.structured-json {
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 16px;
  font-size: 12px;
  overflow-x: auto;
  color: var(--text-secondary);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  color: var(--text-tertiary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin { to { transform: rotate(360deg); } }

.btn {
  padding: 4px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn-ghost { background: transparent; color: var(--text-secondary); border-color: var(--border-secondary); }
.btn-ghost:hover { background: var(--bg-tertiary); color: var(--text-primary); }
.btn-sm { padding: 4px 12px; font-size: 12px; }
</style>
