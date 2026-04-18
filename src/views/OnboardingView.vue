<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/app'
import { useTauriCommand } from '../composables/useTauriCommand'
import type { LlmConfig, LlmModelInfo } from '../types'

const router = useRouter()
const appStore = useAppStore()
const cmd = useTauriCommand()

const step = ref(0) // 0: welcome, 1: llm setup, 2: done
const llmConfig = ref<LlmConfig>({
  provider: 'ollama',
  model_name: 'llama3',
  api_base_url: 'http://localhost:11434',
  api_key_set: false,
  max_tokens: 4096,
  temperature: 0.7,
  is_local: true,
})
const detectingOllama = ref(false)
const ollamaDetected = ref(false)
const ollamaModels = ref<LlmModelInfo[]>([])
const testingConnection = ref(false)
const connectionOk = ref(false)
const apiKeyInput = ref('')
const savingApiKey = ref(false)
const setupMode = ref<'auto' | 'cloud' | 'skip'>('auto')

onMounted(async () => {
  await detectOllama()
})

async function detectOllama() {
  detectingOllama.value = true
  try {
    const result = await cmd.detectOllama()
    ollamaDetected.value = result.available
    ollamaModels.value = result.models
    if (result.available) {
      llmConfig.value.provider = 'ollama'
      llmConfig.value.is_local = true
      // Auto-select first model if available
      if (result.models.length > 0) {
        llmConfig.value.model_name = result.models[0].id
      }
      setupMode.value = 'auto'
    }
  } catch {
    ollamaDetected.value = false
  } finally {
    detectingOllama.value = false
  }
}

async function testLlm() {
  testingConnection.value = true
  try {
    connectionOk.value = await cmd.testLlmConnection(llmConfig.value as Record<string, unknown>)
  } catch {
    connectionOk.value = false
  } finally {
    testingConnection.value = false
  }
}

async function saveApiKeyAndTest() {
  if (!apiKeyInput.value.trim()) return
  savingApiKey.value = true
  try {
    await cmd.saveApiKey(llmConfig.value.provider, apiKeyInput.value.trim())
    llmConfig.value.api_key_set = true
    apiKeyInput.value = ''
    // Test connection after saving key
    connectionOk.value = await cmd.testLlmConnection(llmConfig.value as Record<string, unknown>)
  } catch {
    connectionOk.value = false
  } finally {
    savingApiKey.value = false
  }
}

function selectCloud(provider: string) {
  llmConfig.value.provider = provider
  llmConfig.value.is_local = false
  setupMode.value = 'cloud'
  const defaults: Record<string, { model: string; url: string }> = {
    openai: { model: 'gpt-4o-mini', url: 'https://api.openai.com/v1' },
    claude: { model: 'claude-sonnet-4-20250514', url: 'https://api.anthropic.com/v1' },
    deepseek: { model: 'deepseek-chat', url: 'https://api.deepseek.com/v1' },
  }
  const d = defaults[provider]
  if (d) {
    llmConfig.value.model_name = d.model
    llmConfig.value.api_base_url = d.url
  }
}

async function skipLlm() {
  await finish()
}

async function finish() {
  try {
    // Save LLM config
    await cmd.updateLlmConfig(llmConfig.value as Record<string, unknown>)
    // Load sample data
    await cmd.loadSampleData()
    // Mark onboarding complete
    await cmd.completeOnboarding()
    appStore.isFirstLaunch = false
    router.replace('/')
  } catch (error) {
    console.error('Failed to complete onboarding:', error)
    router.replace('/')
  }
}
</script>

<template>
  <div class="onboarding-view">
    <div class="onboarding-container">
      <!-- Step 0: Welcome -->
      <div v-if="step === 0" class="step-content">
        <div class="welcome-icon">&#x1F52E;</div>
        <h1>欢迎使用 EidosInsight</h1>
        <p class="welcome-desc">
          你的 AI 驱动个人科研知识助手。<br/>
          轻松捕获、组织、阅读和理解研究论文。
        </p>
        <div class="feature-grid">
          <div class="feature-item">
            <span class="feature-icon">&#x1F4DA;</span>
            <span>文献管理</span>
          </div>
          <div class="feature-item">
            <span class="feature-icon">&#x1F4D6;</span>
            <span>PDF 阅读</span>
          </div>
          <div class="feature-item">
            <span class="feature-icon">&#x1F916;</span>
            <span>AI 助手</span>
          </div>
          <div class="feature-item">
            <span class="feature-icon">&#x1F50D;</span>
            <span>全文搜索</span>
          </div>
        </div>
        <button class="btn btn-primary btn-lg" @click="step = 1">
          开始配置 (2分钟)
        </button>
        <button class="btn btn-ghost" @click="finish">跳过，直接使用</button>
      </div>

      <!-- Step 1: LLM Setup -->
      <div v-if="step === 1" class="step-content">
        <h2>配置 AI 助手</h2>
        <p class="step-desc">
          EidosInsight 支持本地和云端 LLM，推荐使用 Ollama 实现完全本地化。
        </p>

        <!-- Ollama auto-detection card -->
        <div class="detection-card card" :class="{ 'card-success': ollamaDetected, 'card-danger': !ollamaDetected && !detectingOllama }">
          <div class="detection-header">
            <div class="detection-title">
              <span class="provider-icon">&#x1F4E1;</span>
              <span>Ollama 本地 LLM</span>
            </div>
            <span v-if="detectingOllama" class="detecting">检测中...</span>
            <span v-else-if="ollamaDetected" class="detected-ok">&#x2713; 已检测到</span>
            <span v-else class="detected-fail">&#x2717; 未检测到</span>
          </div>
          <p class="detection-desc">
            推荐：免费、本地运行、数据不出机器
          </p>
          <!-- Show models if detected -->
          <div v-if="ollamaDetected && ollamaModels.length > 0" class="model-list">
            <label>选择模型</label>
            <select v-model="llmConfig.model_name" class="input">
              <option v-for="model in ollamaModels" :key="model.id" :value="model.id">
                {{ model.name }}
              </option>
            </select>
          </div>
          <button v-if="!ollamaDetected" class="btn btn-secondary btn-sm" @click="detectOllama">
            重新检测
          </button>
          <button v-if="ollamaDetected" class="btn btn-primary btn-sm" @click="testLlm" :disabled="testingConnection">
            {{ testingConnection ? '测试中...' : '测试连接' }}
          </button>
        </div>

        <!-- Cloud API options -->
        <div class="cloud-section">
          <p class="cloud-label">或使用云端 API：</p>
          <div class="cloud-options">
            <button class="cloud-btn" @click="selectCloud('openai')">
              <span class="cloud-name">OpenAI</span>
              <span class="cloud-desc">GPT-4o / GPT-4o-mini</span>
            </button>
            <button class="cloud-btn" @click="selectCloud('deepseek')">
              <span class="cloud-name">DeepSeek</span>
              <span class="cloud-desc">高性价比</span>
            </button>
            <button class="cloud-btn" @click="selectCloud('claude')">
              <span class="cloud-name">Claude</span>
              <span class="cloud-desc">Sonnet / Haiku</span>
            </button>
          </div>
        </div>

        <!-- Cloud API Key input (shown when cloud provider selected) -->
        <div v-if="setupMode === 'cloud' && llmConfig.provider !== 'ollama'" class="api-key-section card">
          <div class="setting-item">
            <label>{{ llmConfig.provider }} API Key</label>
            <input
              v-model="apiKeyInput"
              type="password"
              class="input"
              placeholder="输入 API Key"
            />
          </div>
          <div class="setting-item">
            <label>模型</label>
            <input v-model="llmConfig.model_name" class="input" />
          </div>
          <button class="btn btn-primary btn-sm" @click="saveApiKeyAndTest" :disabled="savingApiKey || !apiKeyInput.trim()">
            {{ savingApiKey ? '保存中...' : '保存并测试' }}
          </button>
          <span v-if="connectionOk" class="connection-ok">&#x2713; 连接成功</span>
        </div>

        <div class="step-actions">
          <button class="btn btn-primary" @click="finish">
            完成配置
          </button>
          <button class="btn btn-ghost" @click="skipLlm">
            暂不配置
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboarding-view {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
}

.onboarding-container {
  max-width: 520px;
  width: 100%;
  padding: 40px;
}

.step-content {
  text-align: center;
}

.welcome-icon {
  font-size: 64px;
  margin-bottom: 24px;
}

h1 {
  font-size: 28px;
  font-weight: 700;
  margin-bottom: 12px;
}

h2 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 12px;
}

.welcome-desc, .step-desc {
  color: var(--text-secondary);
  font-size: 16px;
  line-height: 1.6;
  margin-bottom: 32px;
}

.feature-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 32px;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--bg-card);
  border-radius: var(--radius-md);
  font-size: 14px;
}

.feature-icon {
  font-size: 20px;
}

.btn-lg {
  padding: 12px 32px;
  font-size: 16px;
  margin-bottom: 12px;
}

.detection-card {
  text-align: left;
  margin-bottom: 20px;
}

.detection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 600;
  margin-bottom: 4px;
}

.detection-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.provider-icon {
  font-size: 18px;
}

.detection-desc {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
}

.model-list {
  margin: 8px 0;
}

.model-list label {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.detected-ok { color: var(--color-success); }
.detected-fail { color: var(--color-danger); }
.detecting { color: var(--color-info); }
.connection-ok { color: var(--color-success); font-size: 14px; }

.card-success { border-left: 3px solid var(--color-success); }
.card-danger { border-left: 3px solid var(--color-danger); }

.cloud-section {
  text-align: left;
  margin-bottom: 20px;
}

.cloud-label {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
}

.cloud-options {
  display: flex;
  gap: 8px;
}

.cloud-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 12px 8px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color 0.2s;
}

.cloud-btn:hover {
  border-color: var(--color-info);
}

.cloud-name {
  font-size: 14px;
  font-weight: 600;
}

.cloud-desc {
  font-size: 11px;
  color: var(--text-tertiary);
}

.api-key-section {
  text-align: left;
  margin-bottom: 20px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 0;
}

.setting-item label {
  font-size: 14px;
  color: var(--text-secondary);
  min-width: 100px;
}

.setting-item .input {
  max-width: 250px;
}

.step-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  flex-wrap: wrap;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}
</style>
