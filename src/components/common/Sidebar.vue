<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../../stores/app'
import { usePaperStore } from '../../stores/paper'

const router = useRouter()
const appStore = useAppStore()
const paperStore = usePaperStore()

const menuItems = computed(() => [
  { id: 'library', icon: '📚', label: '文献库', route: '/', count: paperStore.totalPapers },
  { id: 'favorites', icon: '⭐', label: '收藏', route: '/', count: paperStore.favoriteCount },
  { id: 'unread', icon: '📖', label: '未读', route: '/', count: paperStore.unreadCount },
  { id: 'search', icon: '🔍', label: '搜索', route: '/search' },
  { id: 'notes', icon: '📝', label: '笔记', route: '/notes' },
  { id: 'automation', icon: '🤖', label: '自动化', route: '/automation' },
  { id: 'settings', icon: '⚙️', label: '设置', route: '/settings' },
])

function navigate(item: { route: string; id: string }) {
  if (item.id === 'favorites') {
    paperStore.fetchPapers({ is_favorite: true })
  } else if (item.id === 'unread') {
    paperStore.fetchPapers({ reading_status: 'unread' })
  }
  router.push(item.route)
}

async function quitApp() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch (e) {
    console.error('Failed to quit app:', e)
  }
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="app-brand" @click="router.push('/')">
        <span class="brand-icon">🔮</span>
        <span v-if="!appStore.sidebarCollapsed" class="brand-text">EidosInsight</span>
      </div>
      <button class="btn-ghost collapse-btn" @click="appStore.toggleSidebar">
        <span v-if="appStore.sidebarCollapsed">»</span>
        <span v-else>«</span>
      </button>
    </div>

    <nav class="sidebar-nav">
      <div
        v-for="item in menuItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: appStore.activeView === item.id }"
        @click="appStore.activeView = item.id; navigate(item)"
      >
        <span class="nav-icon">{{ item.icon }}</span>
        <span v-if="!appStore.sidebarCollapsed" class="nav-label">{{ item.label }}</span>
        <span v-if="!appStore.sidebarCollapsed && item.count" class="nav-count">{{ item.count }}</span>
      </div>
    </nav>

    <!-- Folders section -->
    <div v-if="!appStore.sidebarCollapsed" class="sidebar-section">
      <div class="section-header">
        <span>文件夹</span>
        <button class="btn-ghost" title="新建文件夹">+</button>
      </div>
      <div
        v-for="folder in appStore.folders"
        :key="folder.id"
        class="nav-item folder-item"
        @click="paperStore.fetchPapers({ folder_id: folder.id })"
      >
        <span class="folder-dot" :style="{ backgroundColor: folder.color || '#6366f1' }"></span>
        <span class="nav-label">{{ folder.name }}</span>
      </div>
    </div>

    <!-- Tags section -->
    <div v-if="!appStore.sidebarCollapsed" class="sidebar-section">
      <div class="section-header">
        <span>标签</span>
      </div>
      <div class="tags-list">
        <span
          v-for="tag in appStore.tags.slice(0, 10)"
          :key="tag.id"
          class="badge badge-primary tag-item"
          @click="paperStore.fetchPapers({ tag: tag.name })"
        >
          {{ tag.name }}
        </span>
      </div>
    </div>

    <!-- Bottom spacer + quit -->
    <div class="sidebar-footer">
      <div class="nav-item quit-item" @click="quitApp" title="退出应用">
        <span class="nav-icon">🚪</span>
        <span v-if="!appStore.sidebarCollapsed" class="nav-label">退出</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-primary);
  display: flex;
  flex-direction: column;
  transition: width var(--transition-slow);
  overflow-y: auto;
  flex-shrink: 0;
}

.sidebar-collapsed .sidebar {
  width: var(--sidebar-collapsed-width);
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-primary);
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-weight: 700;
  font-size: 16px;
  color: var(--color-primary);
}

.brand-icon {
  font-size: 20px;
}

.collapse-btn {
  padding: 4px 8px;
  font-size: 14px;
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
}

.collapse-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.sidebar-nav {
  padding: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  color: var(--text-secondary);
  font-size: 14px;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-weight: 500;
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.nav-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: 9999px;
}

.sidebar-section {
  padding: 12px 8px;
  border-top: 1px solid var(--border-primary);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.folder-item {
  padding: 6px 12px;
}

.folder-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 4px 12px;
}

.tag-item {
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.tag-item:hover {
  opacity: 0.8;
}

.sidebar-footer {
  margin-top: auto;
  padding: 8px;
  border-top: 1px solid var(--border-primary);
}

.quit-item {
  color: var(--text-tertiary);
}

.quit-item:hover {
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 8%, transparent);
}
</style>
