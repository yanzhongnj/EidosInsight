<script setup lang="ts">
import { computed } from 'vue'
import { useUpdater } from '../../composables/useUpdater'

const {
  updateStatus,
  updateInfo,
  updateError,
  currentVersion,
  checkForUpdate,
  downloadAndInstall,
  dismissUpdate,
} = useUpdater()

const statusText = computed(() => {
  switch (updateStatus.value) {
    case 'checking': return '检查中...'
    case 'available': return '发现新版本'
    case 'downloading': return '下载中...'
    case 'ready': return '更新就绪'
    case 'error': return '检查失败'
    default: return '检查更新'
  }
})

const versionText = computed(() => {
  if (!updateInfo.value) return ''
  return `v${currentVersion.value} → v${updateInfo.value.version}`
})
</script>

<template>
  <div class="update-checker">
    <div v-if="updateStatus === 'available' && updateInfo" class="update-banner">
      <div class="update-info">
        <span class="update-icon">🔄</span>
        <div class="update-text">
          <span class="update-title">发现新版本 v{{ updateInfo.version }}</span>
          <span class="update-detail">{{ versionText }}</span>
        </div>
      </div>
      <div class="update-actions">
        <button class="btn btn-primary btn-sm" @click="downloadAndInstall">
          下载更新
        </button>
        <button class="btn btn-ghost btn-sm" @click="dismissUpdate">
          稍后提醒
        </button>
      </div>
      <div v-if="updateInfo.body" class="release-notes">
        <details>
          <summary>更新内容</summary>
          <div class="notes-content">{{ updateInfo.body }}</div>
        </details>
      </div>
    </div>

    <div v-else-if="updateStatus === 'error'" class="update-error">
      <span class="error-text">{{ updateError || '检查更新失败' }}</span>
      <button class="btn btn-ghost btn-sm" @click="checkForUpdate">重试</button>
    </div>

    <div v-else class="update-row">
      <span class="current-version">当前版本：v{{ currentVersion }}</span>
      <button
        class="btn btn-ghost btn-sm"
        @click="checkForUpdate"
        :disabled="updateStatus === 'checking'"
      >
        {{ statusText }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.update-checker {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--border-primary);
}

.update-banner {
  background: color-mix(in srgb, var(--color-primary) 8%, var(--bg-secondary));
  border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
  border-radius: var(--radius-md);
  padding: 12px 16px;
}

.update-info {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.update-icon {
  font-size: 20px;
}

.update-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.update-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.update-detail {
  font-size: 12px;
  color: var(--text-tertiary);
}

.update-actions {
  display: flex;
  gap: 8px;
}

.release-notes {
  margin-top: 10px;
}

.release-notes summary {
  font-size: 12px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px 0;
}

.notes-content {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  max-height: 150px;
  overflow-y: auto;
  padding: 8px 0;
  white-space: pre-wrap;
}

.update-error {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--color-danger) 8%, var(--bg-secondary));
  border-radius: var(--radius-md);
}

.error-text {
  font-size: 13px;
  color: var(--color-danger);
}

.update-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.current-version {
  font-size: 13px;
  color: var(--text-tertiary);
}

.btn-sm {
  padding: 4px 12px;
  font-size: 12px;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s;
}

.btn-primary:hover {
  filter: brightness(1.1);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  padding: 4px 12px;
  font-size: 12px;
  transition: all 0.15s;
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn-ghost:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
