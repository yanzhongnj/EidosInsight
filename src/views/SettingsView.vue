<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useAppStore } from '../stores/app'
import { useTauriCommand } from '../composables/useTauriCommand'
import AiConfigWizard from '../components/settings/AiConfigWizard.vue'
import LlmProviderGrid from '../components/settings/LlmProviderGrid.vue'
import UpdateChecker from '../components/settings/UpdateChecker.vue'
import type { AppSettings, LlmConfig, LlmModelInfo, WizardDemoResult, ProviderPreset, ConfiguredProvider } from '../types'

const appStore = useAppStore()
const cmd = useTauriCommand()

const settings = ref<AppSettings>({ ...appStore.settings })
const llmConfig = ref<LlmConfig>({ ...appStore.llmConfig })
const saving = ref(false)

// LLM enhanced state
const llmModels = ref<LlmModelInfo[]>([])
const loadingModels = ref(false)
const ollamaDetected = ref(false)
const detectingOllama = ref(false)

// Wizard mode
const showWizard = ref(false)

// Provider grid state
const providerPresets = ref<ProviderPreset[]>([])
const configuredProviders = ref<ConfiguredProvider[]>([])
const selectedProviderId = ref<string>('ollama')

// Quick demo
const runningQuickDemo = ref(false)
const quickDemoResult = ref<WizardDemoResult | null>(null)

// Connection test
const testingConnection = ref(false)
const connectionResult = ref<boolean | null>(null)

onMounted(async () => {
  settings.value = { ...appStore.settings }
  llmConfig.value = { ...appStore.llmConfig }
  selectedProviderId.value = llmConfig.value.provider || 'ollama'
  await detectOllama()
  await loadModels()
  await loadProviderPresets()
  await loadConfiguredProviders()
})

watch(() => llmConfig.value.provider, (newProvider) => {
  llmConfig.value.is_local = newProvider === 'ollama'
  connectionResult.value = null
  loadModels()
})

// ── Ollama ────────────────────────────────────────────────────────────────────

async function loadProviderPresets() {
  try {
    providerPresets.value = await cmd.getLlmProviderPresets()
  } catch {
    providerPresets.value = []
  }
}

async function loadConfiguredProviders() {
  try {
    configuredProviders.value = await cmd.listConfiguredProviders()
  } catch {
    configuredProviders.value = []
  }
}

// When user switches provider via LlmProviderGrid
function onProviderSwitched(providerId: string) {
  selectedProviderId.value = providerId
  llmConfig.value.provider = providerId
  llmConfig.value.is_local = providerId === 'ollama'
  const preset = providerPresets.value.find(p => p.id === providerId)
  if (preset) {
    llmConfig.value.api_base_url = preset.base_url
    if (!llmConfig.value.model_name || llmConfig.value.model_name === preset.default_model) {
      llmConfig.value.model_name = preset.default_model
    }
  }
  connectionResult.value = null
  loadModels()
  loadConfiguredProviders()
  // Auto-save the active provider switch
  cmd.switchActiveProvider(providerId).catch(e => console.error('Switch provider failed:', e))
}

// When API key is saved or removed via LlmProviderGrid
function onProviderConfigChanged() {
  loadConfiguredProviders()
  appStore.refreshLlmStatus()
}

async function detectOllama() {
  detectingOllama.value = true
  try {
    const result = await cmd.detectOllama()
    ollamaDetected.value = result.available
    if (result.available && result.models.length > 0) {
      llmModels.value = result.models
    }
  } catch {
    ollamaDetected.value = false
  } finally {
    detectingOllama.value = false
  }
}

async function loadModels() {
  loadingModels.value = true
  try {
    llmModels.value = await cmd.listLlmModels()
  } catch {
    llmModels.value = []
  } finally {
    loadingModels.value = false
  }
}

// ── Settings ──────────────────────────────────────────────────────────────────

async function saveSettings() {
  saving.value = true
  try {
    await appStore.updateSettings(settings.value)
  } catch (error) {
    console.error('Failed to save settings:', error)
  } finally {
    saving.value = false
  }
}

async function saveLlmConfig() {
  try {
    await cmd.updateLlmConfig(llmConfig.value as Record<string, unknown>)
    await cmd.configureLlmProvider(llmConfig.value as unknown as import('../types').LlmConfig)
    appStore.llmConfig = { ...llmConfig.value }
    await appStore.refreshLlmStatus()
  } catch (error) {
    console.error('Failed to save LLM config:', error)
  }
}

async function testConnection() {
  testingConnection.value = true
  connectionResult.value = null
  try {
    await saveLlmConfig()
    connectionResult.value = await cmd.testLlmConnection(llmConfig.value as Record<string, unknown>)
  } catch {
    connectionResult.value = false
  } finally {
    testingConnection.value = false
  }
}

async function runQuickDemo() {
  runningQuickDemo.value = true
  quickDemoResult.value = null
  try {
    await saveLlmConfig()
    quickDemoResult.value = await cmd.aiWizardDemo()
  } catch (e: any) {
    quickDemoResult.value = {
      success: false,
      demo_text: '',
      provider: llmConfig.value.provider,
      model: llmConfig.value.model_name,
      latency_ms: 0,
      error: e?.toString() || '演示失败',
    }
  } finally {
    runningQuickDemo.value = false
  }
}

function getStatusColor() {
  switch (appStore.llmStatus.status) {
    case 'local_available': return 'var(--color-success)'
    case 'cloud_available': return 'var(--color-info)'
    default: return 'var(--color-ai-off, #9CA3AF)'
  }
}

function getStatusLabel() {
  switch (appStore.llmStatus.status) {
    case 'local_available': return '本地 AI 可用'
    case 'cloud_available': return '云端 AI 可用'
    default: return 'AI 未配置'
  }
}

function formatModelSize(bytes: number | null): string {
  if (!bytes) return ''
  const gb = bytes / (1024 * 1024 * 1024)
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / (1024 * 1024)).toFixed(0)} MB`
}

function onWizardDone() {
  showWizard.value = false
  // Reload config from store
  llmConfig.value = { ...appStore.llmConfig }
  loadModels()
}

async function quitApp() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch (e) {
    console.error('Failed to quit app:', e)
  }
}
</script>

<template>
  <div class="settings-view">
    <h1 class="page-title">设置</h1>

    <!-- Wizard overlay -->
    <div v-if="showWizard" class="wizard-overlay">
      <div class="wizard-card card">
        <AiConfigWizard @done="onWizardDone" @cancel="showWizard = false" />
      </div>
    </div>

    <!-- General settings -->
    <section class="settings-section card">
      <h2>通用</h2>
      <div class="setting-item">
        <label>主题</label>
        <select v-model="settings.theme" class="input">
          <option value="light">浅色</option>
          <option value="dark">深色</option>
          <option value="system">跟随系统</option>
        </select>
      </div>
      <div class="setting-item">
        <label>语言</label>
        <select v-model="settings.language" class="input">
          <option value="zh-CN">中文</option>
          <option value="en-US">English</option>
        </select>
      </div>
      <div class="setting-item">
        <label>自动处理新论文</label>
        <input type="checkbox" v-model="settings.auto_process_papers" />
      </div>
      <div class="setting-item">
        <label>启动时检查更新</label>
        <input type="checkbox" v-model="settings.check_updates_on_start" />
      </div>
      <div class="setting-item">
        <label>默认导出格式</label>
        <select v-model="settings.export_format" class="input">
          <option value="bibtex">BibTeX</option>
          <option value="apa">APA</option>
          <option value="csv">CSV</option>
          <option value="markdown">Markdown</option>
        </select>
      </div>
      <button class="btn btn-primary" @click="saveSettings" :disabled="saving">
        {{ saving ? '保存中...' : '保存设置' }}
      </button>
    </section>

    <!-- LLM settings -->
    <section class="settings-section card">
      <div class="section-header">
        <h2>AI 配置</h2>
        <button class="btn btn-secondary btn-sm" @click="showWizard = true">
          配置向导
        </button>
      </div>

      <!-- Status indicator -->
      <div class="llm-status-bar">
        <span class="status-dot" :style="{ backgroundColor: getStatusColor() }"></span>
        <span class="status-label">{{ getStatusLabel() }}</span>
        <span v-if="llmConfig.model_name" class="status-model">{{ llmConfig.model_name }}</span>
      </div>

      <!-- Provider Grid -->
      <LlmProviderGrid
        @switch="onProviderSwitched"
        @config-changed="onProviderConfigChanged"
      />

      <!-- Ollama detection -->
      <div v-if="llmConfig.provider === 'ollama'" class="ollama-detection">
        <div class="detection-row">
          <span>Ollama 状态</span>
          <span v-if="detectingOllama" class="text-muted">检测中...</span>
          <span v-else-if="ollamaDetected" class="text-success">运行中</span>
          <span v-else class="text-danger">未运行</span>
        </div>
        <button class="btn btn-ghost btn-sm" @click="detectOllama">重新检测</button>
      </div>

      <!-- Ollama base URL -->
      <div v-if="llmConfig.provider === 'ollama'" class="setting-item">
        <label>Ollama 地址</label>
        <input v-model="llmConfig.api_base_url" class="input" placeholder="http://localhost:11434" />
      </div>

      <!-- Custom endpoint URL -->
      <div v-if="llmConfig.provider === 'custom'" class="setting-item">
        <label>API 地址</label>
        <input v-model="llmConfig.api_base_url" class="input" placeholder="https://your-api-endpoint/v1" />
      </div>

      <!-- Model selection -->
      <div class="setting-item">
        <label>模型</label>
        <div class="model-select-group">
          <select v-model="llmConfig.model_name" class="input">
            <option v-if="loadingModels" disabled value="">加载中...</option>
            <option v-for="model in llmModels" :key="model.id" :value="model.id">
              {{ model.name }}
              <template v-if="model.size_bytes"> ({{ formatModelSize(model.size_bytes) }})</template>
            </option>
            <option v-if="!loadingModels && llmModels.length === 0" disabled value="">无可用模型</option>
          </select>
          <button class="btn btn-ghost btn-sm" @click="loadModels" :disabled="loadingModels">刷新</button>
        </div>
      </div>

      <!-- Manual model name -->
      <div class="setting-item manual-model-row">
        <label></label>
        <input v-model="llmConfig.model_name" class="input input-sm" placeholder="或手动输入模型名称" />
      </div>

      <!-- Advanced settings -->
      <details class="llm-advanced">
        <summary>高级设置</summary>
        <div class="setting-item">
          <label>最大 Token 数</label>
          <input v-model.number="llmConfig.max_tokens" type="number" class="input" min="256" max="128000" />
        </div>
        <div class="setting-item">
          <label>Temperature</label>
          <input v-model.number="llmConfig.temperature" type="range" min="0" max="2" step="0.1" class="input-range" />
          <span class="range-value">{{ llmConfig.temperature }}</span>
        </div>
      </details>

      <!-- Actions -->
      <div class="llm-actions">
        <button class="btn btn-secondary" @click="testConnection" :disabled="testingConnection">
          {{ testingConnection ? '测试中...' : '测试连接' }}
        </button>
        <span v-if="connectionResult === true" class="connection-ok">连接成功</span>
        <span v-if="connectionResult === false" class="connection-fail">连接失败</span>
        <button class="btn btn-primary" @click="saveLlmConfig">保存 LLM 配置</button>
      </div>

      <!-- Quick demo -->
      <div class="demo-section">
        <div class="demo-header">
          <span class="demo-label">能力演示</span>
          <button class="btn btn-ghost btn-sm" @click="runQuickDemo" :disabled="runningQuickDemo || appStore.llmStatus.status === 'unavailable'">
            {{ runningQuickDemo ? '生成中...' : '试一试' }}
          </button>
        </div>
        <div v-if="quickDemoResult" class="demo-result" :class="{ success: quickDemoResult.success, fail: !quickDemoResult.success }">
          <div class="demo-meta">
            <span>{{ quickDemoResult.provider }}</span>
            <span class="demo-sep">·</span>
            <span>{{ quickDemoResult.model }}</span>
            <span class="demo-sep">·</span>
            <span>{{ quickDemoResult.latency_ms }}ms</span>
          </div>
          <div v-if="quickDemoResult.success" class="demo-text">
            {{ quickDemoResult.demo_text }}
          </div>
          <div v-else class="demo-error">
            {{ quickDemoResult.error }}
          </div>
        </div>
      </div>
    </section>

    <!-- About -->
    <section class="settings-section card">
      <h2>关于</h2>
      <p>EidosInsight v0.1.0</p>
      <p class="text-muted">AI 驱动的个人科研知识助手</p>
      <UpdateChecker />
    </section>

    <!-- Exit -->
    <section class="settings-section card">
      <div class="exit-section">
        <div class="exit-info">
          <span class="exit-label">退出应用</span>
          <span class="exit-desc text-muted">关闭 EidosInsight 桌面应用</span>
        </div>
        <button class="btn btn-exit" @click="quitApp">退出</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-view {
  max-width: 700px;
  margin: 0 auto;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 24px;
}

.settings-section {
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.section-header h2 {
  margin-bottom: 0;
}

.settings-section h2 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-primary);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 0;
}

.setting-item label {
  font-size: 14px;
  color: var(--text-secondary);
  min-width: 120px;
}

.setting-item .input {
  max-width: 300px;
}

.manual-model-row {
  padding-top: 0;
}

.input-sm {
  font-size: 13px;
  padding: 6px 10px;
}

.input-range {
  flex: 1;
  max-width: 200px;
}

.range-value {
  min-width: 32px;
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
}

.llm-status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  margin-bottom: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-label {
  font-size: 14px;
  font-weight: 500;
}

.status-model {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-left: auto;
}

.ollama-detection {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  margin-bottom: 12px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.detection-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.model-select-group {
  display: flex;
  gap: 8px;
  flex: 1;
  max-width: 300px;
}

.model-select-group .input {
  flex: 1;
}

.llm-advanced {
  margin: 12px 0;
}

.llm-advanced summary {
  font-size: 13px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px 0;
}

.llm-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
}

.connection-ok {
  color: var(--color-success);
  font-size: 14px;
}

.connection-fail {
  color: var(--color-danger);
  font-size: 14px;
}

/* Demo section */
.demo-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border-primary);
}

.demo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.demo-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

.demo-result {
  padding: 14px;
  border-radius: var(--radius-md);
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
  font-size: 11px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
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

/* Wizard overlay */
.wizard-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.wizard-card {
  max-width: 620px;
  width: 90%;
  max-height: 85vh;
  overflow-y: auto;
  padding: 32px;
}

.text-muted {
  color: var(--text-tertiary);
  font-size: 13px;
}

.text-success {
  color: var(--color-success);
  font-weight: 500;
}

.text-danger {
  color: var(--color-danger);
  font-weight: 500;
}

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

.exit-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.exit-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.exit-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.exit-desc {
  font-size: 13px;
}

.btn-exit {
  padding: 8px 24px;
  font-size: 14px;
  background: transparent;
  color: var(--color-danger);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-exit:hover {
  background: var(--color-danger);
  color: white;
}
</style>
