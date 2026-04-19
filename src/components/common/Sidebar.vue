<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../../stores/app'
import { usePaperStore } from '../../stores/paper'
import { useCollectionStore } from '../../stores/collection'
import FolderTree from '../folder/FolderTree.vue'
import TagCloud from '../tag/TagCloud.vue'

const router = useRouter()
const appStore = useAppStore()
const paperStore = usePaperStore()
const collectionStore = useCollectionStore()

// 标签区域折叠状态，默认折叠
const tagsExpanded = ref(false)
// 专题区域折叠状态，默认展开
const topicsExpanded = ref(true)
// 所有文献子导航折叠状态，默认折叠
const libraryExpanded = ref(false)

// 判断"所有文献"是否高亮：仅在 activeView 为 library 且不是子项时
const isLibraryActive = computed(() => {
  return appStore.activeView === 'library'
})
const totalPapers = computed(() => appStore.libraryStats?.total_papers ?? 0)

// 所有文献下的子导航项（未读/已读/收藏）
const librarySubItems = computed(() => {
  const stats = appStore.libraryStats
  return [
    { id: 'unread',    icon: '📖', label: '未读', count: stats?.unread_count ?? 0 },
    { id: 'read',      icon: '✅', label: '已读', count: stats?.read_count ?? 0 },
    { id: 'favorites', icon: '⭐', label: '收藏', count: stats?.favorite_count ?? 0 },
  ]
})

// 中部工具导航项
const toolNavItems = computed(() => [
  { id: 'collections', icon: '🔬', label: '智能分析', route: '/collections', count: collectionStore.collections.length },
  { id: 'search',      icon: '🔍', label: '搜索',     route: '/search' },
  { id: 'notes',       icon: '📝', label: '笔记',     route: '/notes' },
])

// 底部工具
const bottomNavItems = [
  { id: 'automation', icon: '🤖', label: '自动化', route: '/automation' },
  { id: 'settings',   icon: '⚙️', label: '设置',   route: '/settings' },
]

function navigateTop(id: string) {
  appStore.activeView = id
  // Clear folder selection when switching to system views
  appStore.selectedFolderId = null
  // Auto-expand library sub-nav when clicking sub-items
  if (['unread', 'read', 'favorites'].includes(id)) {
    libraryExpanded.value = true
  }
  if (id === 'hot-track') {
    router.push('/hot-track')
  } else if (id === 'library') {
    paperStore.fetchPapers({}, true)
    router.push('/library')
  } else if (id === 'unread') {
    paperStore.fetchPapers({ reading_status: 'unread' }, true)
    router.push('/library')
  } else if (id === 'read') {
    paperStore.fetchPapers({ reading_status: 'read' }, true)
    router.push('/library')
  } else if (id === 'favorites') {
    paperStore.fetchPapers({ is_favorite: true }, true)
    router.push('/library')
  }
}

function navigateTool(item: { id: string; route: string }) {
  appStore.activeView = item.id
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
    <!-- ── 品牌栏 ── -->
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

    <div class="sidebar-scroll">
      <!-- ── 文献库导航 ── -->
      <nav class="sidebar-nav">
        <!-- 热点跟踪 -->
        <div
          class="nav-item"
          :class="{ active: appStore.activeView === 'hot-track' }"
          @click="navigateTop('hot-track')"
        >
          <span class="nav-icon">🔥</span>
          <span v-if="!appStore.sidebarCollapsed" class="nav-label">热点跟踪</span>
        </div>

        <!-- 所有文献（可展开） -->
        <div
          class="nav-item nav-item-parent"
          :class="{ active: isLibraryActive }"
          @click="navigateTop('library')"
        >
          <span
            class="nav-toggle"
            @click.stop="libraryExpanded = !libraryExpanded"
          >{{ libraryExpanded ? '▾' : '▸' }}</span>
          <span class="nav-icon">📚</span>
          <span v-if="!appStore.sidebarCollapsed" class="nav-label">所有文献</span>
          <span v-if="!appStore.sidebarCollapsed && totalPapers" class="nav-count">{{ totalPapers }}</span>
        </div>

        <!-- 子导航：未读/已读/收藏（默认折叠） -->
        <div v-if="libraryExpanded && !appStore.sidebarCollapsed" class="nav-children">
          <div
            v-for="item in librarySubItems"
            :key="item.id"
            class="nav-item nav-item-child"
            :class="{ active: appStore.activeView === item.id }"
            @click="navigateTop(item.id)"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span class="nav-label">{{ item.label }}</span>
            <span v-if="item.count" class="nav-count">{{ item.count }}</span>
          </div>
        </div>
      </nav>

      <!-- ── 专题区域 ── -->
      <div v-if="!appStore.sidebarCollapsed" class="sidebar-section">
        <div class="section-header clickable" @click="topicsExpanded = !topicsExpanded">
          <span>专题</span>
          <span class="section-toggle">{{ topicsExpanded ? '▾' : '▸' }}</span>
        </div>
        <div v-if="topicsExpanded" class="section-body">
          <FolderTree />
        </div>
      </div>

      <!-- ── 中部工具导航 ── -->
      <nav class="sidebar-nav sidebar-nav-tools">
        <div
          v-for="item in toolNavItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: appStore.activeView === item.id }"
          @click="navigateTool(item)"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          <span v-if="!appStore.sidebarCollapsed" class="nav-label">{{ item.label }}</span>
          <span v-if="!appStore.sidebarCollapsed && item.count" class="nav-count">{{ item.count }}</span>
        </div>
      </nav>

      <!-- ── 标签区域（默认折叠）── -->
      <div v-if="!appStore.sidebarCollapsed" class="sidebar-section">
        <div class="section-header clickable" @click="tagsExpanded = !tagsExpanded">
          <span>🏷️ 标签</span>
          <span class="section-toggle">{{ tagsExpanded ? '▾' : '▸' }}</span>
        </div>
        <div v-if="tagsExpanded" class="section-body">
          <TagCloud />
        </div>
      </div>
    </div>

    <!-- ── 底部工具 + 退出 ── -->
    <div class="sidebar-footer">
      <div
        v-for="item in bottomNavItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: appStore.activeView === item.id }"
        @click="navigateTool(item)"
      >
        <span class="nav-icon">{{ item.icon }}</span>
        <span v-if="!appStore.sidebarCollapsed" class="nav-label">{{ item.label }}</span>
      </div>
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
  flex-shrink: 0;
  overflow: hidden;
}

.sidebar-collapsed .sidebar {
  width: var(--sidebar-collapsed-width);
}

/* 可滚动主体 */
.sidebar-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
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

/* 导航 */
.sidebar-nav {
  padding: 8px 8px 0;
}

.sidebar-nav-tools {
  padding-top: 0;
  border-top: 1px solid var(--border-primary);
  margin-top: 4px;
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

/* Expand/collapse toggle for parent nav items */
.nav-toggle {
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

.nav-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* Child navigation items (indented) */
.nav-children {
  padding-left: 20px;
}

.nav-item-child {
  font-size: 13px;
  padding: 5px 12px;
  gap: 8px;
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.nav-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: 9999px;
}

/* Section 区域 */
.sidebar-section {
  padding: 4px 8px;
  border-top: 1px solid var(--border-primary);
  margin-top: 4px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-header.clickable {
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background 0.15s;
}

.section-header.clickable:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.section-toggle {
  font-size: 11px;
  opacity: 0.7;
}

.section-body {
  padding-bottom: 4px;
}

/* 底部 */
.sidebar-footer {
  flex-shrink: 0;
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
