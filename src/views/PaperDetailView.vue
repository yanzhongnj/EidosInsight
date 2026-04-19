<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { usePaperStore } from '../stores/paper'
import { useAppStore } from '../stores/app'
import { useNoteStore } from '../stores/note'
import { useTauriCommand } from '../composables/useTauriCommand'
import NoteEditor from '../components/note/NoteEditor.vue'
import AiAssistPanel from '../components/ai/AiAssistPanel.vue'
import type { Note, CreateNoteInput, UpdateNoteInput, Tag } from '../types'

const route = useRoute()
const router = useRouter()
const paperStore = usePaperStore()
const appStore = useAppStore()
const noteStore = useNoteStore()
const cmd = useTauriCommand()
const showNoteEditor = ref(false)
const editingNote = ref<Note | null>(null)
const creatingNote = ref(false)

// AI panel toggle
const showAiPanel = ref(false)

// Tag editing state
const tagInput = ref('')
const showTagInput = ref(false)
const tagSearchInput = ref<HTMLInputElement | null>(null)

onMounted(async () => {
  const id = route.params.id as string
  await paperStore.fetchPaper(id)
  await paperStore.loadPaperTags([id])
  await noteStore.fetchNotes(id)
})

// Paper tags
const paperTags = computed<Tag[]>(() => {
  const id = route.params.id as string
  return paperStore.getPaperTags(id)
})

// Available tags (not yet on this paper), filtered by search
const availableTags = computed(() => {
  const selectedIds = new Set(paperTags.value.map(t => t.id))
  let tags = appStore.tags.filter(t => !selectedIds.has(t.id))
  if (tagInput.value.trim()) {
    const q = tagInput.value.toLowerCase()
    tags = tags.filter(t => t.name.toLowerCase().includes(q))
  }
  return tags.slice(0, 10)
})

function toggleTagInput() {
  showTagInput.value = !showTagInput.value
  if (showTagInput.value) {
    tagInput.value = ''
    setTimeout(() => tagSearchInput.value?.focus(), 50)
  }
}

async function handleAddTag(tagName: string) {
  const paperId = route.params.id as string
  try {
    await cmd.addPaperTag(paperId, tagName)
    tagInput.value = ''
    await paperStore.loadPaperTags([paperId])
    await appStore.fetchTags()
  } catch (e) {
    console.error('Add tag failed:', e)
  }
}

async function handleRemoveTag(tagId: string) {
  const paperId = route.params.id as string
  try {
    await cmd.removePaperTag(paperId, tagId)
    await paperStore.loadPaperTags([paperId])
    await appStore.fetchTags()
  } catch (e) {
    console.error('Remove tag failed:', e)
  }
}

function handleTagKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && tagInput.value.trim()) {
    handleAddTag(tagInput.value.trim())
  } else if (e.key === 'Escape') {
    showTagInput.value = false
    tagInput.value = ''
  }
}

function startCreateNote() {
  editingNote.value = null
  creatingNote.value = true
  showNoteEditor.value = true
}

function handleNoteSave(input: CreateNoteInput | UpdateNoteInput) {
  if (creatingNote.value) {
    noteStore.createNote(input as CreateNoteInput).then(() => {
      showNoteEditor.value = false
      creatingNote.value = false
    })
  } else if (editingNote.value) {
    noteStore.updateNote(input as UpdateNoteInput).then(() => {
      showNoteEditor.value = false
      editingNote.value = null
    })
  }
}

function handleNoteEdit(note: Note) {
  editingNote.value = note
  creatingNote.value = false
  showNoteEditor.value = true
}

function cancelEdit() {
  showNoteEditor.value = false
  editingNote.value = null
  creatingNote.value = false
}

const noteTypeIcons: Record<string, string> = {
  general: '📝',
  annotation: '📌',
  excerpt: '📋',
  summary: '📄',
}

const noteTypeLabels: Record<string, string> = {
  general: '通用',
  annotation: '批注',
  excerpt: '摘录',
  summary: '总结',
}

const currentPaper = computed(() => paperStore.currentPaper)
</script>

<template>
  <div class="paper-detail-layout">
    <!-- Main content -->
    <div class="paper-detail" :class="{ 'with-panel': showAiPanel }">
      <div v-if="paperStore.loading" class="loading-state">
        <div class="spinner"></div>
      </div>

      <div v-else-if="paperStore.currentPaper" class="detail-content">
        <!-- Back button + AI toggle -->
        <div class="top-bar">
          <button class="btn-ghost back-btn" @click="router.back()">← 返回</button>
          <button
            class="btn-ghost ai-toggle-btn"
            :class="{ active: showAiPanel }"
            @click="showAiPanel = !showAiPanel"
            :title="showAiPanel ? '关闭 AI 面板' : '打开 AI 助理'"
          >
            <svg viewBox="0 0 20 20" fill="currentColor" width="15" height="15">
              <path fill-rule="evenodd"
                d="M9.664 1.319a.75.75 0 01.672 0 41.059 41.059 0 018.198 5.424.75.75 0 01-.254 1.285 31.372 31.372 0 00-7.86 3.83.75.75 0 01-.84 0 31.508 31.508 0 00-2.08-1.287V9.482a31.516 31.516 0 00-5.774-2.458.75.75 0 01-.254-1.285 41.059 41.059 0 018.198-5.424zM6 12.459v3.116a.75.75 0 00.894.734 31.39 31.39 0 005.332-1.76A.75.75 0 0013 13.85v-3.391a33.016 33.016 0 00-7 2z"
                clip-rule="evenodd" />
            </svg>
            AI 助理
          </button>
        </div>

        <!-- Paper header -->
        <div class="paper-header">
          <h1 class="paper-title">{{ paperStore.currentPaper.title }}</h1>
          <div class="paper-actions">
            <button
              class="btn btn-secondary"
              @click="router.push(`/reader/${paperStore.currentPaper.id}`)"
            >
              📖 阅读PDF
            </button>
            <button
              class="btn-ghost fav-toggle"
              :class="{ active: paperStore.currentPaper.is_favorite }"
              @click="paperStore.toggleFavorite(paperStore.currentPaper)"
            >
              {{ paperStore.currentPaper.is_favorite ? '★ 已收藏' : '☆ 收藏' }}
            </button>
          </div>
        </div>

        <!-- Meta info -->
        <div class="paper-meta-section">
          <p v-if="paperStore.currentPaper.authors" class="meta-row">
            <span class="meta-label">作者</span>
            <span>{{ paperStore.currentPaper.authors }}</span>
          </p>
          <p v-if="paperStore.currentPaper.journal" class="meta-row">
            <span class="meta-label">期刊</span>
            <span>{{ paperStore.currentPaper.journal }}</span>
          </p>
          <p v-if="paperStore.currentPaper.publication_date" class="meta-row">
            <span class="meta-label">发表日期</span>
            <span>{{ paperStore.currentPaper.publication_date }}</span>
          </p>
          <p v-if="paperStore.currentPaper.doi" class="meta-row">
            <span class="meta-label">DOI</span>
            <a :href="`https://doi.org/${paperStore.currentPaper.doi}`" target="_blank">{{ paperStore.currentPaper.doi }}</a>
          </p>
          <p v-if="paperStore.currentPaper.arxiv_id" class="meta-row">
            <span class="meta-label">arXiv</span>
            <a :href="`https://arxiv.org/abs/${paperStore.currentPaper.arxiv_id}`" target="_blank">{{ paperStore.currentPaper.arxiv_id }}</a>
          </p>
          <div class="meta-row">
            <span class="meta-label">阅读状态</span>
            <select
              :value="paperStore.currentPaper.reading_status"
              @change="paperStore.setReadingStatus(paperStore.currentPaper!.id, ($event.target as HTMLSelectElement).value as 'unread' | 'reading' | 'read')"
              class="input status-select"
            >
              <option value="unread">未读</option>
              <option value="reading">在读</option>
              <option value="read">已读</option>
            </select>
          </div>
        </div>

        <!-- Tags -->
        <div class="section tag-section">
          <div class="section-header">
            <h3>🏷️ 标签</h3>
            <button class="btn btn-secondary btn-sm" @click="toggleTagInput">
              {{ showTagInput ? '收起' : '+ 添加标签' }}
            </button>
          </div>

          <div v-if="paperTags.length > 0 || showTagInput" class="tag-area">
            <!-- Current tags -->
            <div v-if="paperTags.length > 0" class="current-tags">
              <span
                v-for="tag in paperTags"
                :key="tag.id"
                class="tag-chip"
                :style="tag.color ? { backgroundColor: `${tag.color}18`, borderColor: `${tag.color}40`, color: tag.color } : {}"
              >
                {{ tag.name }}
                <button class="tag-remove-btn" @click="handleRemoveTag(tag.id)">×</button>
              </span>
            </div>
            <div v-else class="no-tags-hint">暂无标签</div>

            <!-- Add tag input -->
            <div v-if="showTagInput" class="tag-add-area">
              <input
                ref="tagSearchInput"
                v-model="tagInput"
                class="tag-input"
                placeholder="搜索或新建标签..."
                @keydown="handleTagKeydown"
              />
              <div v-if="availableTags.length > 0" class="tag-dropdown">
                <button
                  v-for="tag in availableTags"
                  :key="tag.id"
                  class="tag-option"
                  :style="tag.color ? { color: tag.color } : {}"
                  @click="handleAddTag(tag.name)"
                >
                  {{ tag.name }}
                  <span class="tag-usage-count">{{ tag.usage_count }}</span>
                </button>
              </div>
              <button
                v-if="tagInput.trim() && !appStore.tags.find(t => t.name === tagInput.trim())"
                class="tag-option tag-create-option"
                @click="handleAddTag(tagInput.trim())"
              >
                + 创建 "{{ tagInput.trim() }}"
              </button>
            </div>
          </div>
        </div>

        <!-- TL;DR -->
        <div v-if="paperStore.currentPaper.tldr" class="section">
          <h3>TL;DR</h3>
          <p class="tldr-text">{{ paperStore.currentPaper.tldr }}</p>
        </div>

        <!-- Abstract -->
        <div v-if="paperStore.currentPaper.abstract_text" class="section">
          <h3>摘要</h3>
          <p class="abstract-text">{{ paperStore.currentPaper.abstract_text }}</p>
        </div>

        <!-- Notes -->
        <div class="section">
          <div class="section-header">
            <h3>笔记 ({{ noteStore.notes.length }})</h3>
            <button class="btn btn-secondary btn-sm" @click="startCreateNote">+ 新笔记</button>
          </div>

          <!-- Note editor inline -->
          <div v-if="showNoteEditor" class="inline-editor card">
            <NoteEditor
              :note="editingNote"
              :paper-id="paperStore.currentPaper.id"
              :mode="creatingNote ? 'create' : 'edit'"
              @save="handleNoteSave"
              @cancel="cancelEdit"
            />
          </div>

          <div v-if="noteStore.notes.length === 0 && !showNoteEditor" class="empty-notes">
            还没有笔记，点击上方按钮添加
          </div>
          <div v-else class="notes-list">
            <div
              v-for="note in noteStore.notes"
              :key="note.id"
              class="note-item card"
              @click="note.page_number ? router.push(`/reader/${paperStore.currentPaper!.id}`) : undefined"
            >
              <div class="note-item-header">
                <span class="note-type-icon">{{ noteTypeIcons[note.note_type] || '📝' }}</span>
                <h4>{{ note.title }}</h4>
                <div class="note-item-badges">
                  <span v-if="note.page_number" class="page-tag">P{{ note.page_number }}</span>
                  <span class="type-tag">{{ noteTypeLabels[note.note_type] || '笔记' }}</span>
                </div>
              </div>
              <p class="note-content-preview">{{ note.content.slice(0, 150) }}</p>
              <div class="note-item-footer">
                <span class="note-date">{{ note.date_modified }}</span>
                <div class="note-item-actions">
                  <button class="btn-ghost btn-sm" @click.stop="handleNoteEdit(note)">编辑</button>
                  <button class="btn-ghost btn-sm danger" @click.stop="noteStore.deleteNote(note.id)">删除</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <p>未找到该文献</p>
      </div>
    </div>

    <!-- AI Panel (right side) -->
    <Transition name="ai-panel-slide">
      <div v-if="showAiPanel && currentPaper" class="ai-panel-wrapper">
        <AiAssistPanel :paper="currentPaper" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* ── Layout ── */
.paper-detail-layout {
  display: flex;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.paper-detail {
  flex: 1;
  overflow-y: auto;
  transition: all var(--transition-fast);
}

.paper-detail.with-panel {
  /* Main content shrinks slightly when AI panel is open */
  flex: 1 1 0;
  min-width: 0;
}

.detail-content {
  max-width: 800px;
  margin: 0 auto;
  padding: 0 0 40px;
}

/* ── AI Panel wrapper ── */
.ai-panel-wrapper {
  width: 340px;
  flex-shrink: 0;
  border-left: 1px solid var(--border-primary);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.ai-panel-slide-enter-active,
.ai-panel-slide-leave-active {
  transition: width 0.22s ease, opacity 0.18s ease;
  overflow: hidden;
}

.ai-panel-slide-enter-from,
.ai-panel-slide-leave-to {
  width: 0;
  opacity: 0;
}

.ai-panel-slide-enter-to,
.ai-panel-slide-leave-from {
  width: 340px;
  opacity: 1;
}

/* ── Top bar ── */
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.ai-toggle-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  padding: 5px 12px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-primary);
  color: var(--text-secondary);
}

.ai-toggle-btn:hover,
.ai-toggle-btn.active {
  background: rgba(99, 102, 241, 0.1);
  color: var(--color-primary);
  border-color: var(--color-primary);
}

/* ── Loading / empty ── */
.loading-state, .empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 60px;
  color: var(--text-tertiary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* ── Back btn ── */
.back-btn {
  padding: 6px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 14px;
  border-radius: var(--radius-md);
}

.back-btn:hover { background: var(--bg-hover); }

/* ── Paper header ── */
.paper-header { margin: 16px 0; }

.paper-title {
  font-size: 24px;
  font-weight: 700;
  line-height: 1.3;
  color: var(--text-primary);
}

.paper-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
}

.fav-toggle.active { color: var(--color-accent); }

/* ── Meta section ── */
.paper-meta-section {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: 16px 20px;
  margin-bottom: 24px;
}

.meta-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 0;
  font-size: 14px;
}

.meta-label {
  color: var(--text-tertiary);
  min-width: 70px;
  font-weight: 500;
}

.meta-row a {
  color: var(--color-primary);
  text-decoration: none;
}

.meta-row a:hover { text-decoration: underline; }

.status-select { max-width: 120px; }

/* ── Sections ── */
.section { margin-bottom: 24px; }

.section h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.tldr-text, .abstract-text {
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-secondary);
}

.empty-notes {
  color: var(--text-tertiary);
  font-size: 14px;
  padding: 16px 0;
}

/* ── Notes list ── */
.notes-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.note-item {
  cursor: pointer;
  transition: all var(--transition-fast);
}

.note-item:hover { border-color: var(--color-primary); }

.note-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.note-type-icon { font-size: 14px; }

.note-item-header h4 {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.note-item-badges {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.page-tag {
  background: var(--color-primary);
  color: var(--text-inverse);
  font-size: 10px;
  font-weight: 600;
  font-family: var(--font-mono);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
}

.type-tag {
  font-size: 10px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
}

.note-content-preview {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.note-item-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
}

.note-date {
  font-size: 12px;
  color: var(--text-tertiary);
}

.note-item-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.note-item:hover .note-item-actions { opacity: 1; }

/* ── Inline editor ── */
.inline-editor {
  margin-bottom: 16px;
  min-height: 300px;
  overflow: hidden;
}

.inline-editor :deep(.note-editor) { height: 300px; }

/* ── Buttons ── */
.btn-ghost {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  transition: all var(--transition-fast);
}

.btn-ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-ghost.danger:hover { color: var(--color-danger); }

.btn-sm { padding: 4px 12px; font-size: 12px; }

.btn {
  padding: 6px 14px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all var(--transition-fast);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-secondary);
}

.btn-secondary:hover { background: var(--bg-active); }

.btn-primary {
  background: var(--color-primary);
  color: var(--text-inverse);
}

.btn-primary:hover { background: var(--color-primary-hover); }

/* ── Tags ── */
.tag-section { margin-bottom: 24px; }

.tag-area {
  margin-top: 8px;
}

.current-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 3px 8px 3px 10px;
  border-radius: 9999px;
  border: 1px solid var(--border-secondary);
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  transition: all 0.15s;
}

.tag-chip:hover {
  box-shadow: 0 0 0 1px var(--color-primary);
}

.tag-remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  opacity: 0.4;
  padding: 0 2px;
  color: inherit;
  line-height: 1;
  transition: opacity 0.15s;
}

.tag-remove-btn:hover { opacity: 1; }

.no-tags-hint {
  font-size: 13px;
  color: var(--text-tertiary);
  font-style: italic;
  margin-bottom: 8px;
}

.tag-add-area {
  position: relative;
}

.tag-input {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-md);
  padding: 6px 10px;
  font-size: 13px;
  color: var(--text-primary);
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.15s;
}

.tag-input:focus { border-color: var(--color-primary); }

.tag-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--bg-card);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-md);
  max-height: 160px;
  overflow-y: auto;
  z-index: 50;
  margin-top: 2px;
}

.tag-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 6px 10px;
  border: none;
  background: none;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s;
}

.tag-option:hover { background: var(--bg-tertiary); }

.tag-usage-count {
  font-size: 10px;
  opacity: 0.4;
}

.tag-create-option {
  color: var(--color-primary);
  font-weight: 500;
  margin-top: 2px;
  border-top: 1px solid var(--border-primary);
  padding-top: 8px;
  width: 100%;
}
</style>
