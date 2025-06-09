<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Top Header -->
    <header class="bg-gradient-to-r from-slate-800 to-slate-700 text-white shadow-md">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <!-- Logo and Title -->
          <div class="flex items-center space-x-2 sm:space-x-4">
            <div class="flex items-center space-x-2 sm:space-x-3">
              <div class="w-7 h-7 sm:w-8 sm:h-8 bg-gradient-to-br from-indigo-500 to-indigo-600 rounded-lg flex items-center justify-center shadow-sm">
                <LinkIcon className="w-4 h-4 sm:w-5 sm:h-5 text-white" />
              </div>
              <div class="hidden sm:block">
                <h1 class="text-xl font-bold">{{ $t('layout.title') }}</h1>
                <p class="text-slate-300 text-sm">{{ $t('layout.subtitle') }}</p>
              </div>
              <!-- 移动端简化标题 -->
              <div class="sm:hidden">
                <h1 class="text-lg font-bold">{{ $t('layout.title') }}</h1>
              </div>
            </div>
          </div>

          <!-- Right Side -->
          <div class="flex items-center space-x-2 sm:space-x-4">
            <!-- Language Switcher -->
            <LanguageSwitcher />

            <!-- Health Status - 移动端优化 -->
            <button
              @click="openHealthModal"
              class="flex items-center space-x-1 sm:space-x-2 bg-white/10 px-2 py-1 sm:px-3 rounded-full hover:bg-white/20 transition-all duration-200 transform hover:scale-105"
            >
              <div class="relative">
                <div
                  :class="[
                    'w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full transition-all duration-300 relative z-10',
                    healthStatus === 'healthy'
                      ? 'bg-emerald-400'
                      : healthStatus === 'unhealthy'
                        ? 'bg-red-400'
                        : 'bg-amber-400',
                  ]"
                ></div>
                <!-- 呼吸灯外圈动画 -->
                <div
                  v-if="healthStatus === 'healthy'"
                  class="absolute inset-0 w-2.5 h-2.5 sm:w-3 sm:h-3 bg-emerald-400 rounded-full animate-breathing opacity-75"
                ></div>
                <div
                  v-else-if="healthStatus === 'unhealthy'"
                  class="absolute inset-0 w-2.5 h-2.5 sm:w-3 sm:h-3 bg-red-400 rounded-full animate-ping opacity-75"
                ></div>
                <div
                  v-else
                  class="absolute inset-0 w-2.5 h-2.5 sm:w-3 sm:h-3 bg-amber-400 rounded-full animate-pulse opacity-75"
                ></div>
              </div>
              <span class="text-xs sm:text-sm font-medium capitalize hidden sm:inline">{{ $t(`layout.health.${healthStatus}`) }}</span>
              <ChevronRightIcon className="w-2.5 h-2.5 sm:w-3 sm:h-3 text-white/60" />
            </button>

            <!-- Mobile Menu Toggle -->
            <button
              @click="toggleMobileMenu"
              class="sm:hidden flex items-center justify-center w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 transition-all duration-200"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  v-if="!showMobileMenu"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 6h16M4 12h16M4 18h16"
                />
                <path
                  v-else
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>

            <!-- Desktop Logout -->
            <button
              @click="handleLogout"
              class="hidden sm:flex items-center space-x-2 text-slate-300 hover:text-white transition-colors"
            >
              <LogoutIcon className="w-4 h-4" />
              <span class="text-sm font-medium">{{ $t('layout.logout') }}</span>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs - Desktop -->
    <div class="bg-gradient-to-r from-slate-700 to-slate-600 shadow-sm hidden sm:block">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <nav class="flex items-center gap-2 py-3">
          <router-link
            v-for="item in menuItems"
            :key="item.path"
            :to="item.path"
            :class="[
              'flex items-center space-x-2 px-4 py-2 rounded-full font-medium text-sm transition-all duration-300 ease-in-out transform hover:scale-105',
              $route.path === item.path
                ? 'bg-white/15 text-white shadow-lg scale-105 backdrop-blur-sm'
                : 'text-slate-300 hover:text-white hover:bg-white/8',
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
      <div v-if="showMobileMenu" class="sm:hidden bg-gradient-to-r from-slate-700 to-slate-600 shadow-lg">
        <nav class="px-4 py-3 space-y-2">
          <router-link
            v-for="item in menuItems"
            :key="item.path"
            :to="item.path"
            @click="closeMobileMenu"
            :class="[
              'flex items-center space-x-3 px-4 py-3 rounded-lg font-medium text-sm transition-all duration-200 w-full',
              $route.path === item.path
                ? 'bg-white/15 text-white shadow-md'
                : 'text-slate-300 hover:text-white hover:bg-white/8',
            ]"
          >
            <component :is="item.icon" class="w-5 h-5" />
            <span>{{ $t(item.label) }}</span>
          </router-link>

          <!-- Mobile Logout -->
          <button
            @click="handleLogout"
            class="flex items-center space-x-3 px-4 py-3 rounded-lg font-medium text-sm text-slate-300 hover:text-white hover:bg-white/8 transition-all duration-200 w-full"
          >
            <LogoutIcon className="w-5 h-5" />
            <span>{{ $t('layout.logout') }}</span>
          </button>
        </nav>
      </div>
    </Transition>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-6">
      <RouterView />
    </main>

    <!-- Health Modal -->
    <HealthModal
      :is-open="showHealthModal"
      :health-data="healthData"
      @close="closeHealthModal"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useHealthStore } from '@/stores/health'
import { storeToRefs } from 'pinia'
import { Squares2X2Icon, LinkIcon as HeroLinkIcon, ChartBarIcon as HeroChartBarIcon } from '@heroicons/vue/24/outline'
import { LinkIcon, ChevronRightIcon, LogoutIcon } from '@/components/icons'
import HealthModal from '@/components/HealthModal.vue'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'

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

const healthStatus = computed(() => {
  return healthData.value?.status || 'unknown'
})

const toggleMobileMenu = () => {
  showMobileMenu.value = !showMobileMenu.value
}

const closeMobileMenu = () => {
  showMobileMenu.value = false
}

const openHealthModal = () => {
  // 打开模态框前刷新健康状态
  checkHealth()
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
/* 自定义呼吸灯动画 */
@keyframes breathing {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.7;
  }
  50% {
    transform: scale(1.4);
    opacity: 0.3;
  }
}

.animate-breathing {
  animation: breathing 2s ease-in-out infinite;
}

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

/* 增强阴影层次 */
header {
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

/* 移动端优化 */
@media (max-width: 640px) {
  .animate-breathing {
    animation: breathing 2s ease-in-out infinite;
  }

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
