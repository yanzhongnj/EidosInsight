//! Automation store - manages auto tasks and execution history

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AutoTask, TaskExecution, PresetTemplate, ScheduleConfig } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

export const useAutomationStore = defineStore('automation', () => {
  const cmd = useTauriCommand()

  const tasks = ref<AutoTask[]>([])
  const presets = ref<PresetTemplate[]>([])
  const loading = ref(false)
  const selectedTaskId = ref<string | null>(null)
  const executionHistory = ref<TaskExecution[]>([])

  async function fetchPresets() {
    try {
      presets.value = await cmd.listPresetTasks()
    } catch (e) {
      console.error('Failed to fetch presets:', e)
    }
  }

  async function fetchTasks() {
    loading.value = true
    try {
      tasks.value = await cmd.listAutoTasks()
    } catch (e) {
      console.error('Failed to fetch tasks:', e)
    } finally {
      loading.value = false
    }
  }

  async function enablePreset(presetId: string, name: string, definition: string, schedule: ScheduleConfig, config?: Record<string, unknown>) {
    try {
      const scheduleStr = JSON.stringify(schedule)
      const configStr = config ? JSON.stringify(config) : undefined
      const task = await cmd.enableTask({
        presetId,
        name,
        definition,
        schedule: scheduleStr,
        config: configStr,
      })
      tasks.value.push(task)
      return task
    } catch (e) {
      console.error('Failed to enable task:', e)
      throw e
    }
  }

  async function disableTask(id: string) {
    try {
      await cmd.disableTask(id)
      const task = tasks.value.find(t => t.id === id)
      if (task) task.is_enabled = false
    } catch (e) {
      console.error('Failed to disable task:', e)
    }
  }

  async function updateSchedule(id: string, schedule: ScheduleConfig) {
    try {
      const scheduleStr = JSON.stringify(schedule)
      await cmd.updateTaskSchedule(id, scheduleStr)
      await fetchTasks()
    } catch (e) {
      console.error('Failed to update schedule:', e)
    }
  }

  async function updateConfig(id: string, config: Record<string, unknown>) {
    try {
      await cmd.updateTaskConfig(id, JSON.stringify(config))
      await fetchTasks()
    } catch (e) {
      console.error('Failed to update config:', e)
    }
  }

  async function removeTask(id: string) {
    try {
      await cmd.deleteTask(id)
      tasks.value = tasks.value.filter(t => t.id !== id)
    } catch (e) {
      console.error('Failed to delete task:', e)
    }
  }

  async function runNow(id: string) {
    try {
      const execution = await cmd.runTaskNow(id)
      await fetchTasks()
      if (selectedTaskId.value === id) {
        await fetchHistory(id)
      }
      return execution
    } catch (e) {
      console.error('Failed to run task:', e)
      throw e
    }
  }

  async function fetchHistory(taskId: string) {
    try {
      executionHistory.value = await cmd.getTaskHistory(taskId, 20)
    } catch (e) {
      console.error('Failed to fetch history:', e)
    }
  }

  function selectTask(id: string | null) {
    selectedTaskId.value = id
    if (id) fetchHistory(id)
    else executionHistory.value = []
  }

  return {
    tasks, presets, loading, selectedTaskId, executionHistory,
    fetchPresets, fetchTasks, enablePreset, disableTask, updateSchedule, updateConfig, removeTask, runNow, fetchHistory, selectTask,
  }
})
