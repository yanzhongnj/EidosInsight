<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../../stores/app'
import { usePaperStore } from '../../stores/paper'
import { useTauriCommand } from '../../composables/useTauriCommand'
import type { Folder } from '../../types'

const appStore = useAppStore()
const paperStore = usePaperStore()
const cmd = useTauriCommand()
const router = useRouter()

const showCreateInput = ref(false)
const newFolderName = ref('')
const newFolderParentId = ref<string | null>(null)
const editingFolderId = ref<string | null>(null)
const editFolderName = ref('')
const showContextMenu = ref(false)
const contextMenuFolder = ref<Folder | null>(null)
const contextMenuPos = ref({ x: 0, y: 0 })

// Track expanded/collapsed state for each folder node
const expandedFolders = ref<Set<string>>(new Set())

// Only user-created folders (no system defaults)
const rootFolders = computed(() => {
  return appStore.folders.filter(f => !f.parent_id)
})

function getChildFolders(parentId: string): Folder[] {
  return appStore.folders.filter(f => f.parent_id === parentId)
}

function hasChildren(folder: Folder): boolean {
  return appStore.folders.some(f => f.parent_id === folder.id)
}

function isExpanded(folderId: string): boolean {
  return expandedFolders.value.has(folderId)
}

function toggleExpand(folderId: string) {
  if (expandedFolders.value.has(folderId)) {
    expandedFolders.value.delete(folderId)
  } else {
    expandedFolders.value.add(folderId)
  }
  // Trigger reactivity
  expandedFolders.value = new Set(expandedFolders.value)
}

async function createFolder() {
  if (!newFolderName.value.trim()) return
  try {
    await cmd.createFolder({
      name: newFolderName.value.trim(),
      parent_id: newFolderParentId.value,
      color: randomColor(),
    })
    await appStore.fetchFolders()
    // Auto-expand parent after creating child
    if (newFolderParentId.value) {
      expandedFolders.value.add(newFolderParentId.value)
      expandedFolders.value = new Set(expandedFolders.value)
    }
    newFolderName.value = ''
    showCreateInput.value = false
    newFolderParentId.value = null
  } catch (e) {
    console.error('Failed to create folder:', e)
  }
}

async function renameFolder(folder: Folder) {
  if (!editFolderName.value.trim()) return
  try {
    await cmd.updateFolder({ id: folder.id, name: editFolderName.value.trim() })
    await appStore.fetchFolders()
    editingFolderId.value = null
  } catch (e) {
    console.error('Failed to rename folder:', e)
  }
}

async function deleteFolder(folder: Folder) {
  try {
    await cmd.deleteFolder(folder.id)
    await appStore.fetchFolders()
  } catch (e) {
    console.error('Failed to delete folder:', e)
  }
}

function startCreate(parentId: string | null = null) {
  newFolderParentId.value = parentId
  showCreateInput.value = true
  newFolderName.value = ''
}

function startRename(folder: Folder) {
  editingFolderId.value = folder.id
  editFolderName.value = folder.name
  showContextMenu.value = false
}

function onContextMenu(e: MouseEvent, folder: Folder) {
  e.preventDefault()
  contextMenuFolder.value = folder
  contextMenuPos.value = { x: e.clientX, y: e.clientY }
  showContextMenu.value = true
}

function closeContextMenu() {
  showContextMenu.value = false
  contextMenuFolder.value = null
}

function randomColor(): string {
  const colors = ['#6366f1', '#f59e0b', '#10b981', '#ef4444', '#8b5cf6', '#ec4899', '#14b8a6', '#f97316']
  return colors[Math.floor(Math.random() * colors.length)]
}

function selectFolder(folder: Folder) {
  appStore.selectedFolderId = folder.id
  appStore.activeView = 'library'
  router.push('/library')
  paperStore.fetchPapers({ folder_id: folder.id }, true)
}
</script>

<template>
  <div class="folder-tree" @click="closeContextMenu">
    <!-- Recursive tree rendering -->
    <div
      v-for="folder in rootFolders"
      :key="folder.id"
      class="topic-node"
    >
      <div
        class="topic-item"
        :class="{ 'is-selected': appStore.selectedFolderId === folder.id }"
        @click="selectFolder(folder)"
        @contextmenu="onContextMenu($event, folder)"
      >
        <!-- Expand/collapse arrow (only if has children) -->
        <span
          v-if="hasChildren(folder)"
          class="topic-toggle"
          @click.stop="toggleExpand(folder.id)"
        >{{ isExpanded(folder.id) ? '▾' : '▸' }}</span>
        <span v-else class="topic-toggle-placeholder"></span>

        <span class="topic-dot" :style="{ backgroundColor: folder.color || '#6366f1' }"></span>

        <!-- Rename input or display name -->
        <span v-if="editingFolderId === folder.id" class="topic-rename-input">
          <input
            v-model="editFolderName"
            @keyup.enter="renameFolder(folder)"
            @keyup.escape="editingFolderId = null"
            @blur="renameFolder(folder)"
            autofocus
          />
        </span>
        <span v-else class="topic-name">{{ folder.name }}</span>

        <!-- Paper count badge -->
        <span v-if="folder.paper_count > 0" class="topic-count">{{ folder.paper_count }}</span>

        <!-- Quick actions (visible on hover) -->
        <span class="topic-actions">
          <button class="action-btn" title="新建子专题" @click.stop="startCreate(folder.id)">+</button>
        </span>
      </div>

      <!-- Children (recursive via template loop) -->
      <div v-if="hasChildren(folder) && isExpanded(folder.id)" class="topic-children">
        <template v-for="child in getChildFolders(folder.id)" :key="child.id">
          <!-- Sub-folder node -->
          <div
            class="topic-item child-item"
            :class="{ 'is-selected': appStore.selectedFolderId === child.id }"
            @click="selectFolder(child)"
            @contextmenu="onContextMenu($event, child)"
          >
            <span
              v-if="hasChildren(child)"
              class="topic-toggle"
              @click.stop="toggleExpand(child.id)"
            >{{ isExpanded(child.id) ? '▾' : '▸' }}</span>
            <span v-else class="topic-toggle-placeholder"></span>

            <span class="topic-dot" :style="{ backgroundColor: child.color || '#6366f1' }"></span>

            <span v-if="editingFolderId === child.id" class="topic-rename-input">
              <input
                v-model="editFolderName"
                @keyup.enter="renameFolder(child)"
                @keyup.escape="editingFolderId = null"
                @blur="renameFolder(child)"
                autofocus
              />
            </span>
            <span v-else class="topic-name">{{ child.name }}</span>

            <!-- Paper count badge -->
            <span v-if="child.paper_count > 0" class="topic-count">{{ child.paper_count }}</span>

            <span class="topic-actions">
              <button class="action-btn" title="新建子专题" @click.stop="startCreate(child.id)">+</button>
            </span>
          </div>

          <!-- Grand-children (3rd level) -->
          <div v-if="hasChildren(child) && isExpanded(child.id)" class="topic-children depth-3">
            <div
              v-for="grandChild in getChildFolders(child.id)"
              :key="grandChild.id"
              class="topic-item child-item"
              :class="{ 'is-selected': appStore.selectedFolderId === grandChild.id }"
              @click="selectFolder(grandChild)"
              @contextmenu="onContextMenu($event, grandChild)"
            >
              <span class="topic-toggle-placeholder"></span>
              <span class="topic-dot" :style="{ backgroundColor: grandChild.color || '#6366f1' }"></span>
              <span v-if="editingFolderId === grandChild.id" class="topic-rename-input">
                <input
                  v-model="editFolderName"
                  @keyup.enter="renameFolder(grandChild)"
                  @keyup.escape="editingFolderId = null"
                  @blur="renameFolder(grandChild)"
                  autofocus
                />
              </span>
              <span v-else class="topic-name">{{ grandChild.name }}</span>
              <span v-if="grandChild.paper_count > 0" class="topic-count">{{ grandChild.paper_count }}</span>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="rootFolders.length === 0 && !showCreateInput" class="empty-topics">
      <span class="empty-text">暂无专题</span>
    </div>

    <!-- Create new topic input -->
    <div v-if="showCreateInput" class="create-topic-input">
      <input
        v-model="newFolderName"
        placeholder="输入专题名称..."
        @keyup.enter="createFolder"
        @keyup.escape="showCreateInput = false"
        autofocus
      />
      <button class="confirm-btn" @click="createFolder">✓</button>
      <button class="cancel-btn" @click="showCreateInput = false">✕</button>
    </div>

    <!-- Create button -->
    <button v-if="!showCreateInput" class="create-topic-btn" @click="startCreate(null)">
      + 新建专题
    </button>

    <!-- Context menu -->
    <div
      v-if="showContextMenu && contextMenuFolder"
      class="context-menu"
      :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
    >
      <div class="context-menu-item" @click="startRename(contextMenuFolder!)">重命名</div>
      <div class="context-menu-item" @click="startCreate(contextMenuFolder!.id)">新建子专题</div>
      <div class="context-menu-item danger" @click="deleteFolder(contextMenuFolder!)">删除专题</div>
    </div>
  </div>
</template>

<style scoped>
.folder-tree {
  padding: 4px 0;
}

.topic-node {
  margin-bottom: 1px;
}

.topic-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s;
  color: var(--text-secondary);
  font-size: 13px;
  position: relative;
}

.topic-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.topic-item.is-selected {
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-weight: 500;
}

/* Expand/collapse toggle arrow */
.topic-toggle {
  font-size: 10px;
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  cursor: pointer;
  border-radius: 3px;
  color: var(--text-tertiary);
  transition: background 0.15s;
}

.topic-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.topic-toggle-placeholder {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

/* Topic dot indicator */
.topic-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* Topic name */
.topic-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Paper count badge */
.topic-count {
  font-size: 11px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 0 5px;
  border-radius: 9999px;
  min-width: 18px;
  text-align: center;
  line-height: 16px;
  flex-shrink: 0;
}

/* Hover actions */
.topic-actions {
  display: none;
  gap: 2px;
}

.topic-item:hover .topic-actions {
  display: flex;
}

.action-btn {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 12px;
  padding: 2px 4px;
  border-radius: 4px;
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* Children indentation */
.topic-children {
  padding-left: 16px;
}

.topic-children.depth-3 {
  padding-left: 16px;
}

.child-item {
  font-size: 12px;
  padding: 4px 8px;
}

/* Rename input */
.topic-rename-input input {
  background: var(--bg-primary);
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 13px;
  color: var(--text-primary);
  width: 100%;
  outline: none;
}

/* Empty state */
.empty-topics {
  padding: 8px 12px;
  text-align: center;
}

.empty-text {
  font-size: 12px;
  color: var(--text-tertiary);
}

/* Create topic input */
.create-topic-input {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
}

.create-topic-input input {
  flex: 1;
  background: var(--bg-primary);
  border: 1px solid var(--border-secondary);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
}

.create-topic-input input:focus {
  border-color: var(--color-primary);
}

.confirm-btn, .cancel-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 4px 6px;
  border-radius: 4px;
}

.confirm-btn { color: var(--color-primary); }
.cancel-btn { color: var(--text-tertiary); }

.create-topic-btn {
  display: block;
  width: calc(100% - 24px);
  margin: 4px 12px;
  padding: 5px 8px;
  background: transparent;
  border: 1px dashed var(--border-secondary);
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.create-topic-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 5%, transparent);
}

/* Context menu */
.context-menu {
  position: fixed;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  padding: 4px;
  min-width: 140px;
  box-shadow: var(--shadow-lg);
  z-index: 1000;
}

.context-menu-item {
  padding: 6px 12px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: var(--bg-hover);
}

.context-menu-item.danger:hover {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #ef4444;
}
</style>
