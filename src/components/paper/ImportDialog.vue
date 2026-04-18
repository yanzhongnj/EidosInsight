<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePaperStore } from '../../stores/paper'

const paperStore = usePaperStore()
const emit = defineEmits<{
  close: []
}>()

const importing = ref(false)
const importError = ref<string | null>(null)
const importSuccess = ref(false)
const importMode = ref<'single' | 'batch'>('single')
const dragOver = ref(false)

const canImport = computed(() => !importing.value)

async function handleImportPdf() {
  if (!canImport.value) return
  importing.value = true
  importError.value = null
  importSuccess.value = false

  try {
    if (importMode.value === 'batch') {
      await paperStore.batchImportPdfs()
    } else {
      await paperStore.importPdf()
    }
    importSuccess.value = true
    setTimeout(() => {
      emit('close')
    }, 800)
  } catch (error) {
    importError.value = error instanceof Error ? error.message : String(error)
  } finally {
    importing.value = false
  }
}

async function handleDrop(event: DragEvent) {
  event.preventDefault()
  dragOver.value = false

  if (!event.dataTransfer?.files.length) return

  importing.value = true
  importError.value = null

  try {
    // For each dropped file, import it
    for (const file of event.dataTransfer.files) {
      if (file.type === 'application/pdf' || file.name.endsWith('.pdf')) {
        // Tauri needs the file path, which we can get from the File object
        // But in web context we don't have the path - use the dialog-based import instead
        await paperStore.importPdf()
        break // Use single import for drag-drop (opens dialog as fallback)
      }
    }
    importSuccess.value = true
    setTimeout(() => {
      emit('close')
    }, 800)
  } catch (error) {
    importError.value = error instanceof Error ? error.message : String(error)
  } finally {
    importing.value = false
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  dragOver.value = true
}

function handleDragLeave() {
  dragOver.value = false
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content import-dialog">
      <div class="modal-header">
        <h2>导入文献</h2>
        <button class="btn-icon" @click="emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <!-- Import mode tabs -->
        <div class="import-tabs">
          <button
            class="tab-btn"
            :class="{ active: importMode === 'single' }"
            @click="importMode = 'single'"
          >
            单个导入
          </button>
          <button
            class="tab-btn"
            :class="{ active: importMode === 'batch' }"
            @click="importMode = 'batch'"
          >
            批量导入
          </button>
        </div>

        <!-- Drop zone -->
        <div
          class="drop-zone"
          :class="{ 'drag-over': dragOver, importing }"
          @drop="handleDrop"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
        >
          <div v-if="importing" class="import-progress">
            <div class="spinner"></div>
            <p>正在导入，请稍候...</p>
          </div>
          <div v-else-if="importSuccess" class="import-success">
            <span class="success-icon">✓</span>
            <p>导入成功！</p>
          </div>
          <template v-else>
            <div class="drop-icon">📄</div>
            <p class="drop-text">拖拽 PDF 文件到此处</p>
            <p class="drop-hint">或者点击下方按钮选择文件</p>
          </template>
        </div>

        <!-- Error display -->
        <div v-if="importError" class="import-error">
          <span class="error-icon">⚠</span>
          <span>{{ importError }}</span>
        </div>

        <!-- Action buttons -->
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="emit('close')" :disabled="importing">
            取消
          </button>
          <button
            class="btn btn-primary"
            @click="handleImportPdf"
            :disabled="!canImport || importing"
          >
            {{ importing ? '导入中...' : (importMode === 'batch' ? '选择多个文件' : '选择 PDF 文件') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal-content {
  background: var(--bg-primary);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  width: 480px;
  max-width: 90vw;
  border: 1px solid var(--border-primary);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--border-primary);
}

.modal-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.btn-icon {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-md);
  transition: all 0.15s;
}

.btn-icon:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.modal-body {
  padding: 20px 24px 24px;
}

.import-tabs {
  display: flex;
  gap: 4px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-lg);
  padding: 4px;
  margin-bottom: 20px;
}

.tab-btn {
  flex: 1;
  padding: 8px 16px;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.tab-btn.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.drop-zone {
  border: 2px dashed var(--border-secondary);
  border-radius: var(--radius-lg);
  padding: 40px 20px;
  text-align: center;
  transition: all 0.2s;
  cursor: pointer;
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.drop-zone.drag-over {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
}

.drop-zone.importing {
  border-color: var(--color-primary);
  opacity: 0.8;
}

.drop-icon {
  font-size: 40px;
  margin-bottom: 12px;
}

.drop-text {
  font-size: 16px;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.drop-hint {
  font-size: 13px;
  color: var(--text-tertiary);
}

.import-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-secondary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.import-success {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--color-success, #22c55e);
}

.success-icon {
  font-size: 32px;
  font-weight: bold;
}

.import-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: color-mix(in srgb, #ef4444 10%, transparent);
  border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
  border-radius: var(--radius-md);
  color: #ef4444;
  font-size: 13px;
  margin-top: 12px;
}

.error-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn {
  padding: 8px 20px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
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
</style>
