/**
 * useSearch — Search state management composable
 *
 * Features:
 * - Debounced suggestion fetch (200ms)
 * - Search history with localStorage persistence (max 20 entries)
 * - Advanced filter state
 * - Pagination helpers
 */

import { ref, computed, watch } from 'vue'
import { useTauriCommand } from './useTauriCommand'
import type { SearchResult, SearchFilters, SuggestResult, SuggestionItem } from '../types'

const HISTORY_KEY = 'ei_search_history'
const MAX_HISTORY = 20
const SUGGEST_DEBOUNCE_MS = 200

// ─── History helpers ────────────────────────────────────────────────────────

function loadHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

function saveHistory(hist: string[]) {
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(hist.slice(0, MAX_HISTORY)))
  } catch { /* storage full - ignore */ }
}

// ─── Composable ─────────────────────────────────────────────────────────────

export function useSearch() {
  const cmd = useTauriCommand()

  // ── Query state ──
  const query = ref('')
  const searchField = ref<'all' | 'title' | 'abstract' | 'authors'>('all')
  const readingStatusFilter = ref<'' | 'unread' | 'reading' | 'read'>('')
  const yearFrom = ref('')
  const yearTo = ref('')
  const favoritesOnly = ref(false)
  const showAdvanced = ref(false)

  // ── Results ──
  const results = ref<SearchResult | null>(null)
  const searching = ref(false)
  const searchError = ref<string | null>(null)
  const currentPage = ref(0)
  const pageSize = 20

  const totalPages = computed(() =>
    results.value ? Math.ceil(results.value.total / pageSize) : 0
  )

  // ── Suggestions ──
  const suggestions = ref<SuggestResult | null>(null)
  const suggestVisible = ref(false)
  const loadingSuggestions = ref(false)
  let suggestTimer: ReturnType<typeof setTimeout> | null = null

  // ── History ──
  const history = ref<string[]>(loadHistory())

  // Watch query for suggestions
  watch(query, (val) => {
    if (suggestTimer) clearTimeout(suggestTimer)
    if (!val.trim() || val.trim().length < 2) {
      suggestions.value = null
      suggestVisible.value = false
      return
    }
    suggestTimer = setTimeout(async () => {
      loadingSuggestions.value = true
      try {
        suggestions.value = await cmd.searchSuggest(val.trim())
        suggestVisible.value = (suggestions.value?.items.length ?? 0) > 0
      } catch {
        suggestions.value = null
      } finally {
        loadingSuggestions.value = false
      }
    }, SUGGEST_DEBOUNCE_MS)
  })

  // ── Build filters ──
  function buildFilters(page = currentPage.value): SearchFilters {
    return {
      query: query.value.trim(),
      search_field: searchField.value === 'all' ? null : searchField.value,
      reading_status: readingStatusFilter.value || null,
      year_from: yearFrom.value || null,
      year_to: yearTo.value || null,
      is_favorite: favoritesOnly.value || null,
      limit: pageSize,
      offset: page * pageSize,
    }
  }

  // ── Execute search ──
  async function doSearch(page = 0) {
    const q = query.value.trim()
    if (!q) return
    suggestVisible.value = false
    currentPage.value = page
    searching.value = true
    searchError.value = null
    try {
      results.value = await cmd.searchAdvanced(buildFilters(page))
      // Persist to history
      if (!history.value.includes(q)) {
        history.value = [q, ...history.value].slice(0, MAX_HISTORY)
        saveHistory(history.value)
      }
    } catch (err: unknown) {
      searchError.value = err instanceof Error ? err.message : String(err)
      results.value = null
    } finally {
      searching.value = false
    }
  }

  // ── Pagination ──
  function goToPage(page: number) {
    if (page < 0 || page >= totalPages.value) return
    doSearch(page)
  }

  // ── Select suggestion ──
  function selectSuggestion(item: SuggestionItem, navigate?: (id: string) => void) {
    query.value = item.title
    suggestVisible.value = false
    if (navigate) {
      navigate(item.id)
    } else {
      doSearch()
    }
  }

  // ── History ──
  function useHistoryItem(q: string) {
    query.value = q
    doSearch()
  }

  function removeHistoryItem(q: string) {
    history.value = history.value.filter(h => h !== q)
    saveHistory(history.value)
  }

  function clearHistory() {
    history.value = []
    saveHistory([])
  }

  // ── Reset filters ──
  function resetFilters() {
    searchField.value = 'all'
    readingStatusFilter.value = ''
    yearFrom.value = ''
    yearTo.value = ''
    favoritesOnly.value = false
  }

  const hasActiveFilters = computed(() =>
    searchField.value !== 'all' ||
    !!readingStatusFilter.value ||
    !!yearFrom.value ||
    !!yearTo.value ||
    favoritesOnly.value
  )

  return {
    // State
    query,
    searchField,
    readingStatusFilter,
    yearFrom,
    yearTo,
    favoritesOnly,
    showAdvanced,
    // Results
    results,
    searching,
    searchError,
    currentPage,
    pageSize,
    totalPages,
    // Suggestions
    suggestions,
    suggestVisible,
    loadingSuggestions,
    // History
    history,
    // Computed
    hasActiveFilters,
    // Actions
    doSearch,
    goToPage,
    selectSuggestion,
    useHistoryItem,
    removeHistoryItem,
    clearHistory,
    resetFilters,
  }
}
