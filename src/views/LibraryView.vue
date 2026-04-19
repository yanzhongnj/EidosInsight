<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { usePaperStore } from '../stores/paper'
import { useAppStore } from '../stores/app'
import { useTauriCommand } from '../composables/useTauriCommand'
import PaperCard from '../components/paper/PaperCard.vue'
import ImportDialog from '../components/paper/ImportDialog.vue'
import type { PaperFilter } from '../types'

const router = useRouter()
const paperStore = usePaperStore()
const appStore = useAppStore()
const { getExtensionStatus, registerNativeHost, addPaperTag, removePaperTag } = useTauriCommand()

const viewMode = ref<'grid' | 'list'>('grid')
const showImportDialog = ref(false)
const dragOver = ref(false)

// Tag filter state
const selectedTagFilter = ref<string | null>(null)

// Browser extension banner state
const extensionRegistered = ref<boolean | null>(null) // null = loading
const showExtensionBanner = ref(false)
const registeringHost = ref(false)
const registerError = ref<string | null>(null)

// Filter & Sort state
const currentSort = ref<'date_added' | 'title' | 'date_modified'>('date_added')
const currentSortOrder = ref<'desc' | 'asc'>('desc')
const currentStatusFilter = ref<'all' | 'unread' | 'reading' | 'read'>('all')
const showFavoritesOnly = ref(false)
const currentPage = ref(0)
const pageSize = 50

const currentFilter = computed<PaperFilter>(() => ({
  sort_by: currentSort.value,
  sort_order: currentSortOrder.value,
  reading_status: currentStatusFilter.value === 'all' ? undefined : currentStatusFilter.value,
  is_favorite: showFavoritesOnly.value ? true : undefined,
  folder_id: appStore.selectedFolderId ?? undefined,
  tag: selectedTagFilter.value ?? undefined,
  limit: pageSize,
  offset: currentPage.value * pageSize,
}))

const totalPages = computed(() => Math.ceil(paperStore.totalPapers / pageSize))

// Watch filter changes and reload
watch([currentSort, currentSortOrder, currentStatusFilter, showFavoritesOnly, selectedTagFilter], () => {
  currentPage.value = 0
  paperStore.fetchPapers(currentFilter.value, true)
})

watch(currentPage, () => {
  paperStore.fetchPapers(currentFilter.value, true)
})

// Watch for folder selection changes from Sidebar
watch(() => appStore.selectedFolderId, (newFolderId) => {
  if (router.currentRoute.value.path === '/library') {
    currentPage.value = 0
    paperStore.fetchPapers({
      folder_id: newFolderId ?? undefined,
    }, true)
  }
})

onMounted(async () => {
  // Sync UI state from appStore.activeView
  const activeView = appStore.activeView
  if (activeView === 'unread') {
    currentStatusFilter.value = 'unread'
  } else if (activeView === 'read') {
    currentStatusFilter.value = 'read'
  } else if (activeView === 'favorites') {
    showFavoritesOnly.value = true
  }
  // If folder is selected, respect it
  // (folder_id is already in currentFilter via appStore.selectedFolderId)

  // Fetch papers using the computed filter (which now reflects appStore state)
  paperStore.fetchPapers(currentFilter.value, true)

  // Check extension status (non-blocking)
  try {
    const status = await getExtensionStatus()
    extensionRegistered.value = status.registered
    // Show banner only if not yet registered, and only once per session
    if (!status.registered) {
      const dismissed = sessionStorage.getItem('extensionBannerDismissed')
      if (!dismissed) showExtensionBanner.value = true
    }
  } catch {
    extensionRegistered.value = null
  }
})

// Delete confirmation state
const deleteTarget = ref<string | null>(null)

function openPaper(id: string) {
  router.push(`/paper/${id}`)
}

function openReader(id: string) {
  router.push(`/reader/${id}`)
}

function confirmDelete(paperId: string) {
  deleteTarget.value = paperId
}

async function doDelete() {
  if (!deleteTarget.value) return
  try {
    await paperStore.deletePaper(deleteTarget.value)
  } catch (e) {
    console.error('Delete failed:', e)
  } finally {
    deleteTarget.value = null
  }
}

function cancelDelete() {
  deleteTarget.value = null
}

async function handleMoveToFolder(paperId: string, folderId: string | null) {
  try {
    await paperStore.movePaperToFolder(paperId, folderId)
    // Refresh list to reflect the move
    paperStore.fetchPapers(currentFilter.value, true)
  } catch (e) {
    console.error('Move failed:', e)
  }
}

async function handleSetStatus(paperId: string, status: 'unread' | 'reading' | 'read') {
  try {
    await paperStore.setReadingStatus(paperId, status)
  } catch (e) {
    console.error('Set status failed:', e)
  }
}

function openImport() {
  showImportDialog.value = true
}

function closeImport() {
  showImportDialog.value = false
  // Refresh list after import
  paperStore.fetchPapers(currentFilter.value, true)
}

function setSort(sort: 'date_added' | 'title' | 'date_modified') {
  if (currentSort.value === sort) {
    currentSortOrder.value = currentSortOrder.value === 'desc' ? 'asc' : 'desc'
  } else {
    currentSort.value = sort
    currentSortOrder.value = 'desc'
  }
}

function setStatusFilter(status: 'all' | 'unread' | 'reading' | 'read') {
  currentStatusFilter.value = status
}

function toggleFavoritesOnly() {
  showFavoritesOnly.value = !showFavoritesOnly.value
}

function setTagFilter(tagName: string | null) {
  selectedTagFilter.value = tagName
}

const popularTags = computed(() => {
  return [...appStore.tags]
    .sort((a, b) => b.usage_count - a.usage_count)
    .slice(0, 10)
})

function prevPage() {
  if (currentPage.value > 0) currentPage.value--
}

function nextPage() {
  if (currentPage.value < totalPages.value - 1) currentPage.value++
}

function handleDrop(event: DragEvent) {
  event.preventDefault()
  dragOver.value = false
  if (!event.dataTransfer?.files.length) return
  showImportDialog.value = true
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer?.types.includes('Files')) {
    dragOver.value = true
  }
}

function handleDragLeave() {
  dragOver.value = false
}

function dismissExtensionBanner() {
  showExtensionBanner.value = false
  sessionStorage.setItem('extensionBannerDismissed', '1')
}

async function doRegisterHost() {
  registeringHost.value = true
  registerError.value = null
  try {
    await registerNativeHost()
    extensionRegistered.value = true
    showExtensionBanner.value = false
  } catch (err: unknown) {
    registerError.value = err instanceof Error ? err.message : String(err)
  } finally {
    registeringHost.value = false
  }
}

async function handleAddTag(paperId: string, tagName: string) {
  try {
    await addPaperTag(paperId, tagName)
    // Refresh paper tags and global tag list
    await paperStore.loadPaperTags([paperId])
    await appStore.fetchTags()
  } catch (e) {
    console.error('Add tag failed:', e)
  }
}

async function handleRemoveTag(paperId: string, tagId: string) {
  try {
    await removePaperTag(paperId, tagId)
    // Refresh paper tags and global tag list
    await paperStore.loadPaperTags([paperId])
    await appStore.fetchTags()
  } catch (e) {
    console.error('Remove tag failed:', e)
  }
}
</script>

<template>
  <div
    class="library-view"
    :class="{ 'drag-over': dragOver }"
    @drop="handleDrop"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
  >
    <div class="library-header">
      <h1 class="page-title">文献库</h1>
      <div class="header-right">
        <button class="btn btn-primary" @click="openImport">
          + 导入文献
        </button>
      </div>
    </div>

    <!-- Browser extension banner (shown when native host not registered) -->
    <div v-if="showExtensionBanner" class="extension-banner">
      <div class="banner-content">
        <span class="banner-icon">🧩</span>
        <div class="banner-text">
          <strong>安装 Chrome 扩展</strong> — 在 arXiv、ACM、IEEE 等网站一键保存论文到 EidosInsight
        </div>
        <div class="banner-actions">
          <button
            class="btn-banner-install"
            :disabled="registeringHost"
            @click="doRegisterHost"
          >
            {{ registeringHost ? '安装中...' : '一键安装连接组件' }}
          </button>
          <button class="btn-banner-dismiss" @click="dismissExtensionBanner">✕</button>
        </div>
      </div>
      <p v-if="registerError" class="banner-error">{{ registerError }}</p>
    </div>

    <!-- Filter bar -->
    <div class="filter-bar">
      <div class="filter-group">
        <button
          class="filter-btn"
          :class="{ active: currentStatusFilter === 'all' }"
          @click="setStatusFilter('all')"
        >
          全部
        </button>
        <button
          class="filter-btn"
          :class="{ active: currentStatusFilter === 'unread' }"
          @click="setStatusFilter('unread')"
        >
          未读
        </button>
        <button
          class="filter-btn"
          :class="{ active: currentStatusFilter === 'reading' }"
          @click="setStatusFilter('reading')"
        >
          在读
        </button>
        <button
          class="filter-btn"
          :class="{ active: currentStatusFilter === 'read' }"
          @click="setStatusFilter('read')"
        >
          已读
        </button>
        <span class="filter-separator">|</span>
        <button
          class="filter-btn"
          :class="{ active: showFavoritesOnly }"
          @click="toggleFavoritesOnly"
        >
          ★ 收藏
        </button>
        <span v-if="popularTags.length > 0" class="filter-separator">|</span>
        <div v-if="popularTags.length > 0" class="tag-filter-group">
          <button
            class="filter-btn tag-filter-btn"
            :class="{ active: !selectedTagFilter }"
            @click="setTagFilter(null)"
          >
            全部标签
          </button>
          <button
            v-for="tag in popularTags.slice(0, 5)"
            :key="tag.id"
            class="filter-btn tag-filter-btn"
            :class="{ active: selectedTagFilter === tag.name }"
            :style="selectedTagFilter === tag.name && tag.color ? { backgroundColor: `${tag.color}18`, color: tag.color, borderColor: `${tag.color}40` } : {}"
            @click="setTagFilter(tag.name)"
          >
            {{ tag.name }}
          </button>
        </div>
      </div>

      <div class="sort-group">
        <span class="sort-label">排序:</span>
        <button
          class="sort-btn"
          :class="{ active: currentSort === 'date_added' }"
          @click="setSort('date_added')"
        >
          添加时间
          <span v-if="currentSort === 'date_added'">{{ currentSortOrder === 'desc' ? '↓' : '↑' }}</span>
        </button>
        <button
          class="sort-btn"
          :class="{ active: currentSort === 'date_modified' }"
          @click="setSort('date_modified')"
        >
          修改时间
          <span v-if="currentSort === 'date_modified'">{{ currentSortOrder === 'desc' ? '↓' : '↑' }}</span>
        </button>
        <button
          class="sort-btn"
          :class="{ active: currentSort === 'title' }"
          @click="setSort('title')"
        >
          标题
          <span v-if="currentSort === 'title'">{{ currentSortOrder === 'desc' ? '↓' : '↑' }}</span>
        </button>

        <span class="filter-separator">|</span>

        <div class="view-toggle">
          <button
            class="view-btn"
            :class="{ active: viewMode === 'grid' }"
            @click="viewMode = 'grid'"
            title="网格视图"
          >
            ▦
          </button>
          <button
            class="view-btn"
            :class="{ active: viewMode === 'list' }"
            @click="viewMode = 'list'"
            title="列表视图"
          >
            ☰
          </button>
        </div>
      </div>
    </div>

    <!-- Drag overlay -->
    <div v-if="dragOver" class="drag-overlay">
      <div class="drag-overlay-content">
        <div class="drag-icon">📄</div>
        <p>松开鼠标导入 PDF 文件</p>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="paperStore.loading" class="loading-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- Empty state -->
    <div v-else-if="paperStore.papers.length === 0" class="empty-state">
      <div class="empty-icon">📚</div>
      <h2>{{ showFavoritesOnly || currentStatusFilter !== 'all' ? '没有匹配的文献' : '文献库为空' }}</h2>
      <p v-if="!showFavoritesOnly && currentStatusFilter === 'all'">
        导入你的第一篇论文，开始构建你的知识库
      </p>
      <p v-else>尝试调整筛选条件</p>
      <button
        v-if="!showFavoritesOnly && currentStatusFilter === 'all'"
        class="btn btn-primary"
        @click="openImport"
      >
        + 导入文献
      </button>
    </div>

    <!-- Paper grid/list -->
    <div v-else :class="['papers-container', viewMode]">
      <PaperCard
        v-for="paper in paperStore.papers"
        :key="paper.id"
        :paper="paper"
        :view-mode="viewMode"
        :tags="paperStore.getPaperTags(paper.id)"
        @click="openPaper(paper.id)"
        @read="openReader(paper.id)"
        @toggle-favorite="paperStore.toggleFavorite(paper)"
        @delete="confirmDelete(paper.id)"
        @move-to-folder="(folderId) => handleMoveToFolder(paper.id, folderId)"
        @set-status="(status) => handleSetStatus(paper.id, status)"
        @add-tag="(tagName) => handleAddTag(paper.id, tagName)"
        @remove-tag="(tagId) => handleRemoveTag(paper.id, tagId)"
      />
    </div>

    <!-- Pagination -->
    <div v-if="paperStore.papers.length > 0" class="pagination">
      <span class="pagination-info">
        显示 {{ currentPage * pageSize + 1 }}-{{ Math.min((currentPage + 1) * pageSize, paperStore.totalPapers) }}
        / {{ paperStore.totalPapers }} 篇文献
      </span>
      <div class="pagination-controls">
        <button
          class="btn btn-ghost btn-sm"
          :disabled="currentPage === 0"
          @click="prevPage"
        >
          上一页
        </button>
        <span class="page-indicator">{{ currentPage + 1 }} / {{ totalPages }}</span>
        <button
          class="btn btn-ghost btn-sm"
          :disabled="currentPage >= totalPages - 1"
          @click="nextPage"
        >
          下一页
        </button>
      </div>
    </div>

    <!-- Delete confirmation dialog -->
    <div v-if="deleteTarget" class="modal-overlay" @click.self="cancelDelete">
      <div class="modal-content">
        <h3>确认删除</h3>
        <p>确定要删除这篇文献吗？此操作不可恢复。</p>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="cancelDelete">取消</button>
          <button class="btn btn-danger" @click="doDelete">确认删除</button>
        </div>
      </div>
    </div>

    <!-- Import Dialog -->
    <ImportDialog v-if="showImportDialog" @close="closeImport" />
  </div>
</template>

<style scoped>
.library-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
  position: relative;
  min-height: 100%;
}

.library-view.drag-over {
  outline: 3px dashed var(--color-primary);
  outline-offset: -3px;
  border-radius: var(--radius-lg);
}

.drag-overlay {
  position: absolute;
  inset: 0;
  background: color-mix(in srgb, var(--color-primary) 5%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  backdrop-filter: blur(2px);
}

.drag-overlay-content {
  text-align: center;
  color: var(--color-primary);
}

.drag-overlay-content .drag-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.drag-overlay-content p {
  font-size: 18px;
  font-weight: 500;
}

.library-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Browser extension banner */
.extension-banner {
  background: color-mix(in srgb, var(--color-primary) 8%, var(--bg-secondary));
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
  border-radius: var(--radius-lg);
  padding: 10px 14px;
  margin-bottom: 16px;
}

.banner-content {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.banner-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.banner-text {
  flex: 1;
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 200px;
}

.banner-text strong {
  color: var(--text-primary);
}

.banner-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.btn-banner-install {
  padding: 5px 14px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: filter 0.15s;
  white-space: nowrap;
}

.btn-banner-install:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-banner-install:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-banner-dismiss {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  transition: color 0.15s, background 0.15s;
}

.btn-banner-dismiss:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.banner-error {
  font-size: 12px;
  color: var(--color-error, #f87171);
  margin-top: 6px;
}

/* Filter bar */
.filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.tag-filter-group {
  display: flex;
  align-items: center;
  gap: 3px;
  flex-wrap: wrap;
}

.tag-filter-btn {
  font-size: 12px;
  padding: 3px 8px;
}

.filter-btn {
  padding: 5px 12px;
  border: 1px solid transparent;
  background: transparent;
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.filter-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.filter-btn.active {
  background: color-mix(in srgb, var(--color-primary) 15%, transparent);
  color: var(--color-primary);
  border-color: color-mix(in srgb, var(--color-primary) 30%, transparent);
}

.filter-separator {
  color: var(--border-secondary);
  margin: 0 4px;
}

.sort-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.sort-label {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-right: 4px;
}

.sort-btn {
  padding: 5px 10px;
  border: 1px solid transparent;
  background: transparent;
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.sort-btn:hover {
  background: var(--bg-tertiary);
}

.sort-btn.active {
  color: var(--color-primary);
  font-weight: 500;
}

.view-toggle {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 2px;
}

.view-btn {
  padding: 4px 8px;
  border: none;
  background: transparent;
  border-radius: 4px;
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.view-btn.active {
  background: var(--bg-primary);
  color: var(--color-primary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}

/* Paper list */
.papers-container.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.papers-container.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
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

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Pagination */
.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  margin-top: 8px;
  border-top: 1px solid var(--border-primary);
}

.pagination-info {
  font-size: 13px;
  color: var(--text-tertiary);
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-indicator {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 60px;
  text-align: center;
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

.btn-danger {
  background: #ef4444;
  color: white;
  border-color: #ef4444;
}

.btn-danger:hover:not(:disabled) {
  filter: brightness(1.1);
}

/* Modal overlay */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  backdrop-filter: blur(2px);
}

.modal-content {
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  padding: 24px;
  max-width: 400px;
  width: 90%;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
}

.modal-content h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.modal-content p {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 20px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
