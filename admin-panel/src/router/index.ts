import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import LoginView from '@/views/LoginView.vue'
import AdminLayout from '@/layouts/AdminLayout.vue'
import DashboardView from '@/views/DashboardView.vue'
import LinksView from '@/views/LinksView.vue'
import AnalyticsView from '@/views/AnalyticsView.vue'

// 从运行时配置或环境变量获取基础路径
const getBasePath = () => {
  // 确保在浏览器环境中运行
  if (typeof window === 'undefined') {
    // 如果在 SSR 环境中，使用构建时的 BASE_URL
    return import.meta.env.BASE_URL || '/'
  }

  // 优先从 window 对象获取（Rust 可以注入）
    if (typeof window !== 'undefined' && (window as any).__APP_CONFIG__) {
      const config = (window as any).__APP_CONFIG__
      if (config.basePath && config.basePath !== '%BASE_PATH%') {
        console.warn('Using base path from Rust config:', config.basePath)
        return config.basePath
      }
    }

  // 其次从 meta 标签获取
  const metaBase = document.querySelector('meta[name="base-path"]')?.getAttribute('content')
  if (metaBase && metaBase !== '%BASE_PATH%') {
    console.warn('Using base path from meta tag:', metaBase)
    return metaBase
  }

  // 最后使用构建时的配置
  console.warn('Using base path from import.meta.env:', import.meta.env.BASE_URL)
  return import.meta.env.BASE_URL || '/'
}

const basePath = getBasePath()

const router = createRouter({
  history: createWebHistory(basePath),
  routes: [
    {
      path: '/',
      redirect: '/dashboard',
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/',
      component: AdminLayout,
      meta: { requiresAuth: true },
      children: [
        {
          path: '',
          redirect: '/dashboard',
        },
        {
          path: 'dashboard',
          name: 'dashboard',
          component: DashboardView,
        },
        {
          path: 'links',
          name: 'links',
          component: LinksView,
        },
        {
          path: 'analytics',
          name: 'analytics',
          component: AnalyticsView,
        },
      ],
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'NotFound',
      redirect: '/dashboard',
    },
  ],
})

router.beforeEach((to) => {
  const authStore = useAuthStore()

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    return { path: '/login' }
  }

  if (to.path === '/login' && authStore.isAuthenticated) {
    return { path: '/dashboard' }
  }
})


export default router
