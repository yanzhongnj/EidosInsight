//! Skill store - manages skills and skill execution

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Skill, SkillResult } from '../types'
import { useTauriCommand } from '../composables/useTauriCommand'

export const useSkillStore = defineStore('skill', () => {
  const cmd = useTauriCommand()

  const skills = ref<Skill[]>([])
  const loading = ref(false)
  const executing = ref(false)

  async function fetchSkills() {
    loading.value = true
    try {
      skills.value = await cmd.listSkills()
    } catch (e) {
      console.error('Failed to fetch skills:', e)
    } finally {
      loading.value = false
    }
  }

  async function createSkill(name: string, definition: string, source: string, trigger?: string) {
    try {
      const skill = await cmd.createSkill(name, definition, source, trigger)
      skills.value.push(skill)
      return skill
    } catch (e) {
      console.error('Failed to create skill:', e)
      throw e
    }
  }

  async function importSkill(markdown: string) {
    try {
      const skill = await cmd.importSkill(markdown)
      skills.value.push(skill)
      return skill
    } catch (e) {
      console.error('Failed to import skill:', e)
      throw e
    }
  }

  async function toggleSkill(id: string, active: boolean) {
    try {
      await cmd.updateSkill(id, { isActive: active })
      const skill = skills.value.find(s => s.id === id)
      if (skill) skill.is_active = active
    } catch (e) {
      console.error('Failed to toggle skill:', e)
    }
  }

  async function removeSkill(id: string) {
    try {
      await cmd.deleteSkill(id)
      skills.value = skills.value.filter(s => s.id !== id)
    } catch (e) {
      console.error('Failed to delete skill:', e)
    }
  }

  async function invokeSkill(skillId: string, params?: Record<string, unknown>): Promise<SkillResult | null> {
    executing.value = true
    try {
      const result = await cmd.invokeSkill(skillId, params)
      return result
    } catch (e) {
      console.error('Failed to invoke skill:', e)
      return null
    } finally {
      executing.value = false
    }
  }

  return {
    skills, loading, executing,
    fetchSkills, createSkill, importSkill, toggleSkill, removeSkill, invokeSkill,
  }
})
