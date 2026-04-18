//! Composable for LLM streaming via Tauri Events

import { ref, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { LlmStreamChunk } from '../types'

interface StreamErrorPayload {
  message: string
}

export function useAiStream() {
  const streaming = ref(false)
  const content = ref('')
  const error = ref<string | null>(null)

  let unlistenStream: UnlistenFn | null = null
  let unlistenDone: UnlistenFn | null = null
  let unlistenError: UnlistenFn | null = null

  async function startStream() {
    // Reset state
    content.value = ''
    error.value = null
    streaming.value = true

    // Listen for stream chunks
    unlistenStream = await listen<LlmStreamChunk>('llm-stream', (event) => {
      content.value += event.payload.content
    })

    // Listen for stream done
    unlistenDone = await listen<void>('llm-done', () => {
      streaming.value = false
    })

    // Listen for stream errors
    unlistenError = await listen<StreamErrorPayload>('llm-error', (event) => {
      error.value = event.payload.message
      streaming.value = false
    })
  }

  function stopStream() {
    streaming.value = false
    cleanup()
  }

  function cleanup() {
    unlistenStream?.()
    unlistenDone?.()
    unlistenError?.()
    unlistenStream = null
    unlistenDone = null
    unlistenError = null
  }

  // Auto-cleanup on unmount
  onUnmounted(() => {
    cleanup()
  })

  return {
    streaming,
    content,
    error,
    startStream,
    stopStream,
  }
}
