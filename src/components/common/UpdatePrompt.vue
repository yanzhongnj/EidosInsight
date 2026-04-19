<script setup lang="ts">
import { useUpdater } from '../../composables/useUpdater'

const { updateStatus, updateInfo, updateProgress, updateError, downloadAndInstall, dismissUpdate } = useUpdater()

const downloadPercent = computed(() => {
  if (!updateProgress.value.total) return 0
  return Math.round((updateProgress.value.downloaded / updateProgress.value.total) * 100)
})

import { computed } from 'vue'
</script>

<template>
  <div v-if="updateStatus === 'available' && updateInfo" class="update-prompt">
    <div class="update-icon">🔄</div>
    <div class="update-content">
      <div class="update-title">发现新版本 v{{ updateInfo.version }}</div>
      <div v-if="updateInfo.body" class="update-body">{{ updateInfo.body.slice(0, 200) }}</div>
      <div v-if="updateError" class="update-error">{{ updateError }}</div>
    </div>
    <div class="update-actions">
      <button class="btn-update" @click="downloadAndInstall">立即更新</button>
      <button class="btn-dismiss" @click="dismissUpdate">稍后</button>
    </div>
  </div>

  <div v-else-if="updateStatus === 'downloading'" class="update-prompt downloading">
    <div class="update-icon">⬇️</div>
    <div class="update-content">
      <div class="update-title">正在下载更新... {{ downloadPercent }}%</div>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: downloadPercent + '%' }"></div>
      </div>
    </div>
  </div>

  <div v-else-if="updateStatus === 'ready'" class="update-prompt ready">
    <div class="update-icon">✅</div>
    <div class="update-content">
      <div class="update-title">更新已就绪</div>
      <div class="update-body">重启应用以完成安装</div>
    </div>
  </div>
</template>

<style scoped>
.update-prompt {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: color-mix(in srgb, var(--color-primary) 8%, var(--bg-secondary));
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
  border-radius: var(--radius-lg);
  margin-bottom: 16px;
}

.update-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.update-content {
  flex: 1;
  min-width: 0;
}

.update-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.update-body {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.update-error {
  font-size: 12px;
  color: #ef4444;
  margin-top: 2px;
}

.update-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.btn-update {
  padding: 5px 14px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: filter 0.15s;
}

.btn-update:hover {
  filter: brightness(1.1);
}

.btn-dismiss {
  padding: 5px 14px;
  background: transparent;
  color: var(--text-tertiary);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-md);
  font-size: 12px;
  cursor: pointer;
}

.btn-dismiss:hover {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.progress-bar {
  width: 100%;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  margin-top: 8px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 2px;
  transition: width 0.3s ease;
}
</style>
