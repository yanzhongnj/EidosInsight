<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { useAppStore } from '../../stores/app'
import { useTauriCommand } from '../../composables/useTauriCommand'
import type { Paper, Tag } from '../../types'

const props = defineProps<{
  paper: Paper
  viewMode: 'grid' | 'list'
  tags?: Tag[]
}>()

const emit = defineEmits<{
  click: []
  read: []
  toggleFavorite: []
  delete: []
  moveToFolder: [folderId: string | null]
  setStatus: [status: 'unread' | 'reading' | 'read']
  addTag: [tagName: string]
  removeTag: [tagId: string]
}>()

const appStore = useAppStore()
const cmd = useTauriCommand()

const folderName = computed(() => {
  if (!props.paper.folder_id) return null
  const folder = appStore.folders.find(f => f.id === props.paper.folder_id)
  return folder ? folder.name : null
})

const folderColor = computed(() => {
  if (!props.paper.folder_id) return null
  const folder = appStore.folders.find(f => f.id === props.paper.folder_id)
  return folder?.color || null
})

const displayTags = computed(() => {
  return props.tags?.slice(0, 3) ?? []
})

const moreTagCount = computed(() => {
  if (!props.tags) return 0
  return Math.max(0, props.tags.length - 3)
})

// Context menu state
const showMenu = ref(false)
const menuRef = ref<HTMLElement | null>(null)

// Tag editing state
const showTagSection = ref(false)
const tagSearchQuery = ref('')
const tagSearchInput = ref<HTMLInputElement | null>(null)

function toggleMenu(e: MouseEvent) {
  e.stopPropagation()
  showMenu.value = !showMenu.value
  // Reset tag section when menu opens
  if (showMenu.value) {
    showTagSection.value = false
    tagSearchQuery.value = ''
  }
}

function closeMenu() {
  showMenu.value = false
  showTagSection.value = false
  tagSearchQuery.value = ''
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    closeMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})

function handleDelete() {
  closeMenu()
  emit('delete')
}

function handleMoveToFolder(folderId: string | null) {
  closeMenu()
  emit('moveToFolder', folderId)
}

function handleSetStatus(status: 'unread' | 'reading' | 'read') {
  closeMenu()
  emit('setStatus', status)
}

function toggleTagSection() {
  showTagSection.value = !showTagSection.value
  if (showTagSection.value) {
    // Focus input after next tick
    setTimeout(() => {
      tagSearchInput.value?.focus()
    }, 50)
  }
}

// Selected tag IDs from the tags prop
const selectedTagIds = computed(() => {
  return (props.tags ?? []).map(t => t.id)
})

// Available tags (not yet on this paper), filtered by search
const availableTags = computed(() => {
  const selected = new Set(selectedTagIds.value)
  let tags = appStore.tags.filter(t => !selected.has(t.id))
  if (tagSearchQuery.value.trim()) {
    const q = tagSearchQuery.value.toLowerCase()
    tags = tags.filter(t => t.name.toLowerCase().includes(q))
  }
  return tags.slice(0, 10)
})

function handleAddTag(tagName: string) {
  emit('addTag', tagName)
  tagSearchQuery.value = ''
}

function handleRemoveTag(tagId: string) {
  emit('removeTag', tagId)
}

function handleTagKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && tagSearchQuery.value.trim()) {
    handleAddTag(tagSearchQuery.value.trim())
  } else if (e.key === 'Escape') {
    showTagSection.value = false
    tagSearchQuery.value = ''
  }
}

// Folders for move menu (excluding current folder)
const availableFolders = computed(() => {
  return appStore.folders.filter(f => f.id !== props.paper.folder_id)
})

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
        <div class="card-header-actions">
          <button
            class="fav-btn"
            :class="{ active: paper.is_favorite }"
            @click.stop="emit('toggleFavorite')"
          >
            {{ paper.is_favorite ? '★' : '☆' }}
          </button>
          <button class="more-btn" @click="toggleMenu" title="更多操作">⋮</button>
        </div>
      </div>

      <p v-if="paper.authors" class="paper-authors">{{ paper.authors }}</p>

      <div class="paper-meta">
        <span v-if="folderName" class="folder-badge" :style="folderColor ? { backgroundColor: folderColor + '20', color: folderColor } : {}">
          📁 {{ folderName }}
        </span>
        <span v-if="paper.journal" class="meta-item">{{ paper.journal }}</span>
        <span v-if="paper.publication_date" class="meta-item">{{ paper.publication_date }}</span>
        <span :class="['status-badge', `status-${paper.reading_status}`]">
          {{ statusLabel(paper.reading_status) }}
        </span>
      </div>

      <div v-if="displayTags.length > 0" class="paper-tags">
        <span
          v-for="tag in displayTags"
          :key="tag.id"
          class="tag-chip"
          :style="tag.color ? { backgroundColor: tag.color + '18', color: tag.color, borderColor: tag.color + '40' } : {}"
        >
          {{ tag.name }}
        </span>
        <span v-if="moreTagCount > 0" class="tag-more">+{{ moreTagCount }}</span>
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

    <!-- Context Menu -->
    <div v-if="showMenu" ref="menuRef" class="context-menu" @click.stop>
      <div class="menu-section">
        <div class="menu-section-title">阅读状态</div>
        <button class="menu-item" :class="{ active: paper.reading_status === 'unread' }" @click="handleSetStatus('unread')">
          📭 未读
        </button>
        <button class="menu-item" :class="{ active: paper.reading_status === 'reading' }" @click="handleSetStatus('reading')">
          📖 在读
        </button>
        <button class="menu-item" :class="{ active: paper.reading_status === 'read' }" @click="handleSetStatus('read')">
          ✅ 已读
        </button>
      </div>

      <div class="menu-divider"></div>

      <div class="menu-section">
        <div class="menu-section-title">归类到专题</div>
        <button
          v-if="paper.folder_id"
          class="menu-item"
          @click="handleMoveToFolder(null)"
        >
          🗑️ 移出当前专题
        </button>
        <button
          v-for="folder in availableFolders"
          :key="folder.id"
          class="menu-item"
          @click="handleMoveToFolder(folder.id)"
        >
          📁 {{ folder.name }}
        </button>
        <div v-if="availableFolders.length === 0 && !paper.folder_id" class="menu-item disabled">
          暂无可用的专题
        </div>
      </div>

      <div class="menu-divider"></div>

      <!-- Tag section -->
      <div class="menu-section">
        <div class="menu-section-title clickable" @click="toggleTagSection">
          🏷️ 管理标签
          <span class="section-toggle-inline">{{ showTagSection ? '▾' : '▸' }}</span>
        </div>

        <div v-if="showTagSection" class="tag-edit-area">
          <!-- Current tags (removable) -->
          <div v-if="tags && tags.length > 0" class="current-tags">
            <span
              v-for="tag in tags"
              :key="tag.id"
              class="tag-chip-removable"
              :style="tag.color ? { backgroundColor: `${tag.color}18`, borderColor: `${tag.color}40`, color: tag.color } : {}"
            >
              {{ tag.name }}
              <button class="tag-remove-btn" @click.stop="handleRemoveTag(tag.id)">×</button>
            </span>
          </div>
          <div v-else class="no-tags-hint">暂无标签</div>

          <!-- Search/add input -->
          <input
            ref="tagSearchInput"
            v-model="tagSearchQuery"
            class="tag-search-input"
            placeholder="搜索或新建标签..."
            @keydown="handleTagKeydown"
            @click.stop
          />

          <!-- Available tags dropdown -->
          <div v-if="availableTags.length > 0" class="tag-dropdown-inline">
            <button
              v-for="tag in availableTags"
              :key="tag.id"
              class="tag-option"
              :style="tag.color ? { color: tag.color } : {}"
              @click.stop="handleAddTag(tag.name)"
            >
              {{ tag.name }}
              <span class="tag-usage-count">{{ tag.usage_count }}</span>
            </button>
          </div>

          <!-- Create new tag option -->
          <button
            v-if="tagSearchQuery.trim() && !appStore.tags.find(t => t.name === tagSearchQuery.trim())"
            class="tag-option tag-create-option"
            @click.stop="handleAddTag(tagSearchQuery.trim())"
          >
            + 创建 "{{ tagSearchQuery.trim() }}"
          </button>
        </div>
      </div>

      <div class="menu-divider"></div>

      <button class="menu-item menu-item-danger" @click="handleDelete">
        🗑️ 删除文献
      </button>
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
  position: relative;
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

.card-header-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
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
  flex: 1;
  min-width: 0;
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
  border-radius: 4px;
}

.fav-btn.active,
.fav-btn:hover {
  color: var(--color-accent);
}

.more-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 18px;
  font-weight: bold;
  color: var(--text-tertiary);
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.15s;
  line-height: 1;
}

.more-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
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

.folder-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 9999px;
  font-weight: 500;
  background: color-mix(in srgb, #8b5cf6 12%, transparent);
  color: #7c3aed;
}

.paper-tags {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 6px;
  flex-wrap: wrap;
}

.tag-chip {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 9999px;
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  white-space: nowrap;
}

.tag-more {
  font-size: 11px;
  color: var(--text-tertiary);
  padding-left: 2px;
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

/* Context Menu */
.context-menu {
  position: absolute;
  top: 8px;
  right: 8px;
  background: var(--bg-card);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  z-index: 100;
  min-width: 180px;
  padding: 4px 0;
  overflow: hidden;
}

.menu-section {
  padding: 2px 0;
}

.menu-section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  padding: 6px 12px 2px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  border: none;
  background: none;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s;
  white-space: nowrap;
}

.menu-item:hover {
  background: var(--bg-tertiary);
}

.menu-item.active {
  color: var(--color-primary);
  font-weight: 500;
}

.menu-item.disabled {
  color: var(--text-tertiary);
  cursor: default;
  font-style: italic;
}

.menu-item.disabled:hover {
  background: none;
}

.menu-item-danger {
  color: #ef4444;
}

.menu-item-danger:hover {
  background: color-mix(in srgb, #ef4444 8%, transparent);
}

.menu-divider {
  height: 1px;
  background: var(--border-primary);
  margin: 4px 0;
}

/* Tag edit area */
.menu-section-title.clickable {
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 12px;
}

.menu-section-title.clickable:hover {
  color: var(--text-secondary);
}

.section-toggle-inline {
  font-size: 10px;
  opacity: 0.6;
}

.tag-edit-area {
  padding: 4px 8px;
}

.current-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 6px;
}

.tag-chip-removable {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 2px 6px 2px 8px;
  border-radius: 9999px;
  border: 1px solid var(--border-secondary);
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}

.tag-remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 11px;
  opacity: 0.5;
  padding: 0 2px;
  color: inherit;
  line-height: 1;
}

.tag-remove-btn:hover {
  opacity: 1;
}

.no-tags-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  font-style: italic;
  margin-bottom: 6px;
}

.tag-search-input {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--border-secondary);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
  box-sizing: border-box;
}

.tag-search-input:focus {
  border-color: var(--color-primary);
}

.tag-dropdown-inline {
  max-height: 120px;
  overflow-y: auto;
  margin-top: 4px;
}

.tag-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 4px 8px;
  border: none;
  background: none;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  border-radius: 3px;
  transition: background 0.1s;
}

.tag-option:hover {
  background: var(--bg-tertiary);
}

.tag-create-option {
  color: var(--color-primary);
  font-weight: 500;
  margin-top: 2px;
}

.tag-usage-count {
  font-size: 10px;
  opacity: 0.4;
}
</style>
