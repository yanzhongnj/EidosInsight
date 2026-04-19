//! Paper store

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Paper, PaperFilter, CreatePaperInput, UpdatePaperInput, Tag } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

import { useAppStore } from './app'

export const usePaperStore = defineStore('paper', () => {
  const cmd = useTauriCommand()
  const appStore = useAppStore()

  const papers = ref<Paper[]>([])
  const totalPapers = ref(0)
  const currentPaper = ref<Paper | null>(null)
  const loading = ref(false)
  const filter = ref<PaperFilter>({})

  // Paper tags cache: paper_id → Tag[]
  const paperTagsMap = ref<Record<string, Tag[]>>({})

  async function fetchPapers(newFilter?: PaperFilter, resetFilter = false) {
    loading.value = true
    try {
      if (newFilter) {
        if (resetFilter) {
          filter.value = { ...newFilter }
        } else {
          filter.value = { ...filter.value, ...newFilter }
        }
      }
      const result = await cmd.listPapers(filter.value)
      papers.value = result.papers
      totalPapers.value = result.total
      // Batch load tags for the fetched papers
      await loadPaperTags(result.papers.map(p => p.id))
    } catch (error) {
      console.error('Failed to fetch papers:', error)
    } finally {
      loading.value = false
    }
  }

  async function loadPaperTags(paperIds: string[]) {
    if (paperIds.length === 0) return
    try {
      const tagsMap = await cmd.batchGetPaperTags(paperIds)
      paperTagsMap.value = { ...paperTagsMap.value, ...tagsMap }
    } catch (e) {
      console.error('Failed to load paper tags:', e)
    }
  }

  function getPaperTags(paperId: string): Tag[] {
    return paperTagsMap.value[paperId] ?? []
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
      appStore.fetchLibraryStats()
      return paper
    } catch (error) {
      console.error('Failed to import paper:', error)
      throw error
    }
  }

  async function importPdf() {
    try {
      const paper = await cmd.importPdf(appStore.selectedFolderId)
      papers.value.unshift(paper)
      totalPapers.value++
      appStore.fetchLibraryStats()
      return paper
    } catch (error) {
      console.error('Failed to import PDF:', error)
      throw error
    }
  }

  async function importPdfFromPath(sourcePath: string) {
    try {
      const paper = await cmd.importPdfFromPath(sourcePath, appStore.selectedFolderId)
      papers.value.unshift(paper)
      totalPapers.value++
      appStore.fetchLibraryStats()
      return paper
    } catch (error) {
      console.error('Failed to import PDF from path:', error)
      throw error
    }
  }

  async function batchImportPdfs() {
    try {
      const imported = await cmd.batchImportPdfs(appStore.selectedFolderId)
      papers.value = [...imported, ...papers.value]
      totalPapers.value += imported.length
      appStore.fetchLibraryStats()
      return imported
    } catch (error) {
      console.error('Failed to batch import PDFs:', error)
      throw error
    }
  }

  // Multi-format document import
  async function importDocument() {
    try {
      const paper = await cmd.importDocument(appStore.selectedFolderId)
      papers.value.unshift(paper)
      totalPapers.value++
      appStore.fetchLibraryStats()
      return paper
    } catch (error) {
      console.error('Failed to import document:', error)
      throw error
    }
  }

  async function importDocumentFromPath(sourcePath: string) {
    try {
      const paper = await cmd.importDocumentFromPath(sourcePath, appStore.selectedFolderId)
      papers.value.unshift(paper)
      totalPapers.value++
      appStore.fetchLibraryStats()
      return paper
    } catch (error) {
      console.error('Failed to import document from path:', error)
      throw error
    }
  }

  async function batchImportDocuments() {
    try {
      const imported = await cmd.batchImportDocuments(appStore.selectedFolderId)
      papers.value = [...imported, ...papers.value]
      totalPapers.value += imported.length
      appStore.fetchLibraryStats()
      return imported
    } catch (error) {
      console.error('Failed to batch import documents:', error)
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
      appStore.fetchLibraryStats() // refresh global counts
      appStore.fetchFolders() // refresh folder paper counts
    } catch (error) {
      console.error('Failed to delete paper:', error)
      throw error
    }
  }

  async function movePaperToFolder(paperId: string, folderId: string | null) {
    try {
      const updated = await cmd.movePaperToFolder(paperId, folderId)
      const idx = papers.value.findIndex(p => p.id === updated.id)
      if (idx !== -1) papers.value[idx] = updated
      if (currentPaper.value?.id === updated.id) currentPaper.value = updated
      appStore.fetchLibraryStats() // refresh global counts
      appStore.fetchFolders() // refresh folder paper counts
      // Re-fetch papers to ensure filtered view stays consistent
      // (e.g. moving out of a folder should remove it from folder-filtered view)
      await fetchPapers()
      return updated
    } catch (error) {
      console.error('Failed to move paper to folder:', error)
      throw error
    }
  }

  async function toggleFavorite(paper: Paper) {
    const result = await updatePaper({ id: paper.id, is_favorite: !paper.is_favorite })
    appStore.fetchLibraryStats() // refresh global counts
    return result
  }

  async function setReadingStatus(paperId: string, status: 'unread' | 'reading' | 'read') {
    const result = await updatePaper({ id: paperId, reading_status: status })
    appStore.fetchLibraryStats() // refresh global counts
    return result
  }

  async function readPdfFile(paperId: string) {
    try {
      return await cmd.readPdfFile(paperId)
    } catch (error) {
      console.error('Failed to read PDF file:', error)
      throw error
    }
  }

  async function readDocumentFile(paperId: string) {
    try {
      return await cmd.readDocumentFile(paperId)
    } catch (error) {
      console.error('Failed to read document file:', error)
      throw error
    }
  }

  return {
    papers, totalPapers, currentPaper, loading, filter, paperTagsMap,
    fetchPapers, fetchPaper, importPaper, importPdf, importPdfFromPath, batchImportPdfs,
    importDocument, importDocumentFromPath, batchImportDocuments,
    updatePaper, deletePaper, movePaperToFolder, getPaperTags,
    toggleFavorite, setReadingStatus, readPdfFile, readDocumentFile,
  }
})
