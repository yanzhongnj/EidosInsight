<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTauriCommand } from '../../composables/useTauriCommand'
import { useAppStore } from '../../stores/app'
import type { WizardStep, LlmConfig, LlmModelInfo, WizardDemoResult } from '../../types'

const cmd = useTauriCommand()
const appStore = useAppStore()

const emit = defineEmits<{
  (e: 'done'): void
  (e: 'cancel'): void
}>()

// ── State ────────────────────────────────────────────────────────────────────
const step = ref<WizardStep>('detect')
const llmConfig = ref<LlmConfig>({ ...appStore.llmConfig })

const detectingOllama = ref(false)
const ollamaDetected = ref(false)
const ollamaModels = ref<LlmModelInfo[]>([])
const loadingModels = ref(false)
const allModels = ref<LlmModelInfo[]>([])

const apiKeyInput = ref('')
const showApiKey = ref(false)
const savingApiKey = ref(false)

const testingConnection = ref(false)
const connectionResult = ref<boolean | null>(null)
const connectionError = ref('')

const runningDemo = ref(false)
const demoResult = ref<WizardDemoResult | null>(null)

const saving = ref(false)

// ── Provider metadata ────────────────────────────────────────────────────────
interface ProviderMeta {
  label: string
  icon: string
  color: string
  defaultModel: string
  defaultUrl: string
}

const providers: Record<string, ProviderMeta> = {
  ollama:   { label: 'Ollama (本地)',   icon: '🏠', color: '#10b981', defaultModel: 'llama3',                         defaultUrl: 'http://localhost:11434' },
  openai:   { label: 'OpenAI',          icon: '☁️', color: '#6366f1', defaultModel: 'gpt-4o-mini',                    defaultUrl: 'https://api.openai.com/v1' },
  deepseek: { label: 'DeepSeek',        icon: '🌊', color: '#3b82f6', defaultModel: 'deepseek-chat',                  defaultUrl: 'https://api.deepseek.com/v1' },
  claude:   { label: 'Claude',          icon: '🎭', color: '#8b5cf6', defaultModel: 'claude-sonnet-4-20250514',        defaultUrl: 'https://api.anthropic.com/v1' },
}

const currentProvider = computed(() => providers[llmConfig.value.provider] || providers.ollama)

const stepIndex = computed(() => ['detect', 'select', 'test', 'demo', 'done'].indexOf(step.value))

const canProceed = computed(() => {
  switch (step.value) {
    case 'detect': return true
    case 'select': return !!llmConfig.value.model_name
    case 'test': return connectionResult.value === true
    case 'demo': return true
    default: return true
  }
})

// ── Watch provider change ────────────────────────────────────────────────────
watch(() => llmConfig.value.provider, (newProvider) => {
  llmConfig.value.is_local = newProvider === 'ollama'
  const meta = providers[newProvider]
  if (meta) {
    llmConfig.value.model_name = meta.defaultModel
    llmConfig.value.api_base_url = meta.defaultUrl
  }
  apiKeyInput.value = ''
  connectionResult.value = null
  loadModels()
})

// ── Actions ──────────────────────────────────────────────────────────────────

async function startWizard() {
  step.value = 'detect'
  await detectOllama()
}

async function detectOllama() {
  detectingOllama.value = true
  try {
    const result = await cmd.detectOllama()
    ollamaDetected.value = result.available
    ollamaModels.value = result.models
    if (result.available) {
      llmConfig.value.provider = 'ollama'
      llmConfig.value.is_local = true
      if (result.models.length > 0) {
        llmConfig.value.model_name = result.models[0].id
      }
    }
  } catch {
    ollamaDetected.value = false
  } finally {
    detectingOllama.value = false
  }
  await loadModels()
  // Auto-advance: if Ollama detected with models, go to select
  if (ollamaDetected.value) {
    step.value = 'select'
  }
}

async function loadModels() {
  loadingModels.value = true
  try {
    allModels.value = await cmd.listLlmModels()
  } catch {
    allModels.value = []
  } finally {
    loadingModels.value = false
  }
}

function chooseProvider(provider: string) {
  llmConfig.value.provider = provider
  llmConfig.value.is_local = provider === 'ollama'
  step.value = 'select'
}

async function saveApiKey() {
  if (!apiKeyInput.value.trim()) return
  savingApiKey.value = true
  try {
    await cmd.saveApiKey(llmConfig.value.provider, apiKeyInput.value.trim())
    llmConfig.value.api_key_set = true
    apiKeyInput.value = ''
    await appStore.refreshLlmStatus()
    await loadModels()
  } catch (error) {
    console.error('Failed to save API key:', error)
  } finally {
    savingApiKey.value = false
  }
}

async function testConnection() {
  testingConnection.value = true
  connectionResult.value = null
  connectionError.value = ''
  try {
    // Save config first so the backend uses it
    await cmd.updateLlmConfig(llmConfig.value as Record<string, unknown>)
    await cmd.configureLlmProvider(llmConfig.value as unknown as import('../../types').LlmConfig)
    connectionResult.value = await cmd.testLlmConnection(llmConfig.value as Record<string, unknown>)
    if (!connectionResult.value) {
      connectionError.value = '连接失败，请检查配置'
    }
  } catch (e: any) {
    connectionResult.value = false
    connectionError.value = e?.toString() || '连接失败'
  } finally {
    testingConnection.value = false
  }
}

async function runDemo() {
  runningDemo.value = true
  demoResult.value = null
  try {
    demoResult.value = await cmd.aiWizardDemo()
  } catch (e: any) {
    demoResult.value = {
      success: false,
      demo_text: '',
      provider: llmConfig.value.provider,
      model: llmConfig.value.model_name,
      latency_ms: 0,
      error: e?.toString() || '演示失败',
    }
  } finally {
    runningDemo.value = false
  }
}

async function saveAndFinish() {
  saving.value = true
  try {
    await cmd.updateLlmConfig(llmConfig.value as Record<string, unknown>)
    await cmd.configureLlmProvider(llmConfig.value as unknown as import('../../types').LlmConfig)
    appStore.llmConfig = { ...llmConfig.value }
    await appStore.refreshLlmStatus()
    emit('done')
  } catch (error) {
    console.error('Failed to save LLM config:', error)
  } finally {
    saving.value = false
  }
}

function nextStep() {
  const steps: WizardStep[] = ['detect', 'select', 'test', 'demo', 'done']
  const idx = steps.indexOf(step.value)
  if (idx < steps.length - 1) {
    step.value = steps[idx + 1]
  }
}

function prevStep() {
  const steps: WizardStep[] = ['detect', 'select', 'test', 'demo', 'done']
  const idx = steps.indexOf(step.value)
  if (idx > 0) {
    step.value = steps[idx - 1]
  }
}

function formatModelSize(bytes: number | null): string {
  if (!bytes) return ''
  const gb = bytes / (1024 * 1024 * 1024)
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / (1024 * 1024)).toFixed(0)} MB`
}

// Auto-start
startWizard()
</script>

<template>
  <div class="wizard">
    <!-- Step indicator -->
    <div class="wizard-steps">
      <div
        v-for="(s, i) in ['检测', '选择', '测试', '演示', '完成']"
        :key="i"
        class="wizard-step-dot"
        :class="{ active: i <= stepIndex, current: i === stepIndex }"
      >
        <span class="step-num">{{ i + 1 }}</span>
        <span class="step-label">{{ s }}</span>
      </div>
    </div>

    <!-- Step: Detect -->
    <div v-if="step === 'detect'" class="wizard-content">
      <h3>检测 AI 环境</h3>
      <p class="wizard-desc">正在检测本地 Ollama 和云端 API 的可用性...</p>

      <!-- Ollama card -->
      <div class="provider-card" :class="{ detected: ollamaDetected, undetected: !ollamaDetected && !detectingOllama }">
        <div class="provider-header">
          <span class="provider-icon">🏠</span>
          <span class="provider-name">Ollama 本地</span>
          <span v-if="detectingOllama" class="status-badge checking">检测中...</span>
          <span v-else-if="ollamaDetected" class="status-badge ok">已检测到</span>
          <span v-else class="status-badge fail">未运行</span>
        </div>
        <p class="provider-desc">免费 · 本地运行 · 数据不出机器</p>
        <button v-if="ollamaDetected" class="btn btn-primary btn-sm" @click="chooseProvider('ollama')">
          使用 Ollama
        </button>
        <button v-else class="btn btn-ghost btn-sm" @click="detectOllama" :disabled="detectingOllama">
          重新检测
        </button>
      </div>

      <!-- Cloud providers -->
      <p class="section-label">或选择云端 AI 服务：</p>
      <div class="cloud-grid">
        <button
          v-for="(meta, key) in providers"
          :key="key"
          v-show="key !== 'ollama'"
          class="cloud-card"
          @click="chooseProvider(key)"
        >
          <span class="cloud-icon">{{ meta.icon }}</span>
          <span class="cloud-name">{{ meta.label }}</span>
          <span class="cloud-model">{{ meta.defaultModel }}</span>
        </button>
      </div>

      <div class="wizard-actions">
        <button class="btn btn-ghost" @click="$emit('cancel')">取消</button>
        <button class="btn btn-primary" @click="chooseProvider('ollama')" :disabled="!ollamaDetected">
          下一步
        </button>
      </div>
    </div>

    <!-- Step: Select model -->
    <div v-if="step === 'select'" class="wizard-content">
      <h3>选择模型</h3>
      <p class="wizard-desc">
        当前服务商：<strong :style="{ color: currentProvider.color }">{{ currentProvider.label }}</strong>
      </p>

      <!-- API Key for cloud -->
      <div v-if="llmConfig.provider !== 'ollama'" class="api-key-section">
        <label class="field-label">API Key</label>
        <div class="api-key-row">
          <input
            v-model="apiKeyInput"
            :type="showApiKey ? 'text' : 'password'"
            class="input"
            :placeholder="llmConfig.api_key_set ? '已保存（留空保持不变）' : '输入 API Key'"
          />
          <button class="btn btn-ghost btn-sm" @click="showApiKey = !showApiKey">
            {{ showApiKey ? '隐藏' : '显示' }}
          </button>
        </div>
        <div class="api-key-actions">
          <button class="btn btn-primary btn-sm" @click="saveApiKey" :disabled="savingApiKey || !apiKeyInput.trim()">
            {{ savingApiKey ? '保存中...' : '保存 Key' }}
          </button>
          <button v-if="llmConfig.api_key_set" class="btn btn-danger btn-sm" @click="async () => { await cmd.removeApiKey(llmConfig.provider); llmConfig.api_key_set = false }">
            删除
          </button>
          <span v-if="llmConfig.api_key_set" class="key-saved">Key 已保存</span>
        </div>
      </div>

      <!-- Ollama base URL -->
      <div v-if="llmConfig.provider === 'ollama'" class="field-section">
        <label class="field-label">Ollama 地址</label>
        <input v-model="llmConfig.api_base_url" class="input" placeholder="http://localhost:11434" />
      </div>

      <!-- Model selection -->
      <div class="field-section">
        <label class="field-label">模型</label>
        <div class="model-select-row">
          <select v-model="llmConfig.model_name" class="input">
            <option v-if="loadingModels" disabled value="">加载中...</option>
            <option v-for="model in allModels" :key="model.id" :value="model.id">
              {{ model.name }}
              <template v-if="model.size_bytes"> ({{ formatModelSize(model.size_bytes) }})</template>
            </option>
            <option v-if="!loadingModels && allModels.length === 0" disabled value="">
              无可用模型 — 请先{{ llmConfig.provider === 'ollama' ? '启动 Ollama 并拉取模型' : '保存 API Key' }}
            </option>
          </select>
          <button class="btn btn-ghost btn-sm" @click="loadModels" :disabled="loadingModels">刷新</button>
        </div>
        <!-- Allow manual model name input -->
        <div class="manual-model">
          <input
            v-model="llmConfig.model_name"
            class="input input-sm"
            placeholder="或手动输入模型名称"
          />
        </div>
      </div>

      <!-- Advanced -->
      <details class="advanced-section">
        <summary>高级设置</summary>
        <div class="field-section">
          <label class="field-label">最大 Token 数</label>
          <input v-model.number="llmConfig.max_tokens" type="number" class="input input-sm" min="256" max="128000" />
        </div>
        <div class="field-section">
          <label class="field-label">Temperature</label>
          <div class="range-row">
            <input v-model.number="llmConfig.temperature" type="range" min="0" max="2" step="0.1" class="range-input" />
            <span class="range-value">{{ llmConfig.temperature }}</span>
          </div>
        </div>
      </details>

      <div class="wizard-actions">
        <button class="btn btn-ghost" @click="prevStep">上一步</button>
        <button class="btn btn-primary" @click="nextStep" :disabled="!canProceed">
          下一步
        </button>
      </div>
    </div>

    <!-- Step: Test connection -->
    <div v-if="step === 'test'" class="wizard-content">
      <h3>连接测试</h3>
      <p class="wizard-desc">验证 AI 服务是否可以正常访问。</p>

      <div class="test-config-summary">
        <div class="summary-row">
          <span class="summary-label">服务商</span>
          <span class="summary-value">{{ currentProvider.label }}</span>
        </div>
        <div class="summary-row">
          <span class="summary-label">模型</span>
          <span class="summary-value">{{ llmConfig.model_name }}</span>
        </div>
        <div class="summary-row" v-if="llmConfig.provider !== 'ollama'">
          <span class="summary-label">API Key</span>
          <span class="summary-value">{{ llmConfig.api_key_set ? '已配置' : '未配置' }}</span>
        </div>
      </div>

      <button class="btn btn-primary" @click="testConnection" :disabled="testingConnection">
        {{ testingConnection ? '测试中...' : '测试连接' }}
      </button>

      <div v-if="connectionResult === true" class="test-result success">
        <span class="result-icon">✓</span>
        <span>连接成功！AI 服务可用。</span>
      </div>
      <div v-if="connectionResult === false" class="test-result fail">
        <span class="result-icon">✗</span>
        <span>{{ connectionError || '连接失败，请检查配置。' }}</span>
      </div>

      <div class="wizard-actions">
        <button class="btn btn-ghost" @click="prevStep">上一步</button>
        <button class="btn btn-primary" @click="nextStep" :disabled="!canProceed">
          下一步
        </button>
      </div>
    </div>

    <!-- Step: Demo -->
    <div v-if="step === 'demo'" class="wizard-content">
      <h3>能力演示</h3>
      <p class="wizard-desc">让 AI 生成一段简短回复，验证端到端工作流。</p>

      <button class="btn btn-primary" @click="runDemo" :disabled="runningDemo">
        {{ runningDemo ? '正在生成...' : '运行演示' }}
      </button>

      <div v-if="demoResult" class="demo-result" :class="{ success: demoResult.success, fail: !demoResult.success }">
        <div class="demo-meta">
          <span>{{ providers[demoResult.provider]?.label || demoResult.provider }}</span>
          <span class="demo-sep">·</span>
          <span>{{ demoResult.model }}</span>
          <span class="demo-sep">·</span>
          <span>{{ demoResult.latency_ms }}ms</span>
        </div>
        <div v-if="demoResult.success" class="demo-text">
          {{ demoResult.demo_text }}
        </div>
        <div v-else class="demo-error">
          {{ demoResult.error }}
        </div>
      </div>

      <div class="wizard-actions">
        <button class="btn btn-ghost" @click="prevStep">上一步</button>
        <button class="btn btn-primary" @click="nextStep">
          下一步
        </button>
      </div>
    </div>

    <!-- Step: Done -->
    <div v-if="step === 'done'" class="wizard-content">
      <div class="done-icon">🎉</div>
      <h3>AI 配置完成</h3>
      <p class="wizard-desc">
        你已成功配置 <strong>{{ currentProvider.label }}</strong> / {{ llmConfig.model_name }}。
        <br />现在可以使用术语解释、论文摘要、Insight 提取等 AI 功能。
      </p>

      <div class="capability-list">
        <div class="capability-item">
          <span class="capability-icon">📖</span>
          <div>
            <span class="capability-name">术语解释</span>
            <span class="capability-desc">选中论文中的专业术语，AI 即刻解释</span>
          </div>
        </div>
        <div class="capability-item">
          <span class="capability-icon">📝</span>
          <div>
            <span class="capability-name">论文摘要</span>
            <span class="capability-desc">自动生成结构化摘要：问题、方法、发现、贡献</span>
          </div>
        </div>
        <div class="capability-item">
          <span class="capability-icon">💡</span>
          <div>
            <span class="capability-name">Insight 提取</span>
            <span class="capability-desc">核心贡献、方法亮点、研究局限一键提取</span>
          </div>
        </div>
      </div>

      <div class="wizard-actions">
        <button class="btn btn-primary" @click="saveAndFinish" :disabled="saving">
          {{ saving ? '保存中...' : '完成' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wizard {
  max-width: 560px;
  margin: 0 auto;
}

/* Step indicator */
.wizard-steps {
  display: flex;
  justify-content: space-between;
  margin-bottom: 28px;
  padding: 0 8px;
}

.wizard-step-dot {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  opacity: 0.4;
  transition: opacity 0.3s;
}

.wizard-step-dot.active {
  opacity: 1;
}

.wizard-step-dot.current .step-num {
  background: var(--color-primary);
  color: white;
}

.step-num {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  background: var(--bg-secondary);
  color: var(--text-secondary);
}

.step-label {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* Content */
.wizard-content h3 {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 8px;
}

.wizard-desc {
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.6;
  margin-bottom: 20px;
}

/* Provider cards */
.provider-card {
  padding: 16px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-primary);
  margin-bottom: 16px;
  background: var(--bg-card);
  transition: border-color 0.2s;
}

.provider-card.detected {
  border-color: var(--color-success);
  background: color-mix(in srgb, var(--color-success) 8%, var(--bg-card));
}

.provider-card.undetected {
  border-color: var(--color-danger);
}

.provider-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.provider-icon {
  font-size: 18px;
}

.provider-name {
  font-weight: 600;
  font-size: 15px;
  flex: 1;
}

.provider-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 10px;
}

/* Status badges */
.status-badge {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.status-badge.ok {
  background: color-mix(in srgb, var(--color-success) 15%, transparent);
  color: var(--color-success);
}

.status-badge.fail {
  background: color-mix(in srgb, var(--color-danger) 15%, transparent);
  color: var(--color-danger);
}

.status-badge.checking {
  background: color-mix(in srgb, var(--color-info) 15%, transparent);
  color: var(--color-info);
}

/* Cloud grid */
.section-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
}

.cloud-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-bottom: 20px;
}

.cloud-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 14px 8px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}

.cloud-card:hover {
  border-color: var(--color-primary);
  background: var(--bg-secondary);
}

.cloud-icon {
  font-size: 20px;
  margin-bottom: 4px;
}

.cloud-name {
  font-size: 13px;
  font-weight: 600;
}

.cloud-model {
  font-size: 10px;
  color: var(--text-tertiary);
}

/* Fields */
.field-section {
  margin-bottom: 14px;
}

.field-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.api-key-row {
  display: flex;
  gap: 8px;
}

.api-key-row .input {
  flex: 1;
}

.api-key-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
}

.key-saved {
  font-size: 12px;
  color: var(--color-success);
}

.model-select-row {
  display: flex;
  gap: 8px;
}

.model-select-row .input {
  flex: 1;
}

.manual-model {
  margin-top: 6px;
}

.input-sm {
  font-size: 13px;
  padding: 6px 10px;
}

/* Advanced */
.advanced-section {
  margin: 12px 0;
}

.advanced-section summary {
  font-size: 12px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px 0;
}

.range-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.range-input {
  flex: 1;
  max-width: 200px;
}

.range-value {
  min-width: 28px;
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
}

/* Test */
.test-config-summary {
  padding: 14px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  margin-bottom: 16px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 14px;
}

.summary-label {
  color: var(--text-secondary);
}

.summary-value {
  font-weight: 500;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-radius: var(--radius-md);
  margin-top: 14px;
  font-size: 14px;
}

.test-result.success {
  background: color-mix(in srgb, var(--color-success) 10%, var(--bg-card));
  color: var(--color-success);
}

.test-result.fail {
  background: color-mix(in srgb, var(--color-danger) 10%, var(--bg-card));
  color: var(--color-danger);
}

.result-icon {
  font-size: 18px;
  font-weight: 700;
}

/* Demo */
.demo-result {
  margin-top: 16px;
  padding: 16px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-primary);
}

.demo-result.success {
  border-color: var(--color-success);
  background: color-mix(in srgb, var(--color-success) 5%, var(--bg-card));
}

.demo-result.fail {
  border-color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 5%, var(--bg-card));
}

.demo-meta {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 10px;
}

.demo-sep {
  margin: 0 4px;
}

.demo-text {
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-primary);
}

.demo-error {
  font-size: 14px;
  color: var(--color-danger);
}

/* Done */
.done-icon {
  font-size: 48px;
  text-align: center;
  margin-bottom: 12px;
}

.capability-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
}

.capability-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.capability-icon {
  font-size: 20px;
  flex-shrink: 0;
  margin-top: 2px;
}

.capability-name {
  display: block;
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 2px;
}

.capability-desc {
  display: block;
  font-size: 12px;
  color: var(--text-tertiary);
}

/* Actions */
.wizard-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-primary);
}

/* Button helpers */
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-danger {
  background: var(--color-danger);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
}

.btn-danger:hover {
  opacity: 0.9;
}
</style>
