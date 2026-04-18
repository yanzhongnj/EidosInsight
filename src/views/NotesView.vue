<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useNoteStore } from '../stores/note'
import { useNoteLinks } from '../composables/useNoteLinks'
import NoteList from '../components/note/NoteList.vue'
import NoteEditor from '../components/note/NoteEditor.vue'
import type { Note, CreateNoteInput, UpdateNoteInput, NoteLinkTarget } from '../types'

const router = useRouter()
const noteStore = useNoteStore()
const { searchResults, searchLoading, searchNotes, clearSearch } = useNoteLinks()

const editingNote = ref(false)
const editingNoteData = ref<Note | null>(null)
const creatingNote = ref(false)
const noteFilter = ref<'all' | 'annotation' | 'general'>('all')
const searchQuery = ref('')
const searchMode = ref(false)

// Backlinks map for all visible notes
const backlinksMap = ref<Record<string, NoteLinkTarget[]>>({})

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

onMounted(() => {
  noteStore.fetchNotes()
})

// Switch between normal list and search mode
watch(searchQuery, (q) => {
  if (q.trim()) {
    searchMode.value = true
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
    searchDebounceTimer = setTimeout(() => {
      searchNotes(q)
    }, 250)
  } else {
    searchMode.value = false
    clearSearch()
  }
})

// When notes list changes, fetch backlinks for each
watch(() => noteStore.notes, async (notes) => {
  if (notes.length === 0) return
  const map: Record<string, NoteLinkTarget[]> = {}
  for (const note of notes.slice(0, 50)) {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      map[note.id] = await invoke<NoteLinkTarget[]>('list_backlinks', { noteId: note.id })
    } catch {
      map[note.id] = []
    }
  }
  backlinksMap.value = map
}, { immediate: true })

const allNotes = computed(() => {
  return noteStore.notes
})

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
  if (note.paper_id) {
    router.push(`/paper/${note.paper_id}`)
  }
}

function handleGoToPage(page: number) {
  const note = noteStore.notes.find(n => n.page_number === page && n.paper_id)
  if (note?.paper_id) {
    router.push(`/reader/${note.paper_id}`)
  }
}

function handleNavigateToNote(noteId: string) {
  const note = noteStore.notes.find(n => n.id === noteId)
  if (note) {
    handleNoteSelect(note)
  }
}

function cancelEdit() {
  editingNote.value = false
  creatingNote.value = false
  editingNoteData.value = null
}

function clearSearchQuery() {
  searchQuery.value = ''
}
</script>

<template>
  <div class="notes-view">
    <div class="notes-header">
      <h1 class="page-title">笔记</h1>
      <button class="btn btn-primary" @click="startCreateNote">+ 新笔记</button>
    </div>

    <!-- Search bar -->
    <div class="search-bar">
      <div class="search-input-wrapper">
        <input
          v-model="searchQuery"
          class="input"
          placeholder="全局搜索笔记 (支持全文检索)..."
        />
        <button v-if="searchQuery" class="search-clear" @click="clearSearchQuery">✕</button>
        <div v-if="searchLoading" class="search-spinner"></div>
      </div>
      <div class="search-meta" v-if="searchMode && searchResults">
        <span class="search-count">找到 {{ searchResults.total }} 条结果</span>
      </div>
      <div v-if="!searchMode" class="filter-tabs">
        <button
          class="filter-tab"
          :class="{ active: noteFilter === 'all' }"
          @click="noteFilter = 'all'"
        >全部</button>
        <button
          class="filter-tab"
          :class="{ active: noteFilter === 'annotation' }"
          @click="noteFilter = 'annotation'"
        >批注</button>
        <button
          class="filter-tab"
          :class="{ active: noteFilter === 'general' }"
          @click="noteFilter = 'general'"
        >通用</button>
      </div>
    </div>

    <!-- Editor modal -->
    <div v-if="editingNote" class="editor-modal">
      <div class="editor-modal-content">
        <NoteEditor
          :note="editingNoteData"
          :mode="creatingNote ? 'create' : 'edit'"
          @save="handleNoteSave"
          @cancel="cancelEdit"
        />
      </div>
    </div>

    <!-- Search results with highlights -->
    <div v-if="searchMode && searchResults" class="search-results">
      <div v-if="searchResults.items.length === 0" class="empty-state">
        <div class="empty-icon">🔍</div>
        <p>未找到匹配的笔记</p>
        <p class="empty-hint">尝试使用不同的关键词</p>
      </div>
      <div
        v-for="item in searchResults.items"
        :key="item.note.id"
        class="search-result-card"
        @click="handleNoteSelect(item.note)"
      >
        <div class="search-result-header">
          <span class="result-type-icon">{{ item.note.note_type === 'annotation' ? '📌' : '📝' }}</span>
          <h4 class="result-title">{{ item.note.title }}</h4>
          <button v-if="item.note.paper_id" class="result-paper-link" @click.stop="router.push(`/paper/${item.note.paper_id}`)">
            查看文献 →
          </button>
        </div>
        <div v-if="item.highlight" class="result-highlight" v-html="item.highlight"></div>
        <div class="result-footer">
          <span class="result-date">{{ item.note.date_modified }}</span>
        </div>
      </div>
    </div>

    <!-- Normal notes list (non-search mode) -->
    <template v-if="!searchMode">
      <div v-if="allNotes.length === 0 && !editingNote" class="empty-state">
        <div class="empty-icon">📝</div>
        <p>还没有笔记</p>
        <p class="empty-hint">在阅读 PDF 时创建批注，或点击上方按钮创建通用笔记</p>
      </div>

      <NoteList
        v-else
        :notes="allNotes"
        :filter="noteFilter"
        :backlinks-map="backlinksMap"
        @select="handleNoteSelect"
        @edit="handleNoteEdit"
        @delete="handleNoteDelete"
        @go-to-page="handleGoToPage"
        @navigate-to-note="handleNavigateToNote"
      />
    </template>
  </div>
</template>

<style scoped>
.notes-view {
  max-width: 900px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.notes-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  flex-shrink: 0;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
}

.search-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input-wrapper .input {
  width: 100%;
  padding-right: 60px;
}

.search-clear {
  position: absolute;
  right: 32px;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-tertiary);
  padding: 4px;
}

.search-clear:hover {
  color: var(--text-primary);
}

.search-spinner {
  position: absolute;
  right: 10px;
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.search-meta {
  display: flex;
  align-items: center;
}

.search-count {
  font-size: 11px;
  color: var(--text-tertiary);
}

.filter-tabs {
  display: flex;
  gap: 4px;
}

.filter-tab {
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--border-secondary);
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.filter-tab:hover {
  background: var(--bg-hover);
}

.filter-tab.active {
  background: var(--color-primary);
  color: var(--text-inverse);
  border-color: var(--color-primary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px;
  color: var(--text-tertiary);
  text-align: center;
  gap: 8px;
}

.empty-icon {
  font-size: 48px;
  opacity: 0.5;
}

.empty-hint {
  font-size: 13px;
  color: var(--text-tertiary);
}

/* ── Search results ─────────────────────────────────────────────────────── */
.search-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  flex: 1;
  padding-bottom: 20px;
}

.search-result-card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  padding: 12px 14px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.search-result-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
}

.search-result-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.result-type-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.result-title {
  flex: 1;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-paper-link {
  font-size: 11px;
  color: var(--color-primary);
  background: rgba(99, 102, 241, 0.08);
  border: none;
  border-radius: var(--radius-sm);
  padding: 2px 6px;
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--transition-fast);
}

.result-paper-link:hover {
  background: rgba(99, 102, 241, 0.15);
}

.result-highlight {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 4px;
}

.result-highlight :deep(mark) {
  background: rgba(245, 158, 11, 0.3);
  color: inherit;
  padding: 0 1px;
  border-radius: 2px;
}

.result-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.result-date {
  font-size: 10px;
  color: var(--text-tertiary);
}

/* Editor modal overlay */
.editor-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.editor-modal-content {
  width: 90%;
  max-width: 800px;
  height: 70vh;
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-lg);
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
</style>
