//! Note links composable — bidirectional link management + [[link autocomplete]]

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { NoteLink, NoteLinkTarget, NoteWithLinks, NoteSearchResult } from '../types'

/**
 * Manages note bidirectional links and [[link autocomplete]].
 * Each component creates its own instance; state is not shared globally.
 */
export function useNoteLinks() {
  const loading = ref(false)
  const currentWithLinks = ref<NoteWithLinks | null>(null)
  const outgoingLinks = ref<NoteLinkTarget[]>([])
  const backlinks = ref<NoteLinkTarget[]>([])

  // [[link autocomplete]] state
  const suggestions = ref<NoteLinkTarget[]>([])
  const suggestionsLoading = ref(false)

  // Global note search
  const searchResults = ref<NoteSearchResult | null>(null)
  const searchLoading = ref(false)

  /**
   * Get a note with its full link context (outgoing + backlinks).
   */
  async function fetchWithLinks(noteId: string) {
    loading.value = true
    try {
      currentWithLinks.value = await invoke<NoteWithLinks>('get_note_with_links', { id: noteId })
      outgoingLinks.value = currentWithLinks.value.outgoing_links
      backlinks.value = currentWithLinks.value.backlinks
    } catch (e) {
      console.error('[useNoteLinks] fetchWithLinks failed:', e)
      currentWithLinks.value = null
      outgoingLinks.value = []
      backlinks.value = []
    } finally {
      loading.value = false
    }
  }

  /**
   * Create a directed link from source to target.
   */
  async function createLink(sourceId: string, targetId: string, label?: string) {
    try {
      const link = await invoke<NoteLink>('create_note_link', {
        input: { source_id: sourceId, target_id: targetId, label: label || null }
      })
      // Refresh link lists
      await fetchWithLinks(sourceId)
      return link
    } catch (e) {
      console.error('[useNoteLinks] createLink failed:', e)
      throw e
    }
  }

  /**
   * Delete a directed link.
   */
  async function deleteLink(sourceId: string, targetId: string) {
    try {
      await invoke('delete_note_link', { sourceId, targetId })
      // Refresh link lists
      if (currentWithLinks.value) {
        await fetchWithLinks(currentWithLinks.value.note.id)
      }
    } catch (e) {
      console.error('[useNoteLinks] deleteLink failed:', e)
      throw e
    }
  }

  /**
   * Fetch only backlinks for a note (lighter weight than fetchWithLinks).
   */
  async function fetchBacklinks(noteId: string) {
    try {
      backlinks.value = await invoke<NoteLinkTarget[]>('list_backlinks', { noteId })
    } catch (e) {
      console.error('[useNoteLinks] fetchBacklinks failed:', e)
      backlinks.value = []
    }
  }

  /**
   * [[link autocomplete]] — search notes by title prefix.
   * Called while the user types inside [[...]].
   */
  async function fetchSuggestions(prefix: string) {
    if (!prefix.trim()) {
      suggestions.value = []
      return
    }
    suggestionsLoading.value = true
    try {
      suggestions.value = await invoke<NoteLinkTarget[]>('note_link_suggest', {
        prefix,
        limit: 10,
      })
    } catch (e) {
      console.error('[useNoteLinks] fetchSuggestions failed:', e)
      suggestions.value = []
    } finally {
      suggestionsLoading.value = false
    }
  }

  function clearSuggestions() {
    suggestions.value = []
  }

  /**
   * Global note search (FTS5 with BM25 ranking).
   */
  async function searchNotes(query: string, opts?: { paperId?: string; noteType?: string; limit?: number; offset?: number }) {
    if (!query.trim()) {
      searchResults.value = null
      return
    }
    searchLoading.value = true
    try {
      searchResults.value = await invoke<NoteSearchResult>('search_notes', {
        query,
        paperId: opts?.paperId || null,
        noteType: opts?.noteType || null,
        limit: opts?.limit || 20,
        offset: opts?.offset || 0,
      })
    } catch (e) {
      console.error('[useNoteLinks] searchNotes failed:', e)
      searchResults.value = null
    } finally {
      searchLoading.value = false
    }
  }

  function clearSearch() {
    searchResults.value = null
  }

  return {
    // State
    loading,
    currentWithLinks,
    outgoingLinks,
    backlinks,
    suggestions,
    suggestionsLoading,
    searchResults,
    searchLoading,
    // Actions
    fetchWithLinks,
    createLink,
    deleteLink,
    fetchBacklinks,
    fetchSuggestions,
    clearSuggestions,
    searchNotes,
    clearSearch,
  }
}
