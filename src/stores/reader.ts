import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useTauriCommand } from '../composables/useTauriCommand'
import type { ReadingState, UpdateReadingStateInput } from '../types'

export const useReaderStore = defineStore('reader', () => {
  const cmd = useTauriCommand()

  const readingState = ref<ReadingState | null>(null)
  const currentPage = ref(1)
  const totalPages = ref(0)
  const zoomLevel = ref(1.0)
  const loading = ref(false)

  async function loadReadingState(paperId: string) {
    loading.value = true
    try {
      readingState.value = await cmd.getReadingState(paperId)
      if (readingState.value) {
        currentPage.value = readingState.value.current_page || 1
        totalPages.value = readingState.value.total_pages || 0
        zoomLevel.value = readingState.value.zoom_level || 1.0
      }
    } catch {
      // No reading state yet, use defaults
      readingState.value = null
    } finally {
      loading.value = false
    }
  }

  async function saveReadingState(paperId: string) {
    const input: UpdateReadingStateInput = {
      paper_id: paperId,
      current_page: currentPage.value,
      total_pages: totalPages.value,
      zoom_level: zoomLevel.value,
    }
    try {
      readingState.value = await cmd.updateReadingState(input as unknown as Record<string, unknown>)
    } catch (error) {
      console.error('Failed to save reading state:', error)
    }
  }

  // Auto-save debounce
  let saveTimeout: ReturnType<typeof setTimeout> | null = null
  function debouncedSave(paperId: string, delay = 2000) {
    if (saveTimeout) clearTimeout(saveTimeout)
    saveTimeout = setTimeout(() => {
      saveReadingState(paperId)
    }, delay)
  }

  return {
    readingState, currentPage, totalPages, zoomLevel, loading,
    loadReadingState, saveReadingState, debouncedSave,
  }
})
