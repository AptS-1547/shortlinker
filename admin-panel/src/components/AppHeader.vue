<template>
  <header class="bg-gradient-to-r from-slate-800 to-slate-700 dark:from-gray-900 dark:to-gray-800 text-white shadow-md dark:shadow-gray-900/20">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center justify-between h-16">
        <!-- Logo and Title -->
        <div class="flex items-center space-x-2 sm:space-x-4">
          <div class="flex items-center space-x-2 sm:space-x-3">
            <div class="w-7 h-7 sm:w-8 sm:h-8 bg-gradient-to-br from-indigo-500 to-indigo-600 dark:from-indigo-400 dark:to-indigo-500 rounded-lg flex items-center justify-center shadow-sm">
              <LinkIcon className="w-4 h-4 sm:w-5 sm:h-5 text-white" />
            </div>
            <div class="hidden sm:block">
              <h1 class="text-xl font-bold">{{ $t('layout.title') }}</h1>
              <p class="text-slate-300 dark:text-gray-400 text-sm">{{ $t('layout.subtitle') }}</p>
            </div>
            <!-- 移动端简化标题 -->
            <div class="sm:hidden">
              <h1 class="text-lg font-bold">{{ $t('layout.title') }}</h1>
            </div>
          </div>
        </div>

        <!-- Right Side -->
        <div class="flex items-center space-x-2 sm:space-x-4">
          <!-- Theme Switcher -->
          <ThemeSwitcher />

          <!-- Language Switcher -->
          <LanguageSwitcher />

          <!-- Health Status - 移动端优化 -->
          <button
            @click="openHealthModal"
            class="flex items-center space-x-1 sm:space-x-2 bg-white/10 dark:bg-black/10 px-2 py-1 sm:px-3 rounded-full hover:bg-white/20 dark:hover:bg-black/20 transition-all duration-200 transform hover:scale-105"
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
            <ChevronRightIcon className="w-2.5 h-2.5 sm:w-3 sm:h-3 text-white/60 dark:text-gray-400" />
          </button>

          <!-- Mobile Menu Toggle -->
          <button
            @click="toggleMobileMenu"
            class="sm:hidden flex items-center justify-center w-8 h-8 rounded-full bg-white/10 dark:bg-black/10 hover:bg-white/20 dark:hover:bg-black/20 transition-all duration-200"
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
            class="hidden sm:flex items-center space-x-2 text-slate-300 dark:text-gray-400 hover:text-white transition-colors"
          >
            <LogoutIcon className="w-4 h-4" />
            <span class="text-sm font-medium">{{ $t('layout.logout') }}</span>
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useHealthStore } from '@/stores/health'
import { storeToRefs } from 'pinia'
import { LinkIcon, ChevronRightIcon, LogoutIcon } from '@/components/icons'
import ThemeSwitcher from '@/components/ThemeSwitcher.vue'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'

const router = useRouter()
const authStore = useAuthStore()
const healthStore = useHealthStore()
const { t } = useI18n()
const { status: healthData } = storeToRefs(healthStore)
const { checkHealth } = healthStore

// Props
interface Props {
  showMobileMenu: boolean
}

defineProps<Props>()

// Emits
const emit = defineEmits<{
  openHealthModal: []
  toggleMobileMenu: []
}>()

const healthStatus = computed(() => {
  return healthData.value?.status || 'unknown'
})

const openHealthModal = () => {
  // 打开模态框前刷新健康状态
  checkHealth()
  emit('openHealthModal')
}

const toggleMobileMenu = () => {
  emit('toggleMobileMenu')
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
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

/* 增强阴影层次 */
header {
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

/* 暗夜模式增强 */
header {
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2);
}

/* 移动端优化 */
@media (max-width: 640px) {
  .animate-breathing {
    animation: breathing 2s ease-in-out infinite;
  }

  /* 移动端按钮优化 */
  button {
    min-height: 40px;
    touch-action: manipulation;
  }
}
</style>
