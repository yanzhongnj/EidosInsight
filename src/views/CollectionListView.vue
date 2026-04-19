<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useCollectionStore } from '../stores/collection'

const router = useRouter()
const collectionStore = useCollectionStore()

const showCreateDialog = ref(false)
const newCollectionName = ref('')
const newCollectionDesc = ref('')
const newCollectionColor = ref('#6366f1')
const creating = ref(false)

const presetColors = [
  '#6366f1', '#8b5cf6', '#ec4899', '#ef4444',
  '#f59e0b', '#10b981', '#06b6d4', '#3b82f6',
]

onMounted(async () => {
  await collectionStore.fetchCollections()
})

function openCollection(id: string) {
  router.push(`/collection/${id}`)
}

function openCreateDialog() {
  newCollectionName.value = ''
  newCollectionDesc.value = ''
  newCollectionColor.value = '#6366f1'
  showCreateDialog.value = true
}

async function createCollection() {
  if (!newCollectionName.value.trim()) return
  creating.value = true
  try {
    await collectionStore.createCollection({
      name: newCollectionName.value.trim(),
      description: newCollectionDesc.value.trim() || null,
      color: newCollectionColor.value,
    })
    showCreateDialog.value = false
  } catch (e) {
    console.error('Failed to create collection:', e)
  } finally {
    creating.value = false
  }
}

async function deleteCollection(id: string) {
  try {
    await collectionStore.deleteCollection(id)
  } catch (e) {
    console.error('Failed to delete collection:', e)
  }
}

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' })
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div class="collection-list-view">
    <div class="page-header">
      <h1 class="page-title">论文集</h1>
      <button class="btn btn-primary" @click="openCreateDialog">+ 新建论文集</button>
    </div>

    <!-- Loading state -->
    <div v-if="collectionStore.loading" class="loading-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- Empty state -->
    <div v-else-if="collectionStore.collections.length === 0" class="empty-state">
      <div class="empty-icon">📁</div>
      <h2>还没有论文集</h2>
      <p>创建论文集来组织相关论文，并使用 AI 进行文献综述和对比分析</p>
      <button class="btn btn-primary" @click="openCreateDialog">+ 新建论文集</button>
    </div>

    <!-- Collection grid -->
    <div v-else class="collections-grid">
      <div
        v-for="collection in collectionStore.collections"
        :key="collection.id"
        class="collection-card"
        @click="openCollection(collection.id)"
      >
        <div class="card-color-bar" :style="{ backgroundColor: collection.color || '#6366f1' }"></div>
        <div class="card-body">
          <div class="card-top">
            <span class="collection-icon">{{ collection.icon || '📁' }}</span>
            <h3 class="collection-name">{{ collection.name }}</h3>
          </div>
          <p v-if="collection.description" class="collection-desc">{{ collection.description }}</p>
          <div class="card-footer">
            <span class="paper-count">{{ collection.paper_count ?? 0 }} 篇论文</span>
            <span class="card-date">{{ formatDate(collection.updated_at) }}</span>
          </div>
          <div class="card-actions-row">
            <button class="btn btn-ghost btn-sm" @click.stop="openCollection(collection.id)">打开</button>
            <button class="btn btn-ghost btn-sm btn-danger" @click.stop="deleteCollection(collection.id)">删除</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create dialog -->
    <div v-if="showCreateDialog" class="dialog-overlay" @click.self="showCreateDialog = false">
      <div class="dialog-content">
        <h2 class="dialog-title">新建论文集</h2>

        <div class="form-group">
          <label class="form-label">名称 *</label>
          <input
            v-model="newCollectionName"
            class="form-input"
            placeholder="例如：Transformer 相关论文"
            @keydown.enter="createCollection"
          />
        </div>

        <div class="form-group">
          <label class="form-label">描述</label>
          <textarea
            v-model="newCollectionDesc"
            class="form-textarea"
            placeholder="可选的描述信息"
            rows="3"
          ></textarea>
        </div>

        <div class="form-group">
          <label class="form-label">颜色</label>
          <div class="color-picker">
            <button
              v-for="color in presetColors"
              :key="color"
              class="color-swatch"
              :class="{ active: newCollectionColor === color }"
              :style="{ backgroundColor: color }"
              @click="newCollectionColor = color"
            ></button>
          </div>
        </div>

        <div class="dialog-actions">
          <button class="btn btn-ghost" @click="showCreateDialog = false">取消</button>
          <button class="btn btn-primary" :disabled="!newCollectionName.trim() || creating" @click="createCollection">
            {{ creating ? '创建中...' : '创建' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.collection-list-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
}

.collections-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
}

.collection-card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all 0.15s;
  overflow: hidden;
}

.collection-card:hover {
  box-shadow: var(--shadow-md);
  border-color: var(--color-primary);
}

.card-color-bar {
  height: 4px;
}

.card-body {
  padding: 16px;
}

.card-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.collection-icon {
  font-size: 18px;
}

.collection-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.collection-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 12px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.paper-count {
  font-size: 12px;
  color: var(--text-tertiary);
}

.card-date {
  font-size: 12px;
  color: var(--text-tertiary);
}

.card-actions-row {
  display: flex;
  gap: 4px;
}

/* States */
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
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

/* Dialog */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog-content {
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  padding: 24px;
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
}

.dialog-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.form-input,
.form-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-md);
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}

.form-input:focus,
.form-textarea:focus {
  border-color: var(--color-primary);
}

.form-textarea {
  resize: vertical;
}

.color-picker {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.color-swatch:hover {
  transform: scale(1.15);
}

.color-swatch.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px var(--bg-primary);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
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

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border-color: var(--border-secondary);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn-sm {
  padding: 4px 12px;
  font-size: 12px;
}

.btn-danger:hover {
  color: var(--color-danger, #ef4444) !important;
}
</style>
