<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue'
import { MdEditor, MdPreview } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'
import { useNoteLinks } from '../../composables/useNoteLinks'
import type { Note, CreateNoteInput, UpdateNoteInput, NoteLinkTarget } from '../../types'

const props = defineProps<{
  note?: Note | null
  paperId?: string
  currentPage?: number
  mode: 'create' | 'edit'
}>()

const emit = defineEmits<{
  save: [input: CreateNoteInput | UpdateNoteInput]
  cancel: []
}>()

const title = ref(props.note?.title || '')
const content = ref(props.note?.content || '')
const noteType = ref<string>(props.note?.note_type || 'general')
const pageNumber = ref<number | null>(props.note?.page_number ?? props.currentPage ?? null)
const previewOnly = ref(false)

const noteTypes = [
  { value: 'general', label: '通用笔记', icon: '📝' },
  { value: 'annotation', label: '批注', icon: '📌' },
  { value: 'excerpt', label: '摘录', icon: '📋' },
  { value: 'summary', label: '总结', icon: '📄' },
]

const isAnnotation = computed(() => noteType.value === 'annotation')

// Auto-set page number when switching to annotation type
watch(noteType, (newType) => {
  if (newType === 'annotation' && pageNumber.value === null && props.currentPage) {
    pageNumber.value = props.currentPage
  }
})

function handleSave() {
  if (!title.value.trim() && !content.value.trim()) return

  if (props.mode === 'create') {
    const input: CreateNoteInput = {
      paper_id: props.paperId || null,
      title: title.value.trim() || (pageNumber.value ? `第 ${pageNumber.value} 页批注` : '无标题笔记'),
      content: content.value,
      note_type: noteType.value,
      page_number: pageNumber.value,
      position: null,
    }
    emit('save', input)
  } else if (props.note) {
    const input: UpdateNoteInput = {
      id: props.note.id,
      title: title.value.trim(),
      content: content.value,
      note_type: noteType.value,
      page_number: pageNumber.value,
    }
    emit('save', input)
  }
}

function togglePreview() {
  previewOnly.value = !previewOnly.value
}

// ── [[Link Autocomplete]] ──────────────────────────────────────────────────

const { suggestions, suggestionsLoading, fetchSuggestions, clearSuggestions, createLink } = useNoteLinks()
const showLinkPopup = ref(false)
const linkSearchQuery = ref('')
let linkDebounceTimer: ReturnType<typeof setTimeout> | null = null

function openLinkPopup() {
  showLinkPopup.value = true
  linkSearchQuery.value = ''
  suggestions.value = []
  nextTick(() => {
    const input = document.querySelector('.link-search-input') as HTMLInputElement
    input?.focus()
  })
}

function closeLinkPopup() {
  showLinkPopup.value = false
  clearSuggestions()
  linkSearchQuery.value = ''
}

function onLinkSearchInput(query: string) {
  linkSearchQuery.value = query
  if (linkDebounceTimer) clearTimeout(linkDebounceTimer)
  linkDebounceTimer = setTimeout(() => {
    fetchSuggestions(query)
  }, 200)
}

function selectLinkTarget(target: NoteLinkTarget) {
  // Insert [[title]] at cursor position in content
  const linkSyntax = `[[${target.title}]]`
  content.value = content.value ? content.value + '\n' + linkSyntax : linkSyntax
  closeLinkPopup()

  // Also create the DB link if we have a note ID
  if (props.note?.id) {
    createLink(props.note.id, target.note_id).catch(e => {
      console.warn('Failed to create DB link:', e)
    })
  }
}
</script>

<template>
  <div class="note-editor">
    <!-- Header bar -->
    <div class="editor-header">
      <div class="editor-header-left">
        <input
          v-model="title"
          class="note-title-input"
          placeholder="笔记标题"
        />
      </div>
      <div class="editor-header-right">
        <!-- Note type selector -->
        <div class="type-selector">
          <button
            v-for="t in noteTypes"
            :key="t.value"
            class="type-btn"
            :class="{ active: noteType === t.value }"
            @click="noteType = t.value"
            :title="t.label"
          >
            {{ t.icon }}
          </button>
        </div>

        <!-- Page number (for annotations) -->
        <div v-if="isAnnotation || pageNumber" class="page-input-group">
          <span class="page-label">P</span>
          <input
            v-model.number="pageNumber"
            type="number"
            min="1"
            class="page-input"
            placeholder="页码"
          />
        </div>

        <!-- Insert link button -->
        <button class="btn-ghost link-btn" @click="openLinkPopup" title="插入笔记链接 [[...]]">
          <svg viewBox="0 0 16 16" fill="currentColor" width="14" height="14">
            <path d="M6.354 5.5H4a3 3 0 0 0 0 6h3a3 3 0 0 0 2.83-4H9c-.086 0-.17.01-.254.031A2 2 0 0 1 7 9.5H4a2 2 0 1 1 0-4h1.5a.5.5 0 0 0 0-1zm3.292 0h1.5a.5.5 0 0 0 0-1H9a3 3 0 0 0-2.83 4H7c.086 0 .17-.01.254-.031A2 2 0 0 1 9 6.5h3a2 2 0 1 1 0 4h-1.5a.5.5 0 0 0 0 1H12a3 3 0 0 0 0-6h-1.5a.5.5 0 0 0 0 1z"/>
          </svg>
        </button>

        <!-- Preview toggle -->
        <button class="btn-ghost" @click="togglePreview" :title="previewOnly ? '编辑' : '仅预览'">
          {{ previewOnly ? '✏️' : '👁️' }}
        </button>

        <!-- Save / Cancel -->
        <button class="btn btn-primary btn-sm" @click="handleSave">保存</button>
        <button class="btn-ghost" @click="emit('cancel')">取消</button>
      </div>
    </div>

    <!-- Editor body -->
    <div class="editor-body">
      <MdEditor
        v-if="!previewOnly"
        v-model="content"
        :preview="true"
        :toolbarsExclude="['github', 'htmlPreview', 'catalog', 'save']"
        language="zh-CN"
        :style="{ height: '100%' }"
        placeholder="开始写笔记... (支持 Markdown 语法，使用 [[笔记标题]] 创建双向链接)"
      />
      <MdPreview
        v-else
        :modelValue="content"
        language="zh-CN"
        class="note-preview"
      />
    </div>

    <!-- [[Link Autocomplete]] Popup -->
    <Teleport to="body">
      <div v-if="showLinkPopup" class="link-popup-overlay" @click.self="closeLinkPopup">
        <div class="link-popup">
          <div class="link-popup-header">
            <svg viewBox="0 0 16 16" fill="currentColor" width="14" height="14" class="link-icon">
              <path d="M6.354 5.5H4a3 3 0 0 0 0 6h3a3 3 0 0 0 2.83-4H9c-.086 0-.17.01-.254.031A2 2 0 0 1 7 9.5H4a2 2 0 1 1 0-4h1.5a.5.5 0 0 0 0-1zm3.292 0h1.5a.5.5 0 0 0 0-1H9a3 3 0 0 0-2.83 4H7c.086 0 .17-.01.254-.031A2 2 0 0 1 9 6.5h3a2 2 0 1 1 0 4h-1.5a.5.5 0 0 0 0 1H12a3 3 0 0 0 0-6h-1.5a.5.5 0 0 0 0 1z"/>
            </svg>
            <span>插入笔记链接</span>
            <button class="btn-ghost" @click="closeLinkPopup">✕</button>
          </div>
          <input
            class="link-search-input"
            :value="linkSearchQuery"
            @input="onLinkSearchInput(($event.target as HTMLInputElement).value)"
            placeholder="搜索笔记标题..."
          />
          <div class="link-suggestions">
            <div v-if="suggestionsLoading" class="link-loading">搜索中...</div>
            <div v-else-if="suggestions.length === 0 && linkSearchQuery" class="link-empty">未找到匹配笔记</div>
            <div
              v-for="s in suggestions"
              :key="s.note_id"
              class="link-suggestion-item"
              @click="selectLinkTarget(s)"
            >
              <span class="suggestion-title">{{ s.title }}</span>
              <span v-if="s.label" class="suggestion-label">{{ s.label }}</span>
            </div>
          </div>
          <div class="link-popup-footer">
            <span class="link-hint">输入关键词搜索笔记，点击插入 [[链接]]</span>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.note-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-primary);
  gap: 8px;
  flex-shrink: 0;
}

.editor-header-left {
  flex: 1;
  min-width: 0;
}

.editor-header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.note-title-input {
  width: 100%;
  border: none;
  background: transparent;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  outline: none;
  padding: 4px 0;
}

.note-title-input::placeholder {
  color: var(--text-tertiary);
}

.type-selector {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  padding: 2px;
}

.type-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  font-size: 14px;
  transition: all var(--transition-fast);
}

.type-btn:hover {
  background: var(--bg-hover);
}

.type-btn.active {
  background: var(--color-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
}

.page-input-group {
  display: flex;
  align-items: center;
  gap: 3px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  padding: 2px 6px;
}

.page-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.page-input {
  width: 40px;
  border: none;
  background: transparent;
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  outline: none;
  text-align: center;
  padding: 2px 0;
}

.page-input::-webkit-inner-spin-button,
.page-input::-webkit-outer-spin-button {
  -webkit-appearance: none;
}

.btn-ghost {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  font-size: 14px;
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.btn-ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn {
  padding: 5px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
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

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.editor-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.note-preview {
  padding: 16px;
  overflow: auto;
  flex: 1;
}

/* Link button */
.link-btn {
  display: flex;
  align-items: center;
  gap: 2px;
  color: var(--color-primary);
}

.link-btn:hover {
  color: var(--color-primary-hover);
  background: rgba(99, 102, 241, 0.08);
}

/* ── Link Popup ──────────────────────────────────────────────────────────── */
.link-popup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}

.link-popup {
  width: 380px;
  max-height: 420px;
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.link-popup-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-primary);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.link-popup-header .link-icon {
  color: var(--color-primary);
}

.link-popup-header span {
  flex: 1;
}

.link-popup-header .btn-ghost {
  font-size: 12px;
  color: var(--text-tertiary);
}

.link-search-input {
  width: 100%;
  border: none;
  border-bottom: 1px solid var(--border-primary);
  background: var(--bg-primary);
  font-size: 13px;
  color: var(--text-primary);
  padding: 10px 14px;
  outline: none;
}

.link-search-input::placeholder {
  color: var(--text-tertiary);
}

.link-suggestions {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  min-height: 80px;
  max-height: 260px;
}

.link-loading,
.link-empty {
  text-align: center;
  padding: 24px;
  color: var(--text-tertiary);
  font-size: 12px;
}

.link-suggestion-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.link-suggestion-item:hover {
  background: var(--bg-hover);
}

.suggestion-title {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suggestion-label {
  font-size: 10px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
}

.link-popup-footer {
  padding: 6px 14px;
  border-top: 1px solid var(--border-primary);
}

.link-hint {
  font-size: 10px;
  color: var(--text-tertiary);
}
</style>
