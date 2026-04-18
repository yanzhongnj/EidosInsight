//! Paper store

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Paper, PaperFilter, CreatePaperInput, UpdatePaperInput } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

export const usePaperStore = defineStore('paper', () => {
  const cmd = useTauriCommand()

  const papers = ref<Paper[]>([])
  const totalPapers = ref(0)
  const currentPaper = ref<Paper | null>(null)
  const loading = ref(false)
  const filter = ref<PaperFilter>({})

  const unreadCount = computed(() => papers.value.filter(p => p.reading_status === 'unread').length)
  const favoriteCount = computed(() => papers.value.filter(p => p.is_favorite).length)

  async function fetchPapers(newFilter?: PaperFilter) {
    loading.value = true
    try {
      if (newFilter) filter.value = { ...filter.value, ...newFilter }
      const result = await cmd.listPapers(filter.value)
      papers.value = result.papers
      totalPapers.value = result.total
    } catch (error) {
      console.error('Failed to fetch papers:', error)
    } finally {
      loading.value = false
    }
  }

  async function fetchPaper(id: string) {
    loading.value = true
    try {
      currentPaper.value = await cmd.getPaper(id)
    } catch (error) {
      console.error('Failed to fetch paper:', error)
    } finally {
      loading.value = false
    }
  }

  async function importPaper(input: CreatePaperInput) {
    try {
      const paper = await cmd.importPaper(input as unknown as Record<string, unknown>)
      papers.value.unshift(paper)
      totalPapers.value++
      return paper
    } catch (error) {
      console.error('Failed to import paper:', error)
      throw error
    }
  }

  async function importPdf() {
    try {
      const paper = await cmd.importPdf()
      papers.value.unshift(paper)
      totalPapers.value++
      return paper
    } catch (error) {
      console.error('Failed to import PDF:', error)
      throw error
    }
  }

  async function importPdfFromPath(sourcePath: string) {
    try {
      const paper = await cmd.importPdfFromPath(sourcePath)
      papers.value.unshift(paper)
      totalPapers.value++
      return paper
    } catch (error) {
      console.error('Failed to import PDF from path:', error)
      throw error
    }
  }

  async function batchImportPdfs() {
    try {
      const imported = await cmd.batchImportPdfs()
      papers.value = [...imported, ...papers.value]
      totalPapers.value += imported.length
      return imported
    } catch (error) {
      console.error('Failed to batch import PDFs:', error)
      throw error
    }
  }

  async function updatePaper(input: UpdatePaperInput) {
    try {
      const updated = await cmd.updatePaper(input as unknown as Record<string, unknown>)
      const idx = papers.value.findIndex(p => p.id === updated.id)
      if (idx !== -1) papers.value[idx] = updated
      if (currentPaper.value?.id === updated.id) currentPaper.value = updated
      return updated
    } catch (error) {
      console.error('Failed to update paper:', error)
      throw error
    }
  }

  async function deletePaper(id: string) {
    try {
      await cmd.deletePaper(id)
      papers.value = papers.value.filter(p => p.id !== id)
      totalPapers.value--
      if (currentPaper.value?.id === id) currentPaper.value = null
    } catch (error) {
      console.error('Failed to delete paper:', error)
      throw error
    }
  }

  async function toggleFavorite(paper: Paper) {
    return updatePaper({ id: paper.id, is_favorite: !paper.is_favorite })
  }

  async function setReadingStatus(paperId: string, status: 'unread' | 'reading' | 'read') {
    return updatePaper({ id: paperId, reading_status: status })
  }

  async function readPdfFile(paperId: string) {
    try {
      return await cmd.readPdfFile(paperId)
    } catch (error) {
      console.error('Failed to read PDF file:', error)
      throw error
    }
  }

  return {
    papers, totalPapers, currentPaper, loading, filter,
    unreadCount, favoriteCount,
    fetchPapers, fetchPaper, importPaper, importPdf, importPdfFromPath, batchImportPdfs, updatePaper, deletePaper,
    toggleFavorite, setReadingStatus, readPdfFile,
  }
})
