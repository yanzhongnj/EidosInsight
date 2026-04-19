<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '../../stores/app'
import { useTauriCommand } from '../../composables/useTauriCommand'
import type { Tag } from '../../types'

const appStore = useAppStore()
const cmd = useTauriCommand()

const showCreateInput = ref(false)
const newTagName = ref('')
const editingTagId = ref<string | null>(null)
const editTagName = ref('')

async function createTag() {
  if (!newTagName.value.trim()) return
  try {
    await cmd.createTag(newTagName.value.trim())
    await appStore.fetchTags()
    newTagName.value = ''
    showCreateInput.value = false
  } catch (e) {
    console.error('Failed to create tag:', e)
  }
}

async function renameTag(tag: Tag) {
  if (!editTagName.value.trim()) return
  try {
    await cmd.renameTag(tag.id, editTagName.value.trim())
    await appStore.fetchTags()
    editingTagId.value = null
  } catch (e) {
    console.error('Failed to rename tag:', e)
  }
}

async function deleteTag(tag: Tag) {
  try {
    await cmd.deleteTag(tag.id)
    await appStore.fetchTags()
  } catch (e) {
    console.error('Failed to delete tag:', e)
  }
}

function getTagSize(count: number): string {
  const min = 12
  const max = 18
  const maxCount = Math.max(...appStore.tags.map(t => t.usage_count), 1)
  const size = min + (max - min) * (count / maxCount)
  return `${Math.round(size)}px`
}
</script>

<template>
  <div class="tag-cloud">
    <div class="tags-container">
      <span
        v-for="tag in appStore.tags"
        :key="tag.id"
        class="tag-chip"
        :style="{
          backgroundColor: tag.color ? `${tag.color}20` : undefined,
          borderColor: tag.color || undefined,
          color: tag.color || undefined,
          fontSize: getTagSize(tag.usage_count),
        }"
        @click="$router.push('/'); $router.push('/')"
        @dblclick="editingTagId = tag.id; editTagName = tag.name"
        @contextmenu.prevent="deleteTag(tag)"
      >
        <template v-if="editingTagId === tag.id">
          <input
            v-model="editTagName"
            class="tag-rename-input"
            @keyup.enter="renameTag(tag)"
            @keyup.escape="editingTagId = null"
            @blur="renameTag(tag)"
            autofocus
          />
        </template>
        <template v-else>
          {{ tag.name }}
          <span class="tag-count">{{ tag.usage_count }}</span>
        </template>
      </span>
    </div>

    <!-- Create new tag input -->
    <div v-if="showCreateInput" class="create-tag-input">
      <input
        v-model="newTagName"
        placeholder="输入标签名称..."
        @keyup.enter="createTag"
        @keyup.escape="showCreateInput = false"
        autofocus
      />
      <button class="confirm-btn" @click="createTag">✓</button>
      <button class="cancel-btn" @click="showCreateInput = false">✕</button>
    </div>

    <button v-if="!showCreateInput" class="create-tag-btn" @click="showCreateInput = true">
      + 新建标签
    </button>
  </div>
</template>

<style scoped>
.tag-cloud {
  padding: 4px 0;
}

.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 4px 12px;
  max-height: 200px;
  overflow-y: auto;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 9999px;
  border: 1px solid var(--border-secondary);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
  font-weight: 500;
}

.tag-chip:hover {
  opacity: 0.85;
  transform: scale(1.05);
}

.tag-count {
  font-size: 10px;
  opacity: 0.7;
  font-weight: 400;
}

.tag-rename-input {
  background: transparent;
  border: none;
  color: inherit;
  font-size: inherit;
  width: 60px;
  outline: none;
  padding: 0;
}

.create-tag-input {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
}

.create-tag-input input {
  flex: 1;
  background: var(--bg-primary);
  border: 1px solid var(--border-secondary);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
}

.create-tag-input input:focus {
  border-color: var(--color-primary);
}

.confirm-btn, .cancel-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 4px 6px;
  border-radius: 4px;
}

.confirm-btn { color: var(--color-primary); }
.cancel-btn { color: var(--text-tertiary); }

.create-tag-btn {
  display: block;
  width: calc(100% - 24px);
  margin: 4px 12px;
  padding: 5px 8px;
  background: transparent;
  border: 1px dashed var(--border-secondary);
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.create-tag-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 5%, transparent);
}
</style>
