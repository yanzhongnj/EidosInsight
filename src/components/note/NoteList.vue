<script setup lang="ts">
import { computed } from 'vue'
import { MdPreview } from 'md-editor-v3'
import 'md-editor-v3/lib/preview.css'
import type { Note, NoteLinkTarget } from '../../types'

const props = defineProps<{
  notes: Note[]
  currentPage?: number
  filter?: 'all' | 'page' | 'annotation' | 'general'
  /** Backlinks map: noteId -> backlinks for that note */
  backlinksMap?: Record<string, NoteLinkTarget[]>
}>()

const emit = defineEmits<{
  select: [note: Note]
  delete: [id: string]
  edit: [note: Note]
  goToPage: [page: number]
  navigateToNote: [noteId: string]
}>()

const filteredNotes = computed(() => {
  let result = [...props.notes]

  switch (props.filter) {
    case 'page':
      if (props.currentPage) {
        result = result.filter(n => n.page_number === props.currentPage)
      }
      break
    case 'annotation':
      result = result.filter(n => n.note_type === 'annotation')
      break
    case 'general':
      result = result.filter(n => n.note_type === 'general')
      break
  }

  // Sort: annotations with page numbers first, then by date
  return result.sort((a, b) => {
    if (a.page_number !== null && b.page_number !== null) {
      return a.page_number - b.page_number || a.date_created.localeCompare(b.date_created)
    }
    if (a.page_number !== null) return -1
    if (b.page_number !== null) return 1
    return b.date_modified.localeCompare(a.date_modified)
  })
})

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

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr)
    const now = new Date()
    const diffMs = now.getTime() - d.getTime()
    const diffMins = Math.floor(diffMs / 60000)

    if (diffMins < 1) return '刚刚'
    if (diffMins < 60) return `${diffMins} 分钟前`
    const diffHours = Math.floor(diffMins / 60)
    if (diffHours < 24) return `${diffHours} 小时前`
    const diffDays = Math.floor(diffHours / 24)
    if (diffDays < 7) return `${diffDays} 天前`
    return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
  } catch {
    return dateStr
  }
}

function getBacklinks(noteId: string): NoteLinkTarget[] {
  return props.backlinksMap?.[noteId] || []
}

/** Count [[wiki-links]] in content */
function countWikiLinks(content: string): number {
  const matches = content.match(/\[\[[^\]]+\]\]/g)
  return matches ? matches.length : 0
}
</script>

<template>
  <div class="note-list">
    <div v-if="filteredNotes.length === 0" class="empty-notes">
      <div class="empty-icon">📝</div>
      <p>{{ filter === 'page' ? '当前页没有批注' : '暂无笔记' }}</p>
    </div>

    <div
      v-for="note in filteredNotes"
      :key="note.id"
      class="note-card"
      @click="emit('select', note)"
    >
      <div class="note-card-header">
        <div class="note-type-badge" :class="`type-${note.note_type}`">
          {{ noteTypeIcons[note.note_type] || '📝' }}
        </div>
        <h4 class="note-card-title">{{ note.title }}</h4>
        <div class="note-card-actions">
          <button
            v-if="note.page_number"
            class="page-badge"
            @click.stop="emit('goToPage', note.page_number)"
            title="跳转到第 {{ note.page_number }} 页"
          >
            P{{ note.page_number }}
          </button>
          <!-- Wiki-link count badge -->
          <span v-if="countWikiLinks(note.content) > 0" class="link-badge" title="包含双向链接">
            🔗 {{ countWikiLinks(note.content) }}
          </span>
          <button class="action-btn" @click.stop="emit('edit', note)" title="编辑">✏️</button>
          <button class="action-btn danger" @click.stop="emit('delete', note.id)" title="删除">🗑️</button>
        </div>
      </div>

      <div class="note-card-content">
        <MdPreview
          :modelValue="note.content.length > 300 ? note.content.slice(0, 300) + '...' : note.content"
          language="zh-CN"
          class="note-content-preview"
        />
      </div>

      <!-- Backlinks section -->
      <div v-if="getBacklinks(note.id).length > 0" class="note-backlinks">
        <span class="backlink-label">被引用:</span>
        <button
          v-for="bl in getBacklinks(note.id).slice(0, 3)"
          :key="bl.note_id"
          class="backlink-chip"
          @click.stop="emit('navigateToNote', bl.note_id)"
          :title="bl.title"
        >
          ← {{ bl.title }}
        </button>
        <span v-if="getBacklinks(note.id).length > 3" class="backlink-more">
          +{{ getBacklinks(note.id).length - 3 }}
        </span>
      </div>

      <div class="note-card-footer">
        <span class="note-type-label">{{ noteTypeLabels[note.note_type] || '笔记' }}</span>
        <span class="note-date">{{ formatDate(note.date_modified) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.note-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
  overflow-y: auto;
  flex: 1;
}

.empty-notes {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  color: var(--text-tertiary);
  text-align: center;
  gap: 8px;
}

.empty-icon {
  font-size: 32px;
  opacity: 0.5;
}

.note-card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.note-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
}

.note-card-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.note-type-badge {
  font-size: 14px;
  flex-shrink: 0;
}

.note-card-title {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin: 0;
}

.note-card-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.note-card:hover .note-card-actions {
  opacity: 1;
}

.page-badge {
  background: var(--color-primary);
  color: var(--text-inverse);
  font-size: 10px;
  font-weight: 600;
  font-family: var(--font-mono);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.page-badge:hover {
  opacity: 0.85;
}

/* Link count badge */
.link-badge {
  font-size: 10px;
  color: var(--color-primary);
  background: rgba(99, 102, 241, 0.1);
  padding: 1px 5px;
  border-radius: var(--radius-sm);
  white-space: nowrap;
}

.action-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  font-size: 12px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-hover);
}

.action-btn.danger:hover {
  background: rgba(220, 38, 38, 0.1);
}

.note-card-content {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  max-height: 100px;
  overflow: hidden;
}

.note-content-preview {
  font-size: 12px !important;
}

.note-content-preview :deep(.md-preview) {
  padding: 0 !important;
}

/* ── Backlinks section ──────────────────────────────────────────────────── */
.note-backlinks {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px dashed var(--border-secondary);
}

.backlink-label {
  font-size: 10px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.backlink-chip {
  font-size: 10px;
  color: var(--color-primary);
  background: rgba(99, 102, 241, 0.08);
  border: 1px solid rgba(99, 102, 241, 0.15);
  border-radius: var(--radius-sm);
  padding: 1px 6px;
  cursor: pointer;
  transition: all var(--transition-fast);
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.backlink-chip:hover {
  background: rgba(99, 102, 241, 0.15);
  border-color: var(--color-primary);
}

.backlink-more {
  font-size: 10px;
  color: var(--text-tertiary);
}

.note-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 6px;
}

.note-type-label {
  font-size: 10px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
}

.note-date {
  font-size: 10px;
  color: var(--text-tertiary);
}
</style>
