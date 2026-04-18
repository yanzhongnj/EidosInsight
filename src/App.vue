<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useAppStore } from './stores/app'
import router from './router'

const appStore = useAppStore()

onMounted(async () => {
  await appStore.initialize()
  if (appStore.isFirstLaunch) {
    router.replace('/onboarding')
  }
})

// Apply theme
watch(() => appStore.isDark, (dark) => {
  document.documentElement.classList.toggle('dark', dark)
}, { immediate: true })
</script>

<template>
  <router-view />
</template>
