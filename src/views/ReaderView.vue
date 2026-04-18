<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { usePaperStore } from '../stores/paper'
import { useReaderStore } from '../stores/reader'
import { useNoteStore } from '../stores/note'
import PdfViewer from '../components/reader/PdfViewer.vue'
import NoteList from '../components/note/NoteList.vue'
import NoteEditor from '../components/note/NoteEditor.vue'
import AiAssistPanel from '../components/ai/AiAssistPanel.vue'
import type { Note, CreateNoteInput, UpdateNoteInput } from '../types'

const route = useRoute()
const router = useRouter()
const paperStore = usePaperStore()
const readerStore = useReaderStore()
const noteStore = useNoteStore()

const pdfData = ref('')
const loading = ref(true)
const error = ref('')
const viewerRef = ref<InstanceType<typeof PdfViewer> | null>(null)
const showPageInput = ref(false)
const pageInputValue = ref('')

// Note sidebar state
const notesOpen = ref(false)
const noteFilter = ref<'all' | 'page' | 'annotation' | 'general'>('all')
const editingNote = ref(false)
const editingNoteData = ref<Note | null>(null)
const creatingNote = ref(false)

// AI panel state
const aiPanelOpen = ref(false)
function toggleAiPanel() {
  aiPanelOpen.value = !aiPanelOpen.value
  if (aiPanelOpen.value && notesOpen.value) {
    // Close notes panel when AI panel opens (avoid crowding)
    notesOpen.value = false
  }
}

const paper = computed(() => paperStore.currentPaper)
const paperId = computed(() => route.params.id as string)
const readingProgress = computed(() => {
  if (readerStore.totalPages === 0) return 0
  return Math.round((readerStore.currentPage / readerStore.totalPages) * 100)
})

const notesForPaper = computed(() => noteStore.notes)

// Watch for page changes to update filter
watch(() => readerStore.currentPage, () => {
  if (noteFilter.value === 'page') {
    // The filter re-computes automatically via NoteList computed property
  }
})

onMounted(async () => {
  const id = paperId.value
  try {
    await paperStore.fetchPaper(id)
    await readerStore.loadReadingState(id)
    await noteStore.fetchNotes(id)

    if (paperStore.currentPaper?.pdf_path) {
      loading.value = true
      pdfData.value = await paperStore.readPdfFile(id)

      if (paperStore.currentPaper.reading_status === 'unread') {
        paperStore.setReadingStatus(id, 'reading')
      }
    } else {
      error.value = '该文献没有关联的 PDF 文件'
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '加载 PDF 失败'
    console.error('Failed to load reader:', e)
  } finally {
    loading.value = false
  }
})

onBeforeUnmount(() => {
  if (paperId.value && readerStore.totalPages > 0) {
    readerStore.saveReadingState(paperId.value)
  }
})

// PDF event handlers
function onPageChange(page: number, total: number) {
  readerStore.currentPage = page
  readerStore.totalPages = total
  readerStore.debouncedSave(paperId.value)
}

function onZoomChange(zoom: number) {
  readerStore.zoomLevel = zoom
  readerStore.debouncedSave(paperId.value)
}

function onDocumentLoaded(totalPages: number) {
  readerStore.totalPages = totalPages
  if (readerStore.readingState?.current_page && readerStore.readingState.current_page > 1) {
    viewerRef.value?.goToPage(readerStore.readingState.current_page)
  } else {
    viewerRef.value?.fitWidth()
  }
}

// Navigation
function goToPage() {
  const page = parseInt(pageInputValue.value, 10)
  if (page >= 1 && page <= readerStore.totalPages) {
    viewerRef.value?.goToPage(page)
  }
  showPageInput.value = false
  pageInputValue.value = ''
}

function showPageInputDialog() {
  showPageInput.value = true
  pageInputValue.value = String(readerStore.currentPage)
}

function markAsRead() {
  paperStore.setReadingStatus(paperId.value, 'read')
}

function toggleFavorite() {
  if (paper.value) {
    paperStore.toggleFavorite(paper.value)
  }
}

// Note sidebar
function toggleNotes() {
  notesOpen.value = !notesOpen.value
}

function startCreateNote() {
  editingNoteData.value = null
  creatingNote.value = true
  editingNote.value = true
}

function handleNoteSave(input: CreateNoteInput | UpdateNoteInput) {
  if (creatingNote.value) {
    noteStore.createNote(input as CreateNoteInput).then(() => {
      editingNote.value = false
      creatingNote.value = false
    })
  } else if (editingNoteData.value) {
    noteStore.updateNote(input as UpdateNoteInput).then(() => {
      editingNote.value = false
      editingNoteData.value = null
    })
  }
}

function handleNoteEdit(note: Note) {
  editingNoteData.value = note
  creatingNote.value = false
  editingNote.value = true
}

function handleNoteDelete(id: string) {
  noteStore.deleteNote(id)
}

function handleNoteSelect(note: Note) {
  // If the note has a page number, jump to that page
  if (note.page_number && note.page_number !== readerStore.currentPage) {
    viewerRef.value?.goToPage(note.page_number)
  }
}

function handleGoToPage(page: number) {
  viewerRef.value?.goToPage(page)
}

function cancelEdit() {
  editingNote.value = false
  creatingNote.value = false
  editingNoteData.value = null
}
</script>

<template>
  <div class="reader-view" :class="{ 'notes-open': notesOpen }">
    <!-- Top toolbar -->
    <div class="reader-toolbar">
      <div class="toolbar-left">
        <button class="btn-ghost" @click="router.back()" title="返回文献库">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M10 3L5 8l5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
          返回
        </button>
      </div>

      <div class="toolbar-center">
        <span class="paper-title-trunc" :title="paper?.title">
          {{ paper?.title || '加载中...' }}
        </span>
      </div>

      <div class="toolbar-right">
        <div class="toolbar-group">
          <button class="btn-ghost" @click="viewerRef?.prevPage()" :disabled="readerStore.currentPage <= 1" title="上一页">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M10 3L5 8l5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </button>
          <button class="btn-ghost page-indicator" @click="showPageInputDialog" title="跳转到页">
            {{ readerStore.currentPage }} / {{ readerStore.totalPages }}
          </button>
          <button class="btn-ghost" @click="viewerRef?.nextPage()" :disabled="readerStore.currentPage >= readerStore.totalPages" title="下一页">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M6 3l5 5-5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </button>
        </div>

        <div class="toolbar-divider" />

        <div class="toolbar-group">
          <button class="btn-ghost" @click="viewerRef?.zoomOut()" title="缩小">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          </button>
          <span class="zoom-display">{{ Math.round(readerStore.zoomLevel * 100) }}%</span>
          <button class="btn-ghost" @click="viewerRef?.zoomIn()" title="放大">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          </button>
          <button class="btn-ghost" @click="viewerRef?.fitWidth()" title="适合宽度">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M2 5v6M14 5v6M5 2h6M5 14h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </button>
        </div>

        <div class="toolbar-divider" />

        <!-- Notes toggle -->
        <div class="toolbar-group">
          <button
            class="btn-ghost"
            :class="{ active: notesOpen }"
            @click="toggleNotes"
            title="笔记面板"
          >
            📝
            <span v-if="notesForPaper.length" class="note-count">{{ notesForPaper.length }}</span>
          </button>
          <!-- AI Panel toggle -->
          <button
            class="btn-ghost ai-btn"
            :class="{ active: aiPanelOpen }"
            @click="toggleAiPanel"
            title="AI 助理"
          >
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14">
              <path fill-rule="evenodd"
                d="M9.664 1.319a.75.75 0 01.672 0 41.059 41.059 0 018.198 5.424.75.75 0 01-.254 1.285 31.372 31.372 0 00-7.86 3.83.75.75 0 01-.84 0 31.508 31.508 0 00-2.08-1.287V9.482a31.516 31.516 0 00-5.774-2.458.75.75 0 01-.254-1.285 41.059 41.059 0 018.198-5.424zM6 12.459v3.116a.75.75 0 00.894.734 31.39 31.39 0 005.332-1.76A.75.75 0 0013 13.85v-3.391a33.016 33.016 0 00-7 2z"
                clip-rule="evenodd" />
            </svg>
          </button>
          <button
            class="btn-ghost"
            :class="{ active: paper?.is_favorite }"
            @click="toggleFavorite"
            title="收藏"
          >
            {{ paper?.is_favorite ? '★' : '☆' }}
          </button>
          <button class="btn-ghost" @click="markAsRead" title="标记已读" v-if="paper?.reading_status !== 'read'">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M3 8l3.5 3.5L13 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Progress bar -->
    <div class="reading-progress-bar">
      <div class="progress-fill" :style="{ width: readingProgress + '%' }" />
    </div>

    <!-- Page jump dialog -->
    <div v-if="showPageInput" class="page-jump-overlay" @click.self="showPageInput = false">
      <div class="page-jump-dialog">
        <label>跳转到页码</label>
        <input
          v-model="pageInputValue"
          type="number"
          :min="1"
          :max="readerStore.totalPages"
          class="input"
          @keydown.enter="goToPage"
          autofocus
        />
        <div class="page-jump-actions">
          <button class="btn btn-secondary" @click="showPageInput = false">取消</button>
          <button class="btn btn-primary" @click="goToPage">跳转</button>
        </div>
      </div>
    </div>

    <!-- Main content area -->
    <div class="reader-content">
      <!-- Loading state -->
      <div v-if="loading" class="reader-placeholder">
        <div class="spinner" />
        <h3>加载 PDF 中...</h3>
        <p>正在读取文件并渲染</p>
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="reader-placeholder error">
        <div class="placeholder-icon">⚠️</div>
        <h3>无法加载 PDF</h3>
        <p>{{ error }}</p>
        <button class="btn btn-secondary" @click="router.back()">返回文献库</button>
      </div>

      <!-- No PDF -->
      <div v-else-if="!paper?.pdf_path" class="reader-placeholder">
        <div class="placeholder-icon">📭</div>
        <h3>暂无 PDF 文件</h3>
        <p>这篇文献尚未关联 PDF 文件</p>
        <button class="btn btn-secondary" @click="router.back()">返回文献库</button>
      </div>

      <!-- PDF Viewer -->
      <PdfViewer
        v-else
        ref="viewerRef"
        :pdf-data="pdfData"
        :initial-page="readerStore.currentPage"
        :initial-zoom="readerStore.zoomLevel"
        @page-change="onPageChange"
        @zoom-change="onZoomChange"
        @document-loaded="onDocumentLoaded"
      />
    </div>

    <!-- Notes sidebar -->
    <div v-if="notesOpen" class="notes-sidebar">
      <div class="sidebar-header">
        <h3>笔记</h3>
        <div class="sidebar-actions">
          <button class="btn btn-primary btn-sm" @click="startCreateNote">+ 新笔记</button>
          <button class="btn-ghost" @click="notesOpen = false" title="关闭">✕</button>
        </div>
      </div>

      <!-- Filter tabs -->
      <div class="filter-tabs">
        <button
          class="filter-tab"
          :class="{ active: noteFilter === 'all' }"
          @click="noteFilter = 'all'"
        >全部</button>
        <button
          class="filter-tab"
          :class="{ active: noteFilter === 'page' }"
          @click="noteFilter = 'page'"
        >当前页</button>
        <button
          class="filter-tab"
          :class="{ active: noteFilter === 'annotation' }"
          @click="noteFilter = 'annotation'"
        >批注</button>
      </div>

      <!-- Note editor -->
      <div v-if="editingNote" class="sidebar-editor">
        <NoteEditor
          :note="editingNoteData"
          :paper-id="paperId"
          :current-page="readerStore.currentPage"
          :mode="creatingNote ? 'create' : 'edit'"
          @save="handleNoteSave"
          @cancel="cancelEdit"
        />
      </div>

      <!-- Note list -->
      <NoteList
        v-else
        :notes="notesForPaper"
        :current-page="readerStore.currentPage"
        :filter="noteFilter"
        @select="handleNoteSelect"
        @edit="handleNoteEdit"
        @delete="handleNoteDelete"
        @go-to-page="handleGoToPage"
      />
    </div>

    <!-- AI panel sidebar -->
    <Transition name="reader-ai-slide">
      <div v-if="aiPanelOpen" class="ai-sidebar">
        <AiAssistPanel :paper="paper" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.reader-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-tertiary);
}

.reader-view.notes-open {
  /* When notes are open, the content area becomes horizontal */
}

.reader-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-primary);
  min-height: 42px;
  gap: 8px;
}

.toolbar-left, .toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-center {
  flex: 1;
  text-align: center;
  overflow: hidden;
  padding: 0 8px;
}

.paper-title-trunc {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: inline-block;
  max-width: 100%;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: var(--border-primary);
  margin: 0 4px;
}

.btn-ghost {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-ghost:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.btn-ghost.active {
  color: var(--color-accent);
}

.note-count {
  background: var(--color-primary);
  color: var(--text-inverse);
  font-size: 10px;
  font-weight: 600;
  padding: 0 4px;
  border-radius: 8px;
  min-width: 14px;
  text-align: center;
  line-height: 16px;
}

.page-indicator {
  font-size: 12px;
  font-family: var(--font-mono);
  min-width: 56px;
  justify-content: center;
  color: var(--text-secondary);
}

.zoom-display {
  font-size: 11px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  min-width: 36px;
  text-align: center;
}

.reading-progress-bar {
  height: 2px;
  background: var(--border-primary);
  position: relative;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  transition: width 0.3s ease;
  border-radius: 0 1px 1px 0;
}

.reader-content {
  flex: 1;
  overflow: hidden;
  display: flex;
}

/* Notes sidebar */
.notes-sidebar {
  width: 380px;
  min-width: 380px;
  background: var(--bg-primary);
  border-left: 1px solid var(--border-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.reader-view.notes-open .reader-content {
  flex: 1;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-primary);
}

.sidebar-header h3 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
}

.sidebar-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.filter-tabs {
  display: flex;
  gap: 2px;
  padding: 6px 12px;
  border-bottom: 1px solid var(--border-primary);
  background: var(--bg-tertiary);
}

.filter-tab {
  flex: 1;
  padding: 5px 8px;
  font-size: 12px;
  font-weight: 500;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.filter-tab:hover {
  background: var(--bg-hover);
}

.filter-tab.active {
  background: var(--color-primary);
  color: var(--text-inverse);
}

.sidebar-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 300px;
}

/* Placeholder states */
.reader-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  gap: 8px;
}

.reader-placeholder.error {
  color: var(--color-danger);
}

.placeholder-icon {
  font-size: 48px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Page jump dialog */
.page-jump-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.page-jump-dialog {
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 240px;
  box-shadow: var(--shadow-lg);
}

.page-jump-dialog label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.page-jump-dialog .input {
  width: 100%;
}

.page-jump-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  padding: 6px 14px;
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

.btn-primary:hover {
  background: var(--color-primary-hover);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-secondary);
}

.btn-secondary:hover {
  background: var(--bg-active);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

/* AI sidebar */
.ai-sidebar {
  width: 320px;
  flex-shrink: 0;
  border-left: 1px solid var(--border-primary);
  background: var(--bg-primary);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.reader-ai-slide-enter-active,
.reader-ai-slide-leave-active {
  transition: width 0.22s ease, opacity 0.18s ease;
  overflow: hidden;
}

.reader-ai-slide-enter-from,
.reader-ai-slide-leave-to {
  width: 0;
  opacity: 0;
}

.reader-ai-slide-enter-to,
.reader-ai-slide-leave-from {
  width: 320px;
  opacity: 1;
}

/* AI button in toolbar */
.ai-btn.active {
  color: var(--color-primary);
  background: rgba(99, 102, 241, 0.1);
}
</style>
