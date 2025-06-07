<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Top Header -->
    <header class="bg-gradient-to-r from-slate-800 to-slate-700 text-white shadow-md">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <!-- Logo and Title -->
          <div class="flex items-center space-x-4">
            <div class="flex items-center space-x-3">
              <div class="w-8 h-8 bg-gradient-to-br from-indigo-500 to-indigo-600 rounded-lg flex items-center justify-center shadow-sm">
                <LinkIcon className="w-5 h-5 text-white" />
              </div>
              <div>
                <h1 class="text-xl font-bold">{{ $t('layout.title') }}</h1>
                <p class="text-slate-300 text-sm">{{ $t('layout.subtitle') }}</p>
              </div>
            </div>
          </div>

          <!-- Right Side -->
          <div class="flex items-center space-x-4">
            <!-- Language Switcher -->
            <div class="relative" ref="languageDropdown">
              <button
                @click="toggleLanguageMenu"
                class="flex items-center space-x-2 bg-white/10 px-3 py-1 rounded-full hover:bg-white/20 transition-all duration-200 transform hover:scale-105"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
                </svg>
                <span class="text-sm font-medium">{{ currentLanguage }}</span>
                <ChevronDownIcon class="w-3 h-3 text-white/60 transition-transform duration-200" :class="{ 'rotate-180': showLanguageMenu }" />
              </button>

              <!-- Language Dropdown -->
              <Transition
                enter-active-class="transition ease-out duration-200"
                enter-from-class="opacity-0 scale-95"
                enter-to-class="opacity-100 scale-100"
                leave-active-class="transition ease-in duration-75"
                leave-from-class="opacity-100 scale-100"
                leave-to-class="opacity-0 scale-95"
              >
                <div
                  v-if="showLanguageMenu"
                  class="absolute right-0 mt-2 w-40 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50"
                >
                  <button
                    @click="changeLanguage('zh')"
                    class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2"
                    :class="{ 'bg-gray-100 font-medium': locale === 'zh' }"
                  >
                    <span>🇨🇳</span>
                    <span>{{ $t('layout.language.chinese') }}</span>
                  </button>
                  <button
                    @click="changeLanguage('en')"
                    class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center space-x-2"
                    :class="{ 'bg-gray-100 font-medium': locale === 'en' }"
                  >
                    <span>🇺🇸</span>
                    <span>{{ $t('layout.language.english') }}</span>
                  </button>
                </div>
              </Transition>
            </div>

            <!-- Health Status - 可点击 -->
            <button
              @click="openHealthModal"
              class="flex items-center space-x-2 bg-white/10 px-3 py-1 rounded-full hover:bg-white/20 transition-all duration-200 transform hover:scale-105"
            >
              <div class="relative">
                <div
                  :class="[
                    'w-3 h-3 rounded-full transition-all duration-300 relative z-10',
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
                  class="absolute inset-0 w-3 h-3 bg-emerald-400 rounded-full animate-breathing opacity-75"
                ></div>
                <div
                  v-else-if="healthStatus === 'unhealthy'"
                  class="absolute inset-0 w-3 h-3 bg-red-400 rounded-full animate-ping opacity-75"
                ></div>
                <div
                  v-else
                  class="absolute inset-0 w-3 h-3 bg-amber-400 rounded-full animate-pulse opacity-75"
                ></div>
              </div>
              <span class="text-sm font-medium capitalize">{{ $t(`layout.health.${healthStatus}`) }}</span>
              <ChevronRightIcon className="w-3 h-3 text-white/60" />
            </button>

            <!-- Logout -->
            <button
              @click="handleLogout"
              class="flex items-center space-x-2 text-slate-300 hover:text-white transition-colors"
            >
              <LogoutIcon className="w-4 h-4" />
              <span class="text-sm font-medium">{{ $t('layout.logout') }}</span>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs -->
    <div class="bg-gradient-to-r from-slate-700 to-slate-600 shadow-sm">
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

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
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
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useHealthStore } from '@/stores/health'
import { storeToRefs } from 'pinia'
import { Squares2X2Icon, LinkIcon as HeroLinkIcon, ChartBarIcon as HeroChartBarIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
import { LinkIcon, ChevronRightIcon, LogoutIcon } from '@/components/icons'
import HealthModal from '@/components/HealthModal.vue'

const router = useRouter()
const authStore = useAuthStore()
const healthStore = useHealthStore()
const { locale, t } = useI18n()
const { status: healthData } = storeToRefs(healthStore)
const { checkHealth } = healthStore

const showHealthModal = ref(false)
const showLanguageMenu = ref(false)
const languageDropdown = ref<HTMLElement>()

const menuItems = [
  { path: '/dashboard', label: 'layout.navigation.dashboard', icon: Squares2X2Icon },
  { path: '/links', label: 'layout.navigation.links', icon: HeroLinkIcon },
  { path: '/analytics', label: 'layout.navigation.analytics', icon: HeroChartBarIcon },
]

const healthStatus = computed(() => {
  return healthData.value?.status || 'unknown'
})

const currentLanguage = computed(() => {
  return locale.value === 'zh' ? '中文' : 'English'
})

const toggleLanguageMenu = () => {
  showLanguageMenu.value = !showLanguageMenu.value
}

const changeLanguage = (newLocale: string) => {
  locale.value = newLocale
  localStorage.setItem('preferred-language', newLocale)
  showLanguageMenu.value = false
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
}

// 点击外部关闭语言菜单
const handleClickOutside = (event: Event) => {
  if (languageDropdown.value && !languageDropdown.value.contains(event.target as Node)) {
    showLanguageMenu.value = false
  }
}

onMounted(() => {
  checkHealth()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
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
</style>
