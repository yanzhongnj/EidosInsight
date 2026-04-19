<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '../../stores/app'
import { useTauriCommand } from '../../composables/useTauriCommand'

const props = defineProps<{
  paperId: string
  selectedTagIds: string[]
}>()

const emit = defineEmits<{
  addTag: [tagName: string]
  removeTag: [tagId: string]
}>()

const appStore = useAppStore()
const cmd = useTauriCommand()

const searchQuery = ref('')
const showSuggestions = ref(false)
const suggestedTags = ref<string[]>([])
const loadingSuggestions = ref(false)

const filteredTags = computed(() => {
  if (!searchQuery.value) return appStore.tags
  const q = searchQuery.value.toLowerCase()
  return appStore.tags.filter(t => t.name.toLowerCase().includes(q))
})

const availableTags = computed(() => {
  return filteredTags.value.filter(t => !props.selectedTagIds.includes(t.id))
})

async function loadSuggestions() {
  loadingSuggestions.value = true
  try {
    suggestedTags.value = await cmd.suggestTags(props.paperId)
  } catch {
    suggestedTags.value = []
  } finally {
    loadingSuggestions.value = false
  }
  showSuggestions.value = true
}

async function createAndAdd() {
  if (!searchQuery.value.trim()) return
  emit('addTag', searchQuery.value.trim())
  searchQuery.value = ''
  showSuggestions.value = false
}

function selectTag(tagName: string) {
  emit('addTag', tagName)
  searchQuery.value = ''
  showSuggestions.value = false
}

function removeTag(tagId: string) {
  emit('removeTag', tagId)
}
</script>

<template>
  <div class="tag-selector">
    <!-- Selected tags -->
    <div class="selected-tags">
      <span
        v-for="tag in appStore.tags.filter(t => selectedTagIds.includes(t.id))"
        :key="tag.id"
        class="selected-tag"
        :style="tag.color ? { backgroundColor: `${tag.color}20`, borderColor: tag.color, color: tag.color } : undefined"
      >
        {{ tag.name }}
        <button class="remove-btn" @click="removeTag(tag.id)">x</button>
      </span>
    </div>

    <!-- Search input -->
    <div class="tag-input-wrapper">
      <input
        v-model="searchQuery"
        placeholder="搜索或创建标签..."
        @focus="loadSuggestions"
        @keyup.enter="createAndAdd"
      />
      <button class="suggest-btn" @click="loadSuggestions" :disabled="loadingSuggestions">
        {{ loadingSuggestions ? '...' : 'AI建议' }}
      </button>
    </div>

    <!-- Dropdown -->
    <div v-if="showSuggestions && (availableTags.length || suggestedTags.length)" class="tag-dropdown">
      <!-- AI suggestions -->
      <div v-if="suggestedTags.length" class="dropdown-section">
        <div class="section-label">AI 建议标签</div>
        <div
          v-for="name in suggestedTags"
          :key="'sug-' + name"
          class="dropdown-item"
          @click="selectTag(name)"
        >
          ✨ {{ name }}
        </div>
      </div>

      <!-- Existing tags -->
      <div v-if="availableTags.length" class="dropdown-section">
        <div class="section-label">已有标签</div>
        <div
          v-for="tag in availableTags.slice(0, 15)"
          :key="tag.id"
          class="dropdown-item"
          :style="{ color: tag.color || undefined }"
          @click="selectTag(tag.name)"
        >
          {{ tag.name }}
          <span class="tag-usage">{{ tag.usage_count }}</span>
        </div>
      </div>

      <!-- Create option -->
      <div v-if="searchQuery.trim() && !appStore.tags.find(t => t.name === searchQuery.trim())" class="dropdown-item create-option" @click="createAndAdd">
        + 创建 "{{ searchQuery.trim() }}"
      </div>
    </div>
  </div>
</template>

<style scoped>
.tag-selector {
  position: relative;
}

.selected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 8px;
}

.selected-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 9999px;
  border: 1px solid;
  font-size: 12px;
  font-weight: 500;
}

.remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 10px;
  opacity: 0.6;
  padding: 0 2px;
  color: inherit;
}

.remove-btn:hover {
  opacity: 1;
}

.tag-input-wrapper {
  display: flex;
  gap: 4px;
}

.tag-input-wrapper input {
  flex: 1;
  background: var(--bg-primary);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-sm);
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
}

.tag-input-wrapper input:focus {
  border-color: var(--color-primary);
}

.suggest-btn {
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-secondary);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  white-space: nowrap;
}

.suggest-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-primary) 10%, var(--bg-tertiary));
  color: var(--color-primary);
}

.suggest-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tag-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  z-index: 100;
  max-height: 200px;
  overflow-y: auto;
  margin-top: 4px;
}

.dropdown-section {
  padding: 4px 0;
}

.dropdown-section + .dropdown-section {
  border-top: 1px solid var(--border-primary);
}

.section-label {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--text-tertiary);
  font-weight: 600;
  text-transform: uppercase;
}

.dropdown-item {
  padding: 6px 12px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.1s;
}

.dropdown-item:hover {
  background: var(--bg-hover);
}

.dropdown-item.create-option {
  color: var(--color-primary);
  font-weight: 500;
  border-top: 1px solid var(--border-primary);
}

.tag-usage {
  font-size: 10px;
  opacity: 0.5;
  margin-left: 4px;
}
</style>
