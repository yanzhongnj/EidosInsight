<script setup lang="ts">
import type { Paper } from '../../types'

defineProps<{
  paper: Paper
  viewMode: 'grid' | 'list'
}>()

const emit = defineEmits<{
  click: []
  read: []
  toggleFavorite: []
}>()

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' })
  } catch {
    return dateStr
  }
}

function statusLabel(status: string): string {
  switch (status) {
    case 'unread': return '未读'
    case 'reading': return '在读'
    case 'read': return '已读'
    default: return status
  }
}
</script>

<template>
  <div
    :class="['paper-card', viewMode]"
    @click="emit('click')"
  >
    <div class="card-content">
      <div class="card-header">
        <h3 class="paper-title">
          <span v-if="paper.pdf_path" class="pdf-indicator" title="已关联 PDF">📄</span>
          {{ paper.title }}
        </h3>
        <button
          class="fav-btn"
          :class="{ active: paper.is_favorite }"
          @click.stop="emit('toggleFavorite')"
        >
          {{ paper.is_favorite ? '★' : '☆' }}
        </button>
      </div>

      <p v-if="paper.authors" class="paper-authors">{{ paper.authors }}</p>

      <div class="paper-meta">
        <span v-if="paper.journal" class="meta-item">{{ paper.journal }}</span>
        <span v-if="paper.publication_date" class="meta-item">{{ paper.publication_date }}</span>
        <span :class="['status-badge', `status-${paper.reading_status}`]">
          {{ statusLabel(paper.reading_status) }}
        </span>
        <span v-if="paper.doi" class="meta-item doi-link" :title="paper.doi">
          DOI: {{ paper.doi.slice(0, 20) }}{{ paper.doi.length > 20 ? '...' : '' }}
        </span>
      </div>

      <p v-if="viewMode === 'grid' && paper.abstract_text" class="paper-abstract">
        {{ paper.abstract_text.slice(0, 150) }}{{ paper.abstract_text.length > 150 ? '...' : '' }}
      </p>

      <div class="card-actions">
        <button
          v-if="paper.pdf_path"
          class="btn btn-secondary btn-sm"
          @click.stop="emit('read')"
        >
          📖 阅读
        </button>
        <span v-else class="no-pdf-hint">未关联 PDF</span>
        <span class="date-added">{{ formatDate(paper.date_added) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.paper-card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-normal);
}

.paper-card:hover {
  box-shadow: var(--shadow-md);
  border-color: var(--color-primary);
}

.paper-card.grid .card-content {
  padding: 16px;
}

.paper-card.list .card-content {
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.paper-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.pdf-indicator {
  font-size: 14px;
  margin-right: 4px;
  vertical-align: middle;
}

.fav-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  color: var(--text-tertiary);
  padding: 2px;
  flex-shrink: 0;
  transition: color var(--transition-fast);
}

.fav-btn.active,
.fav-btn:hover {
  color: var(--color-accent);
}

.paper-authors {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 4px;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.paper-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  flex-wrap: wrap;
}

.meta-item {
  font-size: 12px;
  color: var(--text-tertiary);
}

.doi-link {
  color: var(--color-primary);
  opacity: 0.7;
}

.doi-link:hover {
  opacity: 1;
}

.status-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 9999px;
  font-weight: 500;
}

.status-unread {
  background: color-mix(in srgb, var(--color-primary) 15%, transparent);
  color: var(--color-primary);
}

.status-reading {
  background: color-mix(in srgb, #f59e0b 15%, transparent);
  color: #d97706;
}

.status-read {
  background: color-mix(in srgb, #22c55e 15%, transparent);
  color: #16a34a;
}

.paper-abstract {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 8px;
  line-height: 1.5;
}

.card-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 12px;
}

.btn-sm {
  padding: 4px 12px;
  font-size: 12px;
}

.no-pdf-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  font-style: italic;
}

.date-added {
  font-size: 12px;
  color: var(--text-tertiary);
}

.btn {
  padding: 4px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border-color: var(--border-secondary);
}

.btn-secondary:hover {
  background: var(--bg-active);
  color: var(--color-primary);
}
</style>
