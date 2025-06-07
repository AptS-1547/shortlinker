<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Top Header -->
    <header class="bg-gradient-to-r from-blue-600 to-indigo-700 text-white shadow-lg">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <!-- Logo and Title -->
          <div class="flex items-center space-x-4">
            <div class="flex items-center space-x-3">
              <div class="w-8 h-8 bg-white/20 rounded-lg flex items-center justify-center">
                <svg
                  class="w-5 h-5 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
                  />
                </svg>
              </div>
              <div>
                <h1 class="text-xl font-bold">ShortLinker Admin</h1>
                <p class="text-blue-100 text-sm">Manage your short links</p>
              </div>
            </div>
          </div>

          <!-- Right Side -->
          <div class="flex items-center space-x-4">
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
                      ? 'bg-green-400'
                      : healthStatus === 'unhealthy'
                        ? 'bg-red-400'
                        : 'bg-yellow-400',
                  ]"
                ></div>
                <!-- 呼吸灯外圈动画 -->
                <div
                  v-if="healthStatus === 'healthy'"
                  class="absolute inset-0 w-3 h-3 bg-green-400 rounded-full animate-breathing opacity-75"
                ></div>
                <div
                  v-else-if="healthStatus === 'unhealthy'"
                  class="absolute inset-0 w-3 h-3 bg-red-400 rounded-full animate-ping opacity-75"
                ></div>
                <div
                  v-else
                  class="absolute inset-0 w-3 h-3 bg-yellow-400 rounded-full animate-pulse opacity-75"
                ></div>
              </div>
              <span class="text-sm font-medium capitalize">{{ healthStatus }}</span>
              <svg
                class="w-3 h-3 text-white/60"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>

            <!-- Logout -->
            <button
              @click="handleLogout"
              class="flex items-center space-x-2 text-blue-100 hover:text-white transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                />
              </svg>
              <span class="text-sm font-medium">Logout</span>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs -->
    <div class="bg-white border-b border-gray-200 shadow-sm">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <nav class="flex space-x-8">
          <router-link
            v-for="item in menuItems"
            :key="item.path"
            :to="item.path"
            :class="[
              'flex items-center space-x-2 py-4 px-1 border-b-2 font-medium text-sm transition-colors',
              $route.path === item.path
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300',
            ]"
          >
            <component :is="item.icon" class="w-5 h-5" />
            <span>{{ item.label }}</span>
          </router-link>
        </nav>
      </div>
    </div>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <RouterView />
    </main>

    <!-- Health Modal -->
    <HealthModal :is-open="showHealthModal" :health-data="healthData" @close="closeHealthModal" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useHealthStore } from '@/stores/health'
import { storeToRefs } from 'pinia'
import { Squares2X2Icon, LinkIcon, ChartBarIcon } from '@heroicons/vue/24/outline'
import HealthModal from '@/components/HealthModal.vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const healthStore = useHealthStore()
const { status: healthData } = storeToRefs(healthStore)
const { checkHealth } = healthStore

const showHealthModal = ref(false)

const menuItems = [
  { path: '/admin/dashboard', label: 'Dashboard', icon: Squares2X2Icon },
  { path: '/admin/links', label: 'Links', icon: LinkIcon },
  { path: '/admin/analytics', label: 'Analytics', icon: ChartBarIcon },
]

const healthStatus = computed(() => {
  return healthData.value?.status || 'unknown'
})

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
  router.push('/admin/login')
}

// 组件挂载时检查健康状态
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

@keyframes gentle-pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.4;
  }
}

.animate-breathing {
  animation: breathing 2s ease-in-out infinite;
}

.animate-gentle-pulse {
  animation: gentle-pulse 2s ease-in-out infinite;
}

/* 更强的发光效果 */
.glow-green {
  box-shadow:
    0 0 6px #4ade80,
    0 0 12px #4ade80,
    0 0 18px #4ade80;
}

.glow-red {
  box-shadow:
    0 0 6px #f87171,
    0 0 12px #f87171,
    0 0 18px #f87171;
}

/* 健康状态的特殊动画 */
.health-indicator {
  position: relative;
}

.health-indicator.healthy::before {
  content: '';
  position: absolute;
  inset: -2px;
  background: radial-gradient(circle, rgba(74, 222, 128, 0.3) 0%, transparent 70%);
  border-radius: 50%;
  animation: breathing 2s ease-in-out infinite;
}

.health-indicator.unhealthy::before {
  content: '';
  position: absolute;
  inset: -2px;
  background: radial-gradient(circle, rgba(248, 113, 113, 0.3) 0%, transparent 70%);
  border-radius: 50%;
  animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;
}

/* 原有样式 */
.router-link-active {
  border-color: #3b82f6;
  color: #3b82f6;
}

.router-link-exact-active {
  border-color: #3b82f6;
  color: #3b82f6;
}

nav a:not(.router-link-active) {
  border-color: transparent;
  color: #6b7280;
}

nav a:not(.router-link-active):hover {
  color: #374151;
  border-color: #d1d5db;
}
</style>
