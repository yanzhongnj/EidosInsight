import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('../layouts/MainLayout.vue'),
    children: [
      {
        path: '',
        name: 'library',
        component: () => import('../views/LibraryView.vue'),
        meta: { title: '文献库' },
      },
      {
        path: 'paper/:id',
        name: 'paper-detail',
        component: () => import('../views/PaperDetailView.vue'),
        meta: { title: '文献详情' },
      },
      {
        path: 'reader/:id',
        name: 'reader',
        component: () => import('../views/ReaderView.vue'),
        meta: { title: '阅读器' },
      },
      {
        path: 'search',
        name: 'search',
        component: () => import('../views/SearchView.vue'),
        meta: { title: '搜索' },
      },
      {
        path: 'notes',
        name: 'notes',
        component: () => import('../views/NotesView.vue'),
        meta: { title: '笔记' },
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('../views/SettingsView.vue'),
        meta: { title: '设置' },
      },
      {
        path: 'automation',
        name: 'automation',
        component: () => import('../views/AutomationView.vue'),
        meta: { title: '自动化' },
      },
    ],
  },
  {
    path: '/onboarding',
    name: 'onboarding',
    component: () => import('../views/OnboardingView.vue'),
    meta: { title: '欢迎使用 EidosInsight' },
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to, _from, next) => {
  if (to.meta.title) {
    document.title = `${to.meta.title} - EidosInsight`
  }
  next()
})

export default router
