<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 flex flex-col">
    <!-- App Header -->
    <AppHeader
      :show-mobile-menu="showMobileMenu"
      @open-health-modal="openHealthModal"
      @toggle-mobile-menu="toggleMobileMenu"
    />

    <!-- Navigation Tabs - Desktop -->
    <div class="bg-gradient-to-r from-slate-700 to-slate-600 dark:from-gray-800 dark:to-gray-700 shadow-sm hidden sm:block">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <nav class="flex items-center gap-2 py-3">
          <router-link
            v-for="item in menuItems"
            :key="item.path"
            :to="item.path"
            :class="[
              'flex items-center space-x-2 px-4 py-2 rounded-full font-medium text-sm transition-all duration-300 ease-in-out transform hover:scale-105',
              $route.path === item.path
                ? 'bg-white/15 dark:bg-white/20 text-white shadow-lg scale-105 backdrop-blur-sm'
                : 'text-slate-300 dark:text-gray-400 hover:text-white hover:bg-white/8 dark:hover:bg-white/10',
            ]"
          >
            <component :is="item.icon" class="w-4 h-4 transition-transform duration-300" />
            <span class="transition-colors duration-300">{{ $t(item.label) }}</span>
          </router-link>
        </nav>
      </div>
    </div>

    <!-- Mobile Menu -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-1"
    >
      <div v-if="showMobileMenu" class="sm:hidden bg-gradient-to-r from-slate-700 to-slate-600 dark:from-gray-800 dark:to-gray-700 shadow-lg">
        <nav class="px-4 py-3 space-y-2">
          <router-link
            v-for="item in menuItems"
            :key="item.path"
            :to="item.path"
            @click="closeMobileMenu"
            :class="[
              'flex items-center space-x-3 px-4 py-3 rounded-lg font-medium text-sm transition-all duration-200 w-full',
              $route.path === item.path
                ? 'bg-white/15 dark:bg-white/20 text-white shadow-md'
                : 'text-slate-300 dark:text-gray-400 hover:text-white hover:bg-white/8 dark:hover:bg-white/10',
            ]"
          >
            <component :is="item.icon" class="w-5 h-5" />
            <span>{{ $t(item.label) }}</span>
          </router-link>

          <!-- Mobile Logout -->
          <button
            @click="handleLogout"
            class="flex items-center space-x-3 px-4 py-3 rounded-lg font-medium text-sm text-slate-300 dark:text-gray-400 hover:text-white hover:bg-white/8 dark:hover:bg-white/10 transition-all duration-200 w-full"
          >
            <LogoutIcon className="w-5 h-5" />
            <span>{{ $t('layout.logout') }}</span>
          </button>
        </nav>
      </div>
    </Transition>

    <!-- Main Content -->
    <main class="flex-1 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-6 w-full">
      <RouterView />
    </main>

    <!-- App Footer -->
    <AppFooter />

    <!-- Health Modal -->
    <HealthModal
      :is-open="showHealthModal"
      :health-data="healthData"
      @close="closeHealthModal"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useHealthStore } from '@/stores/health'
import { storeToRefs } from 'pinia'
import { Squares2X2Icon, LinkIcon as HeroLinkIcon, ChartBarIcon as HeroChartBarIcon } from '@heroicons/vue/24/outline'
import { LogoutIcon } from '@/components/icons'
import AppHeader from '@/components/AppHeader.vue'
import HealthModal from '@/components/HealthModal.vue'
import AppFooter from '@/components/AppFooter.vue'

const router = useRouter()
const authStore = useAuthStore()
const healthStore = useHealthStore()
const { t } = useI18n()
const { status: healthData } = storeToRefs(healthStore)
const { checkHealth } = healthStore

const showHealthModal = ref(false)
const showMobileMenu = ref(false)

const menuItems = [
  { path: '/dashboard', label: 'layout.navigation.dashboard', icon: Squares2X2Icon },
  { path: '/links', label: 'layout.navigation.links', icon: HeroLinkIcon },
  { path: '/analytics', label: 'layout.navigation.analytics', icon: HeroChartBarIcon },
]

const toggleMobileMenu = () => {
  showMobileMenu.value = !showMobileMenu.value
}

const closeMobileMenu = () => {
  showMobileMenu.value = false
}

const openHealthModal = () => {
  showHealthModal.value = true
}

const closeHealthModal = () => {
  showHealthModal.value = false
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
  closeMobileMenu()
}

// 监听路由变化关闭移动端菜单
router.beforeEach(() => {
  closeMobileMenu()
})

onMounted(() => {
  checkHealth()
})
</script>

<style scoped>
/* 胶囊式导航的额外效果 */
nav a {
  position: relative;
  backdrop-filter: blur(10px);
}

nav a.router-link-active {
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.1);
}

nav a:hover:not(.router-link-active) {
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 4px -1px rgba(0, 0, 0, 0.1);
}

/* 移动端优化 */
@media (max-width: 640px) {
  /* 移动端菜单样式 */
  nav a {
    min-height: 48px; /* 增加触摸区域 */
  }

  /* 移动端按钮优化 */
  button {
    min-height: 40px;
    touch-action: manipulation;
  }
}

/* 确保移动端菜单在正确的层级 */
.sm\\:hidden nav {
  position: relative;
  z-index: 40;
}

/* 移动端菜单容器层级优化 */
.sm\\:hidden {
  position: relative;
  z-index: 35;
}
</style>
