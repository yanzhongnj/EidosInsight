//! Note store

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Note, CreateNoteInput, UpdateNoteInput } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

export const useNoteStore = defineStore('note', () => {
  const cmd = useTauriCommand()

  const notes = ref<Note[]>([])
  const currentNote = ref<Note | null>(null)
  const loading = ref(false)

  async function fetchNotes(paperId?: string) {
    loading.value = true
    try {
      notes.value = await cmd.listNotes(paperId, 50, 0)
    } catch (error) {
      console.error('Failed to fetch notes:', error)
    } finally {
      loading.value = false
    }
  }

  async function createNote(input: CreateNoteInput) {
    try {
      const note = await cmd.createNote(input as unknown as Record<string, unknown>)
      notes.value.unshift(note)
      return note
    } catch (error) {
      console.error('Failed to create note:', error)
      throw error
    }
  }

  async function updateNote(input: UpdateNoteInput) {
    try {
      const updated = await cmd.updateNote(input as unknown as Record<string, unknown>)
      const idx = notes.value.findIndex(n => n.id === updated.id)
      if (idx !== -1) notes.value[idx] = updated
      if (currentNote.value?.id === updated.id) currentNote.value = updated
      return updated
    } catch (error) {
      console.error('Failed to update note:', error)
      throw error
    }
  }

  async function deleteNote(id: string) {
    try {
      await cmd.deleteNote(id)
      notes.value = notes.value.filter(n => n.id !== id)
      if (currentNote.value?.id === id) currentNote.value = null
    } catch (error) {
      console.error('Failed to delete note:', error)
      throw error
    }
  }

  return {
    notes, currentNote, loading,
    fetchNotes, createNote, updateNote, deleteNote,
  }
})
