//! useAiAssist — AI Assistant composable
//!
//! Provides three AI actions (explain / summarize / insights) that each:
//!   1. Call the corresponding Tauri command and get back a task_id
//!   2. Listen to task-scoped `ai-stream` / `ai-done` / `ai-error` events
//!   3. Accumulate streamed content into reactive AiTask state
//!
//! Multiple tasks can run concurrently — each is tracked by its task_id.

import { ref, reactive, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  AiTask,
  AiTaskType,
  AiStreamChunk,
  AiDonePayload,
  AiErrorPayload,
  AiAvailability,
  AiSummaryStyle,
  AiInsightFocus,
} from '../types'

// ── Shared event listeners (singleton per page mount) ─────────────────────────

// Map: task_id → AiTask reactive ref
const taskMap = reactive<Record<string, AiTask>>({})

let unlistenStream: UnlistenFn | null = null
let unlistenDone: UnlistenFn | null = null
let unlistenError: UnlistenFn | null = null
let refCount = 0

async function attachListeners() {
  if (refCount++ > 0) return

  unlistenStream = await listen<AiStreamChunk>('ai-stream', (event) => {
    const task = taskMap[event.payload.task_id]
    if (task) {
      task.content += event.payload.content
    }
  })

  unlistenDone = await listen<AiDonePayload>('ai-done', (event) => {
    const task = taskMap[event.payload.task_id]
    if (task) {
      task.streaming = false
      task.completed_at = new Date().toISOString()
    }
  })

  unlistenError = await listen<AiErrorPayload>('ai-error', (event) => {
    const task = taskMap[event.payload.task_id]
    if (task) {
      task.streaming = false
      task.error = event.payload.message
      task.completed_at = new Date().toISOString()
    }
  })
}

function detachListeners() {
  if (--refCount > 0) return
  unlistenStream?.()
  unlistenDone?.()
  unlistenError?.()
  unlistenStream = null
  unlistenDone = null
  unlistenError = null
}

// ── Public composable ─────────────────────────────────────────────────────────

export function useAiAssist() {
  /** All tasks started in this component instance */
  const localTaskIds = ref<string[]>([])

  // Attach shared listeners on first use
  attachListeners()
  onUnmounted(() => {
    detachListeners()
    // Remove local task entries to avoid memory leak
    for (const id of localTaskIds.value) {
      delete taskMap[id]
    }
  })

  /** Get reactive task state by id */
  function getTask(task_id: string): AiTask | undefined {
    return taskMap[task_id]
  }

  /** Convenience: the most recently started task in this component */
  const latestTaskId = ref<string | null>(null)
  function latestTask(): AiTask | null {
    return latestTaskId.value ? (taskMap[latestTaskId.value] ?? null) : null
  }

  /** Register a new task and return its reactive entry */
  function createTask(task_id: string, type: AiTaskType): AiTask {
    const task: AiTask = reactive({
      task_id,
      type,
      streaming: true,
      content: '',
      error: null,
      started_at: new Date().toISOString(),
      completed_at: null,
    })
    taskMap[task_id] = task
    localTaskIds.value.push(task_id)
    latestTaskId.value = task_id
    return task
  }

  // ── Action: explain term ────────────────────────────────────────────────────

  async function explainTerm(opts: {
    term: string
    paper_id?: string
    context_text?: string
    lang?: 'zh' | 'en'
  }): Promise<AiTask> {
    const task_id = await invoke<string>('ai_explain_term', {
      input: {
        term: opts.term,
        paper_id: opts.paper_id ?? null,
        context_text: opts.context_text ?? null,
        lang: opts.lang ?? 'zh',
      },
    })
    return createTask(task_id, 'explain')
  }

  // ── Action: summarize paper ─────────────────────────────────────────────────

  async function summarizePaper(opts: {
    paper_id: string
    style?: AiSummaryStyle
    lang?: 'zh' | 'en'
  }): Promise<AiTask> {
    const task_id = await invoke<string>('ai_summarize', {
      input: {
        paper_id: opts.paper_id,
        style: opts.style ?? 'structured',
        lang: opts.lang ?? 'zh',
      },
    })
    return createTask(task_id, 'summarize')
  }

  // ── Action: extract insights ────────────────────────────────────────────────

  async function extractInsights(opts: {
    paper_id: string
    focus?: AiInsightFocus
    lang?: 'zh' | 'en'
  }): Promise<AiTask> {
    const task_id = await invoke<string>('ai_extract_insights', {
      input: {
        paper_id: opts.paper_id,
        focus: opts.focus ?? 'all',
        lang: opts.lang ?? 'zh',
      },
    })
    return createTask(task_id, 'insights')
  }

  // ── Action: check availability ──────────────────────────────────────────────

  async function checkAvailability(): Promise<AiAvailability> {
    return invoke<AiAvailability>('ai_check_availability')
  }

  // ── Utility: cancel / clear a task ─────────────────────────────────────────

  function clearTask(task_id: string) {
    const task = taskMap[task_id]
    if (task) {
      task.streaming = false
      delete taskMap[task_id]
      const idx = localTaskIds.value.indexOf(task_id)
      if (idx >= 0) localTaskIds.value.splice(idx, 1)
      if (latestTaskId.value === task_id) {
        latestTaskId.value = localTaskIds.value[localTaskIds.value.length - 1] ?? null
      }
    }
  }

  function clearAllTasks() {
    for (const id of [...localTaskIds.value]) {
      clearTask(id)
    }
  }

  return {
    // Task map (reactive, all tasks)
    taskMap,
    // Local task list for this component
    localTaskIds,
    latestTask,
    getTask,
    // Actions
    explainTerm,
    summarizePaper,
    extractInsights,
    checkAvailability,
    // Utilities
    clearTask,
    clearAllTasks,
  }
}
