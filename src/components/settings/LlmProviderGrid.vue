<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useTauriCommand } from '../../composables/useTauriCommand'

export interface ProviderPreset {
  id: string
  name: string
  adapter_type: string
  base_url: string
  default_model: string
  requires_api_key: boolean
  is_local: boolean
  available_models: string[]
  description: string
}

export interface ConfiguredProvider {
  id: string
  name: string
  is_available: boolean
  is_local: boolean
}

const cmd = useTauriCommand()

const emit = defineEmits<{
  (e: 'switch', providerId: string): void
  (e: 'config-changed'): void
}>()

const presets = ref<ProviderPreset[]>([])
const configuredProviders = ref<ConfiguredProvider[]>([])
const loading = ref(false)
const saving = ref<string | null>(null)
const showKeyInput = ref<string | null>(null)
const newApiKey = ref('')

const sortedPresets = computed(() => {
  return [...presets.value].sort((a, b) => {
    if (a.is_local !== b.is_local) return a.is_local ? -1 : 1
    return 0
  })
})

onMounted(async () => {
  await loadPresets()
  await loadConfigured()
})

async function loadPresets() {
  try {
    presets.value = await cmd.call<ProviderPreset[]>('get_llm_provider_presets', {})
  } catch (e) {
    console.error('Failed to load presets:', e)
  }
}

async function loadConfigured() {
  try {
    configuredProviders.value = await cmd.call<ConfiguredProvider[]>('list_configured_providers', {})
  } catch (e) {
    console.error('Failed to load configured providers:', e)
  }
}

function isConfigured(providerId: string): boolean {
  return configuredProviders.value.some(p => p.id === providerId)
}

function isAvailable(providerId: string): boolean {
  const p = configuredProviders.value.find(p => p.id === providerId)
  return p?.is_available ?? false
}

function startSaveKey(providerId: string) {
  showKeyInput.value = providerId
  newApiKey.value = ''
}

function cancelSaveKey() {
  showKeyInput.value = null
  newApiKey.value = ''
}

async function saveKey(providerId: string) {
  if (!newApiKey.value.trim()) return
  saving.value = providerId
  try {
    await cmd.saveApiKey(providerId, newApiKey.value.trim())
    showKeyInput.value = null
    newApiKey.value = ''
    await loadConfigured()
    emit('config-changed')
  } catch (e) {
    console.error('Failed to save API key:', e)
  } finally {
    saving.value = null
  }
}

async function removeKey(providerId: string) {
  try {
    await cmd.removeApiKey(providerId)
    await loadConfigured()
    emit('config-changed')
  } catch (e) {
    console.error('Failed to remove API key:', e)
  }
}

async function switchProvider(providerId: string) {
  saving.value = providerId
  try {
    await cmd.call<void>('switch_active_provider', { provider: providerId })
    await loadConfigured()
    emit('switch', providerId)
  } catch (e) {
    console.error('Failed to switch provider:', e)
  } finally {
    saving.value = null
  }
}
</script>

<template>
  <div class="provider-grid">
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
    </div>

    <div v-else class="grid-container">
      <div
        v-for="preset in sortedPresets"
        :key="preset.id"
        class="provider-card"
        :class="{
          configured: isConfigured(preset.id),
          available: isAvailable(preset.id),
          active: isConfigured(preset.id) && isAvailable(preset.id),
        }"
      >
        <div class="card-header">
          <span class="provider-icon">
            <template v-if="preset.is_local">🏠</template>
            <template v-else-if="preset.adapter_type === 'anthropic'">🟠</template>
            <template v-else>☁️</template>
          </span>
          <span class="provider-name">{{ preset.name }}</span>
          <span v-if="isConfigured(preset.id) && isAvailable(preset.id)" class="status-dot active" title="可用">●</span>
          <span v-else-if="isConfigured(preset.id)" class="status-dot partial" title="已配置但不可用">◐</span>
        </div>

        <p class="provider-desc">{{ preset.description }}</p>

        <div v-if="preset.requires_api_key && !isConfigured(preset.id)" class="key-input-area">
          <div v-if="showKeyInput === preset.id" class="key-input-row">
            <input
              v-model="newApiKey"
              class="key-input"
              type="password"
              placeholder="输入 API Key"
              @keydown.enter="saveKey(preset.id)"
            />
            <button class="btn btn-primary btn-sm" :disabled="!newApiKey.trim() || saving === preset.id" @click="saveKey(preset.id)">
              {{ saving === preset.id ? '...' : '保存' }}
            </button>
            <button class="btn btn-ghost btn-sm" @click="cancelSaveKey">取消</button>
          </div>
          <button v-else class="btn btn-outline btn-sm" @click="startSaveKey(preset.id)">
            配置 API Key
          </button>
        </div>

        <div v-if="isConfigured(preset.id) && preset.requires_api_key" class="configured-actions">
          <button class="btn btn-ghost btn-sm" @click="switchProvider(preset.id)" :disabled="saving === preset.id">
            {{ saving === preset.id ? '切换中...' : '切换使用' }}
          </button>
          <button class="btn btn-ghost btn-sm btn-danger" @click="removeKey(preset.id)">
            移除
          </button>
        </div>

        <div v-if="preset.is_local && isAvailable(preset.id)" class="configured-actions">
          <span class="local-badge">✓ 已连接</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.provider-grid {
  width: 100%;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.provider-card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: 16px;
  transition: all 0.15s;
}

.provider-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.provider-card.active {
  border-color: #22c55e;
  background: color-mix(in srgb, #22c55e 5%, var(--bg-card));
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.provider-icon {
  font-size: 16px;
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.status-dot {
  font-size: 12px;
}

.status-dot.active {
  color: #22c55e;
}

.status-dot.partial {
  color: #f59e0b;
}

.provider-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 12px;
}

.key-input-area {
  margin-top: 4px;
}

.key-input-row {
  display: flex;
  gap: 4px;
  align-items: center;
}

.key-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-sm);
  font-size: 12px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
  min-width: 0;
}

.key-input:focus {
  border-color: var(--color-primary);
}

.configured-actions {
  display: flex;
  gap: 4px;
  margin-top: 4px;
}

.local-badge {
  font-size: 12px;
  color: #22c55e;
  font-weight: 500;
}

/* Buttons */
.btn {
  padding: 4px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border-color: var(--border-secondary);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn-outline {
  background: transparent;
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.btn-outline:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 11px;
}

.btn-danger:hover {
  color: var(--color-danger, #ef4444) !important;
  border-color: var(--color-danger, #ef4444) !important;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 20px auto;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
