//! App store - global application state

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AppSettings, LlmConfig, Folder, Tag, LlmStatusResult, LlmModelInfo, LibraryStats } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

export const useAppStore = defineStore('app', () => {
  const cmd = useTauriCommand()

  // App state
  const initialized = ref(false)
  const isFirstLaunch = ref(true)
  const sidebarCollapsed = ref(false)
  const activeView = ref('library')

  // Settings
  const settings = ref<AppSettings>({
    theme: 'system',
    language: 'zh-CN',
    auto_process_papers: true,
    default_llm_provider: 'ollama',
    check_updates_on_start: true,
    paper_storage_path: null,
    max_backup_count: 5,
    export_format: 'bibtex',
  })

  const llmConfig = ref<LlmConfig>({
    provider: 'ollama',
    model_name: 'llama3',
    api_base_url: 'http://localhost:11434',
    api_key_set: false,
    max_tokens: 4096,
    temperature: 0.7,
    is_local: true,
  })

  // LLM state
  const llmStatus = ref<LlmStatusResult>({
    status: 'unavailable',
    has_cloud: false,
    active_cloud_provider: null,
  })
  const llmModels = ref<LlmModelInfo[]>([])
  const llmStatusLoading = ref(false)

  // Sidebar data
  const folders = ref<Folder[]>([])
  const tags = ref<Tag[]>([])
  const selectedFolderId = ref<string | null>(null)

  // Global library stats (independent of current filter)
  const libraryStats = ref<LibraryStats | null>(null)

  const isDark = computed(() => {
    if (settings.value.theme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return settings.value.theme === 'dark'
  })

  const isLlmAvailable = computed(() => llmStatus.value.status !== 'unavailable')

  async function initialize() {
    if (initialized.value) return
    try {
      isFirstLaunch.value = await cmd.isFirstLaunch()
      settings.value = await cmd.getSettings()
      folders.value = await cmd.listFolders()
      tags.value = await cmd.listTags()
      try {
        llmConfig.value = await cmd.getLlmConfig()
      } catch {
        // LLM config may not exist yet, use defaults
      }
      // Check LLM status
      await refreshLlmStatus()
      // Fetch global library stats
      await fetchLibraryStats()
      initialized.value = true
    } catch (error) {
      console.error('Failed to initialize app:', error)
    }
  }

  async function refreshLlmStatus() {
    llmStatusLoading.value = true
    try {
      llmStatus.value = await cmd.getLlmStatus()
    } catch {
      llmStatus.value = { status: 'unavailable', has_cloud: false, active_cloud_provider: null }
    } finally {
      llmStatusLoading.value = false
    }
  }

  async function refreshLlmModels() {
    try {
      llmModels.value = await cmd.listLlmModels()
    } catch {
      llmModels.value = []
    }
  }

  async function fetchFolders() {
    try {
      folders.value = await cmd.listFolders()
    } catch (error) {
      console.error('Failed to fetch folders:', error)
    }
  }

  async function fetchTags() {
    try {
      tags.value = await cmd.listTags()
    } catch (error) {
      console.error('Failed to fetch tags:', error)
    }
  }

  async function fetchLibraryStats() {
    try {
      libraryStats.value = await cmd.getLibraryStats()
    } catch (error) {
      console.error('Failed to fetch library stats:', error)
    }
  }

  async function updateSettings(newSettings: AppSettings) {
    try {
      await cmd.updateSettings(newSettings as unknown as Record<string, unknown>)
      settings.value = newSettings
    } catch (error) {
      console.error('Failed to update settings:', error)
      throw error
    }
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  return {
    initialized, isFirstLaunch, sidebarCollapsed, activeView,
    settings, llmConfig, llmStatus, llmModels, llmStatusLoading,
    folders, tags, selectedFolderId, libraryStats, isDark, isLlmAvailable,
    initialize, refreshLlmStatus, refreshLlmModels, fetchFolders, fetchTags, fetchLibraryStats, updateSettings, toggleSidebar,
  }
})
