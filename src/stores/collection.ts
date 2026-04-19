//! Collection store - manages paper collections and AI results

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Collection, AiResult, CreateCollectionInput, UpdateCollectionInput, GenerateReviewInput, GenerateComparisonInput, GenerateBaselineInput } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

export const useCollectionStore = defineStore('collection', () => {
  const cmd = useTauriCommand()

  const collections = ref<Collection[]>([])
  const currentCollection = ref<Collection | null>(null)
  const collectionPapers = ref<import('../types').Paper[]>([])
  const aiResults = ref<AiResult[]>([])
  const currentAiResult = ref<AiResult | null>(null)
  const loading = ref(false)
  const generating = ref(false)

  async function fetchCollections() {
    loading.value = true
    try {
      collections.value = await cmd.listCollections()
    } catch (e) {
      console.error('Failed to fetch collections:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchCollection(id: string) {
    try {
      currentCollection.value = await cmd.getCollection(id)
    } catch (e) {
      console.error('Failed to fetch collection:', e)
    }
  }

  async function createCollection(input: CreateCollectionInput) {
    try {
      const collection = await cmd.createCollection(input)
      collections.value.push(collection)
      return collection
    } catch (e) {
      console.error('Failed to create collection:', e)
      throw e
    }
  }

  async function updateCollection(input: UpdateCollectionInput) {
    try {
      const updated = await cmd.updateCollection(input)
      const idx = collections.value.findIndex(c => c.id === updated.id)
      if (idx !== -1) collections.value[idx] = updated
      if (currentCollection.value?.id === updated.id) currentCollection.value = updated
      return updated
    } catch (e) {
      console.error('Failed to update collection:', e)
      throw e
    }
  }

  async function deleteCollection(id: string) {
    try {
      await cmd.deleteCollection(id)
      collections.value = collections.value.filter(c => c.id !== id)
      if (currentCollection.value?.id === id) currentCollection.value = null
    } catch (e) {
      console.error('Failed to delete collection:', e)
    }
  }

  async function addPaper(collectionId: string, paperId: string) {
    try {
      await cmd.addPaperToCollection(collectionId, paperId)
      // Refresh paper count
      await fetchCollection(collectionId)
    } catch (e) {
      console.error('Failed to add paper to collection:', e)
    }
  }

  async function removePaper(collectionId: string, paperId: string) {
    try {
      await cmd.removePaperFromCollection(collectionId, paperId)
      collectionPapers.value = collectionPapers.value.filter(p => p.id !== paperId)
      await fetchCollection(collectionId)
    } catch (e) {
      console.error('Failed to remove paper from collection:', e)
    }
  }

  async function fetchCollectionPapers(collectionId: string) {
    try {
      collectionPapers.value = await cmd.listCollectionPapers(collectionId)
    } catch (e) {
      console.error('Failed to fetch collection papers:', e)
    }
  }

  async function addPapersBatch(collectionId: string, paperIds: string[]) {
    try {
      const count = await cmd.addPapersToCollection(collectionId, paperIds)
      await fetchCollection(collectionId)
      return count
    } catch (e) {
      console.error('Failed to add papers batch:', e)
      throw e
    }
  }

  // AI Result operations
  async function fetchAiResults(resultType?: string) {
    try {
      aiResults.value = await cmd.listAiResults(resultType)
    } catch (e) {
      console.error('Failed to fetch AI results:', e)
    }
  }

  async function fetchCollectionAiResults(collectionId: string) {
    try {
      aiResults.value = await cmd.listCollectionAiResults(collectionId)
    } catch (e) {
      console.error('Failed to fetch collection AI results:', e)
    }
  }

  async function fetchAiResult(id: string) {
    try {
      currentAiResult.value = await cmd.getAiResult(id)
    } catch (e) {
      console.error('Failed to fetch AI result:', e)
    }
  }

  async function deleteAiResult(id: string) {
    try {
      await cmd.deleteAiResult(id)
      aiResults.value = aiResults.value.filter(r => r.id !== id)
      if (currentAiResult.value?.id === id) currentAiResult.value = null
    } catch (e) {
      console.error('Failed to delete AI result:', e)
    }
  }

  async function doGenerateReview(input: GenerateReviewInput) {
    generating.value = true
    try {
      const result = await cmd.generateReview(input)
      aiResults.value.unshift(result)
      currentAiResult.value = result
      return result
    } catch (e) {
      console.error('Failed to generate review:', e)
      throw e
    } finally {
      generating.value = false
    }
  }

  async function doGenerateComparison(input: GenerateComparisonInput) {
    generating.value = true
    try {
      const result = await cmd.generateComparison(input)
      aiResults.value.unshift(result)
      currentAiResult.value = result
      return result
    } catch (e) {
      console.error('Failed to generate comparison:', e)
      throw e
    } finally {
      generating.value = false
    }
  }

  async function doGenerateBaseline(input: GenerateBaselineInput) {
    generating.value = true
    try {
      const result = await cmd.generateBaseline(input)
      aiResults.value.unshift(result)
      currentAiResult.value = result
      return result
    } catch (e) {
      console.error('Failed to generate baseline:', e)
      throw e
    } finally {
      generating.value = false
    }
  }

  return {
    collections, currentCollection, collectionPapers,
    aiResults, currentAiResult, loading, generating,
    fetchCollections, fetchCollection, createCollection, updateCollection, deleteCollection,
    addPaper, removePaper, fetchCollectionPapers, addPapersBatch,
    fetchAiResults, fetchCollectionAiResults, fetchAiResult, deleteAiResult,
    doGenerateReview, doGenerateComparison, doGenerateBaseline,
  }
})
