<script setup lang="ts">
import { useAppStore } from '../stores/app'
import Sidebar from '../components/common/Sidebar.vue'
import HeaderBar from '../components/common/HeaderBar.vue'

const appStore = useAppStore()
</script>

<template>
  <div class="main-layout" :class="{ 'sidebar-collapsed': appStore.sidebarCollapsed }">
    <Sidebar />
    <div class="main-content">
      <HeaderBar />
      <div class="content-area">
        <router-view />
      </div>
    </div>
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  transition: margin-left var(--transition-slow);
}

.content-area {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background: var(--bg-secondary);
}
</style>
