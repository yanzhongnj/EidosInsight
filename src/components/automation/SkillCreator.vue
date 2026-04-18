<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  save: [name: string, definition: string, trigger: string]
  cancel: []
}>()

const step = ref<'info' | 'steps' | 'preview'>('info')

// Form state
const skillName = ref('')
const skillDesc = ref('')
const skillTrigger = ref('')

// Steps
interface StepDef {
  id: number
  tool: string
  input_text: string
}
const steps = ref<StepDef[]>([
  { id: 1, tool: 'paper.search', input_text: '' },
])
let nextStepId = 2

const toolOptions = [
  { value: 'paper.search', label: '搜索论文' },
  { value: 'paper.brief', label: '生成摘要' },
  { value: 'export.markdown', label: '导出 Markdown' },
  { value: 'export.summary', label: '导出综述' },
  { value: 'note.create', label: '创建笔记' },
  { value: 'reading.unread', label: '查询未读' },
]

function addStep() {
  steps.value.push({ id: nextStepId++, tool: 'paper.search', input_text: '' })
}

function removeStep(id: number) {
  if (steps.value.length <= 1) return
  steps.value = steps.value.filter(s => s.id !== id)
}

function generateDefinition(): string {
  const stepsYaml = steps.value
    .map((s, i) => {
      const toolLabel = toolOptions.find(t => t.value === s.tool)?.label || s.tool
      return `${i + 1}. **${toolLabel}**\n   tool: ${s.tool}\n   input:\n     ${s.input_text || '# TODO: fill parameters'}`
    })
    .join('\n\n')

  return `---
name: "${skillName.value}"
version: "1.0.0"
description: "${skillDesc.value}"
author: "user"
tags: []
trigger: "${skillTrigger.value}"
tools_required:
${steps.value.map(s => `  - ${s.tool}`).join('\n')}
---

# ${skillName.value}

${skillDesc.value ? `> ${skillDesc.value}\n` : ''}## 步骤

${stepsYaml}`
}

function handleSave() {
  if (!skillName.value.trim()) return
  const definition = generateDefinition()
  emit('save', skillName.value.trim(), definition, skillTrigger.value.trim())
}
</script>

<template>
  <div class="skill-creator">
    <h2>创建技能</h2>

    <!-- Step indicator -->
    <div class="step-indicator">
      <button class="step-dot" :class="{ active: step === 'info' }" @click="step = 'info'">1</button>
      <span class="step-line"></span>
      <button class="step-dot" :class="{ active: step === 'steps' }" @click="step = 'steps'">2</button>
      <span class="step-line"></span>
      <button class="step-dot" :class="{ active: step === 'preview' }" @click="step = 'preview'">3</button>
    </div>

    <!-- Step 1: Basic info -->
    <div v-if="step === 'info'" class="form-step">
      <h3>基本信息</h3>
      <div class="form-group">
        <label>技能名称 <span class="required">*</span></label>
        <input v-model="skillName" class="form-input" placeholder="如：每周热点追踪" />
      </div>
      <div class="form-group">
        <label>描述</label>
        <input v-model="skillDesc" class="form-input" placeholder="简要描述这个技能做什么" />
      </div>
      <div class="form-group">
        <label>触发关键词</label>
        <input v-model="skillTrigger" class="form-input" placeholder="如 /weekly-trending（在 AI 助手中输入触发）" />
      </div>
      <div class="step-actions">
        <button class="btn btn-ghost" @click="emit('cancel')">取消</button>
        <button class="btn btn-primary" :disabled="!skillName.trim()" @click="step = 'steps'">下一步</button>
      </div>
    </div>

    <!-- Step 2: Steps definition -->
    <div v-if="step === 'steps'" class="form-step">
      <h3>定义步骤</h3>
      <div v-for="(s, i) in steps" :key="s.id" class="step-card">
        <div class="step-card-header">
          <span class="step-number">步骤 {{ i + 1 }}</span>
          <button v-if="steps.length > 1" class="btn-remove" @click="removeStep(s.id)">x</button>
        </div>
        <div class="form-group">
          <label>工具</label>
          <select v-model="s.tool" class="form-input">
            <option v-for="opt in toolOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>
        <div class="form-group">
          <label>参数（每行一个 key: value）</label>
          <textarea v-model="s.input_text" class="form-input form-textarea" placeholder="query: {{keywords}}&#10;limit: 10" rows="3"></textarea>
        </div>
      </div>
      <button class="btn btn-ghost" @click="addStep">+ 添加步骤</button>
      <div class="step-actions">
        <button class="btn btn-ghost" @click="step = 'info'">上一步</button>
        <button class="btn btn-primary" @click="step = 'preview'">预览</button>
      </div>
    </div>

    <!-- Step 3: Preview -->
    <div v-if="step === 'preview'" class="form-step">
      <h3>预览技能定义</h3>
      <pre class="definition-preview">{{ generateDefinition() }}</pre>
      <div class="step-actions">
        <button class="btn btn-ghost" @click="step = 'steps'">上一步</button>
        <button class="btn btn-primary" @click="handleSave">保存技能</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-creator h2 { font-size: 18px; font-weight: 600; margin: 0 0 16px; }

.step-indicator { display: flex; align-items: center; gap: 8px; margin-bottom: 20px; }
.step-dot { width: 28px; height: 28px; border-radius: 50%; border: 2px solid var(--border-primary); background: transparent; cursor: pointer; font-size: 12px; font-weight: 600; color: var(--text-tertiary); display: flex; align-items: center; justify-content: center; }
.step-dot.active { border-color: var(--color-primary); background: var(--color-primary); color: white; }
.step-line { flex: 1; height: 2px; background: var(--border-primary); }

.form-step h3 { font-size: 15px; font-weight: 600; margin: 0 0 12px; }
.form-group { margin-bottom: 10px; }
.form-group label { display: block; font-size: 13px; color: var(--text-secondary); margin-bottom: 4px; }
.required { color: var(--color-danger, #ef4444); }
.form-input { width: 100%; padding: 6px 10px; border: 1px solid var(--border-primary); border-radius: var(--radius-sm); background: var(--bg-primary); color: var(--text-primary); font-size: 13px; }
.form-input:focus { outline: none; border-color: var(--color-primary); }
.form-textarea { resize: vertical; font-family: monospace; font-size: 12px; }

.step-card { border: 1px solid var(--border-primary); border-radius: var(--radius-md); padding: 12px; margin-bottom: 8px; }
.step-card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.step-number { font-size: 13px; font-weight: 600; color: var(--color-primary); }
.btn-remove { border: none; background: transparent; color: var(--text-tertiary); cursor: pointer; font-size: 14px; padding: 2px 6px; }
.btn-remove:hover { color: var(--color-danger, #ef4444); }

.definition-preview { background: var(--bg-secondary); border: 1px solid var(--border-primary); border-radius: var(--radius-sm); padding: 12px; font-size: 12px; font-family: monospace; white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; color: var(--text-primary); }

.step-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
.btn { padding: 8px 16px; border-radius: var(--radius-md); border: 1px solid var(--border-primary); background: transparent; cursor: pointer; font-size: 14px; }
.btn.btn-primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.btn.btn-ghost { color: var(--text-secondary); }
.btn.btn-ghost:hover { background: var(--bg-hover); }
.btn:disabled { opacity: .4; cursor: not-allowed; }
</style>
