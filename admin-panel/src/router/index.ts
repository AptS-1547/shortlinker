import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import LoginView from '@/views/LoginView.vue'
import AdminLayout from '@/layouts/AdminLayout.vue'
import DashboardView from '@/views/DashboardView.vue'
import LinksView from '@/views/LinksView.vue'
import AnalyticsView from '@/views/AnalyticsView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/admin/dashboard',
    },
    {
      path: '/admin/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/admin',
      component: AdminLayout,
      meta: { requiresAuth: true },
      children: [
        {
          path: '',
          redirect: '/admin/dashboard',
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
  ],
})

router.beforeEach((to) => {
  const authStore = useAuthStore()

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    return '/admin/login'
  }

  if (to.path === '/admin/login' && authStore.isAuthenticated) {
    return '/admin/dashboard'
  }
})

export default router
