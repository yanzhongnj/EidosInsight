//! Auto-update composable — GitHub Releases based update checker.
//!
//! Uses the backend `check_for_updates` command which queries GitHub Releases API.
//! When an update is found, the user is shown a notification and can choose to
//! download the installer (opens in browser) or dismiss.
//!
//! Future: migrate to tauri-plugin-updater for in-app auto-install once signing keys are ready.

import { ref, onMounted, onUnmounted } from 'vue'
import { useTauriCommand } from './useTauriCommand'
import type { UpdateStatus, UpdateProgress, UpdateInfo } from '../types'

export interface UpdateCheckResult {
  has_update: boolean
  current_version: string
  latest_version: string
  published_at: string | null
  release_notes: string | null
  download_url: string | null
  html_url: string | null
}

export function useUpdater() {
  const cmd = useTauriCommand()

  const updateStatus = ref<UpdateStatus>('idle')
  const updateInfo = ref<UpdateInfo | null>(null)
  const updateProgress = ref<UpdateProgress>({ downloaded: 0, total: null })
  const updateError = ref<string | null>(null)
  const currentVersion = ref<string>('')

  let autoCheckTimer: ReturnType<typeof setTimeout> | null = null

  onMounted(async () => {
    // Get current version
    try {
      currentVersion.value = await cmd.call<string>('get_app_version')
    } catch {
      currentVersion.value = '0.1.0'
    }
  })

  onUnmounted(() => {
    if (autoCheckTimer) {
      clearTimeout(autoCheckTimer)
    }
  })

  async function checkForUpdate(): Promise<void> {
    if (updateStatus.value === 'checking') return

    updateStatus.value = 'checking'
    updateError.value = null

    try {
      const result = await cmd.call<UpdateCheckResult>('check_for_updates')

      if (result.has_update) {
        updateInfo.value = {
          version: result.latest_version,
          date: result.published_at ?? '',
          body: result.release_notes ?? '',
        }
        updateStatus.value = 'available'
      } else {
        updateInfo.value = null
        updateStatus.value = 'idle'
      }
    } catch (e: any) {
      updateError.value = e?.toString() || '检查更新失败'
      updateStatus.value = 'error'
    }
  }

  async function downloadAndInstall(): Promise<void> {
    if (!updateInfo.value) return

    // For now, we open the download URL in the browser since we don't have
    // tauri-plugin-updater configured. The user downloads and installs manually.
    const result = await cmd.call<UpdateCheckResult>('check_for_updates').catch(() => null)

    if (result?.download_url) {
      // Open the download URL in the default browser
      try {
        const { openUrl } = await import('@tauri-apps/plugin-opener')
        await openUrl(result.download_url)
      } catch {
        // Fallback: try shell open
        window.open(result.download_url, '_blank')
      }
    } else if (result?.html_url) {
      try {
        const { openUrl } = await import('@tauri-apps/plugin-opener')
        await openUrl(result.html_url)
      } catch {
        window.open(result.html_url, '_blank')
      }
    }

    updateStatus.value = 'idle'
  }

  function dismissUpdate(): void {
    updateStatus.value = 'idle'
    updateInfo.value = null
  }

  /// Schedule an automatic check after a delay (ms). Only checks once.
  function scheduleCheck(delayMs: number = 5000): void {
    if (autoCheckTimer) clearTimeout(autoCheckTimer)
    autoCheckTimer = setTimeout(() => {
      checkForUpdate()
    }, delayMs)
  }

  return {
    updateStatus,
    updateInfo,
    updateProgress,
    updateError,
    currentVersion,
    checkForUpdate,
    downloadAndInstall,
    dismissUpdate,
    scheduleCheck,
  }
}
