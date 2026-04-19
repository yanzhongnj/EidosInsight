<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useTauriCommand } from '../composables/useTauriCommand'
import { usePaperStore } from '../stores/paper'
import { useAppStore } from '../stores/app'
import { useCollectionStore } from '../stores/collection'
import type { LibraryStats } from '../types'

const router = useRouter()
const paperStore = usePaperStore()
const appStore = useAppStore()
const collectionStore = useCollectionStore()
const { getLibraryStats } = useTauriCommand()

const stats = ref<LibraryStats | null>(null)
const loading = ref(true)

// Recent papers (last 5 added)
const recentPapers = computed(() => paperStore.papers.slice(0, 5))

onMounted(async () => {
  try {
    stats.value = await getLibraryStats()
    // Also fetch recent papers for display
    await paperStore.fetchPapers({ sort_by: 'date_added', sort_order: 'desc', limit: 5 })
    await collectionStore.fetchCollections()
  } catch (e) {
    console.error('Failed to load dashboard stats:', e)
  } finally {
    loading.value = false
  }
})

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
  } catch {
    return dateStr
  }
}

function goToLibrary() {
  appStore.activeView = 'library'
  appStore.selectedFolderId = null
  paperStore.fetchPapers({}, true)
  router.push('/library')
}

function goToUnread() {
  appStore.activeView = 'library'
  appStore.selectedFolderId = null
  paperStore.fetchPapers({ reading_status: 'unread' }, true)
  router.push('/library')
}

function goToRead() {
  appStore.activeView = 'library'
  appStore.selectedFolderId = null
  paperStore.fetchPapers({ reading_status: 'read' }, true)
  router.push('/library')
}

function goToFavorites() {
  appStore.activeView = 'library'
  appStore.selectedFolderId = null
  paperStore.fetchPapers({ is_favorite: true }, true)
  router.push('/library')
}

function goToCollections() {
  router.push('/collections')
}

function goToPaper(id: string) {
  router.push(`/paper/${id}`)
}

function openImport() {
  // Navigate to library and trigger import
  paperStore.importDocument().then(() => {
    paperStore.fetchPapers({}, true)
    // Refresh stats
    getLibraryStats().then(s => { stats.value = s }).catch(() => {})
  }).catch(e => console.error('Import failed:', e))
}

// Reading progress percentage
const readingProgress = computed(() => {
  if (!stats.value || stats.value.total_papers === 0) return 0
  return Math.round((stats.value.read_count / stats.value.total_papers) * 100)
})
</script>

<template>
  <div class="dashboard-view">
    <div class="dashboard-header">
      <h1 class="page-title">EidosInsight</h1>
      <div class="header-actions">
        <button class="btn btn-primary" @click="openImport">+ 导入文献</button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <template v-else>
      <!-- Stats cards row -->
      <div class="stats-grid">
        <div class="stat-card primary" @click="goToLibrary">
          <div class="stat-icon">📚</div>
          <div class="stat-info">
            <div class="stat-value">{{ stats?.total_papers ?? 0 }}</div>
            <div class="stat-label">文献总量</div>
          </div>
          <div v-if="stats?.recent_papers_count" class="stat-badge">
            +{{ stats.recent_papers_count }} 近7天
          </div>
        </div>

        <div class="stat-card" @click="goToUnread">
          <div class="stat-icon">📖</div>
          <div class="stat-info">
            <div class="stat-value">{{ stats?.unread_count ?? 0 }}</div>
            <div class="stat-label">未读</div>
          </div>
        </div>

        <div class="stat-card" @click="goToRead">
          <div class="stat-icon">✅</div>
          <div class="stat-info">
            <div class="stat-value">{{ stats?.read_count ?? 0 }}</div>
            <div class="stat-label">已读</div>
          </div>
        </div>

        <div class="stat-card" @click="goToCollections">
          <div class="stat-icon">🔬</div>
          <div class="stat-info">
            <div class="stat-value">{{ stats?.collection_count ?? 0 }}</div>
            <div class="stat-label">智能分析</div>
          </div>
        </div>
      </div>

      <!-- Reading progress bar -->
      <div v-if="stats && stats.total_papers > 0" class="progress-section">
        <div class="progress-header">
          <span class="progress-title">阅读进度</span>
          <span class="progress-text">{{ stats.read_count }} / {{ stats.total_papers }} 篇已读 ({{ readingProgress }}%)</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: readingProgress + '%' }"></div>
        </div>
      </div>

      <!-- Two-column layout -->
      <div class="dashboard-columns">
        <!-- Left: Recent papers -->
        <div class="dashboard-section">
          <div class="section-header">
            <h2>最近添加</h2>
            <button class="btn-link" @click="goToLibrary">查看全部</button>
          </div>

          <div v-if="recentPapers.length === 0" class="empty-section">
            <p>还没有文献，点击右上角导入你的第一篇文献</p>
          </div>

          <div v-else class="recent-list">
            <div
              v-for="paper in recentPapers"
              :key="paper.id"
              class="recent-item"
              @click="goToPaper(paper.id)"
            >
              <div class="recent-icon">
                <span v-if="paper.file_type === 'pdf'">📄</span>
                <span v-else-if="paper.file_type === 'docx'">📝</span>
                <span v-else-if="paper.file_type === 'txt'">📃</span>
                <span v-else>📎</span>
              </div>
              <div class="recent-content">
                <div class="recent-title">{{ paper.title }}</div>
                <div class="recent-meta">
                  <span v-if="paper.authors" class="recent-authors">{{ paper.authors }}</span>
                  <span class="recent-date">{{ formatDate(paper.date_added) }}</span>
                  <span :class="['status-dot', `status-${paper.reading_status}`]"></span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Right: Quick stats + tags -->
        <div class="dashboard-section">
          <div class="section-header">
            <h2>文献库概览</h2>
          </div>

          <div v-if="stats" class="overview-grid">
            <div class="overview-item">
              <span class="overview-value">{{ stats.monthly_papers_count }}</span>
              <span class="overview-label">近30天新增</span>
            </div>
            <div class="overview-item clickable" @click="goToFavorites">
              <span class="overview-value">{{ stats.favorite_count }}</span>
              <span class="overview-label">收藏</span>
            </div>
            <div class="overview-item clickable" @click="goToLibrary">
              <span class="overview-value">{{ stats.folder_count }}</span>
              <span class="overview-label">专题</span>
            </div>
            <div class="overview-item">
              <span class="overview-value">{{ stats.tag_count }}</span>
              <span class="overview-label">标签</span>
            </div>
            <div class="overview-item">
              <span class="overview-value">{{ stats.note_count }}</span>
              <span class="overview-label">笔记</span>
            </div>
            <div class="overview-item">
              <span class="overview-value">{{ stats.reading_count }}</span>
              <span class="overview-label">在读</span>
            </div>
          </div>

          <!-- File type distribution -->
          <div v-if="stats && stats.file_type_distribution.length > 0" class="file-types">
            <h3 class="sub-title">文件类型分布</h3>
            <div class="file-type-list">
              <div v-for="ft in stats.file_type_distribution" :key="ft.file_type" class="file-type-item">
                <span class="ft-name">{{ ft.file_type.toUpperCase() }}</span>
                <div class="ft-bar">
                  <div
                    class="ft-fill"
                    :style="{ width: (stats.total_papers > 0 ? (ft.count / stats.total_papers * 100) : 0) + '%' }"
                  ></div>
                </div>
                <span class="ft-count">{{ ft.count }}</span>
              </div>
            </div>
          </div>

          <!-- Top tags -->
          <div v-if="stats && stats.top_tags.length > 0" class="top-tags">
            <h3 class="sub-title">热门标签</h3>
            <div class="tag-list">
              <span v-for="tag in stats.top_tags" :key="tag.name" class="tag-item">
                {{ tag.name }} <small>({{ tag.count }})</small>
              </span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.dashboard-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
  padding-bottom: 40px;
}

.dashboard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

/* Stats grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-card:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-sm);
}

.stat-card.primary {
  background: linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 10%, var(--bg-card)), var(--bg-card));
  border-color: color-mix(in srgb, var(--color-primary) 30%, transparent);
}

.stat-icon {
  font-size: 28px;
  flex-shrink: 0;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.stat-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 9999px;
  background: color-mix(in srgb, var(--color-primary) 15%, transparent);
  color: var(--color-primary);
  font-weight: 500;
}

/* Progress */
.progress-section {
  margin-bottom: 24px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: 14px 16px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.progress-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.progress-text {
  font-size: 12px;
  color: var(--text-tertiary);
}

.progress-bar {
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 9999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 9999px;
  transition: width 0.5s ease;
}

/* Two columns */
.dashboard-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.dashboard-section {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.btn-link {
  background: none;
  border: none;
  color: var(--color-primary);
  cursor: pointer;
  font-size: 13px;
  padding: 0;
}

.btn-link:hover {
  text-decoration: underline;
}

/* Recent list */
.empty-section {
  padding: 20px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 14px;
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background 0.15s;
}

.recent-item:hover {
  background: var(--bg-hover);
}

.recent-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.recent-content {
  flex: 1;
  min-width: 0;
}

.recent-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.recent-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.recent-authors {
  max-width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.status-unread {
  background: var(--color-primary);
}

.status-dot.status-reading {
  background: #f59e0b;
}

.status-dot.status-read {
  background: #22c55e;
}

/* Overview grid */
.overview-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-bottom: 16px;
}

.overview-item {
  text-align: center;
  padding: 10px 4px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.overview-item.clickable {
  cursor: pointer;
  transition: background 0.15s;
}

.overview-item.clickable:hover {
  background: var(--bg-hover);
}

.overview-value {
  display: block;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
}

.overview-label {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

/* Sub title */
.sub-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 12px 0 8px;
}

/* File type distribution */
.file-type-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.file-type-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.ft-name {
  width: 40px;
  color: var(--text-secondary);
  font-weight: 500;
}

.ft-bar {
  flex: 1;
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 9999px;
  overflow: hidden;
}

.ft-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 9999px;
  min-width: 2px;
  transition: width 0.3s;
}

.ft-count {
  width: 24px;
  text-align: right;
  color: var(--text-tertiary);
}

/* Top tags */
.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag-item {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 9999px;
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
}

.tag-item small {
  opacity: 0.7;
}

/* Loading */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  color: var(--text-tertiary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Buttons */
.btn {
  padding: 8px 20px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.btn-primary:hover {
  filter: brightness(1.1);
}

/* Responsive */
@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .dashboard-columns {
    grid-template-columns: 1fr;
  }
}
</style>
