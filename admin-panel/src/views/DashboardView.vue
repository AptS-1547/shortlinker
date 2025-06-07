<template>
  <div class="space-y-6">
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900">Dashboard</h1>
      <p class="text-gray-600">Welcome back! Here's an overview of your short links.</p>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
      <!-- 服务状态卡片 -->
      <div
        class="bg-gradient-to-r from-green-50 to-emerald-50 p-4 rounded-xl shadow-sm border-2 border-green-200"
      >
        <div class="flex items-center">
          <div class="p-2 bg-green-100 rounded-lg">
            <div class="flex items-center gap-2">
              <div
                :class="[
                  'w-2 h-2 rounded-full',
                  healthStatus === 'healthy' ? 'bg-green-500 animate-pulse' : 'bg-red-500',
                ]"
              ></div>
              <svg
                class="w-4 h-4 text-green-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-green-700">Service Status</h3>
            <p class="text-xl font-bold text-green-900 capitalize">{{ healthStatus }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200">
        <div class="flex items-center">
          <div class="p-2 bg-blue-50 rounded-lg">
            <svg
              class="w-5 h-5 text-blue-600"
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
          <div class="ml-3">
            <h3 class="text-sm font-medium text-gray-500">Total Links</h3>
            <p class="text-xl font-bold text-gray-900">{{ links.length }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200">
        <div class="flex items-center">
          <div class="p-2 bg-green-50 rounded-lg">
            <svg
              class="w-5 h-5 text-green-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-gray-500">Active Links</h3>
            <p class="text-xl font-bold text-gray-900">{{ activeLinks }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200">
        <div class="flex items-center">
          <div class="p-2 bg-purple-50 rounded-lg">
            <svg
              class="w-5 h-5 text-purple-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-gray-500">Response Time</h3>
            <p class="text-xl font-bold text-gray-900">{{ responseTime }}ms</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 最近的链接 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
      <h2 class="text-lg font-semibold text-gray-900 mb-3">Recent Links</h2>

      <div v-if="loading" class="text-center py-6">
        <div
          class="animate-spin rounded-full h-6 w-6 border-4 border-blue-600 border-t-transparent mx-auto mb-3"
        ></div>
        <p class="text-gray-500">Loading...</p>
      </div>

      <div v-else-if="recentLinks.length === 0" class="text-center py-6">
        <div
          class="w-12 h-12 mx-auto mb-3 bg-gray-100 rounded-full flex items-center justify-center"
        >
          <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
            />
          </svg>
        </div>
        <h3 class="text-base font-medium text-gray-900 mb-1">No links yet</h3>
        <p class="text-gray-600">Create your first short link to get started.</p>
        <router-link
          to="/admin/links"
          class="inline-flex items-center mt-3 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 6v6m0 0v6m0-6h6m-6 0H6"
            />
          </svg>
          Create Link
        </router-link>
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="link in recentLinks"
          :key="link.short_code"
          class="flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
        >
          <div class="flex items-center gap-3 flex-1 min-w-0">
            <span
              class="font-mono text-sm bg-blue-100 text-blue-800 px-2 py-1 rounded flex-shrink-0"
            >
              {{ link.short_code }}
            </span>
            <span class="text-gray-600 truncate">
              {{ link.target_url }}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-500 flex-shrink-0">
              {{ formatDate(link.created_at) }}
            </span>
            <button
              @click="copyShortLink(link.short_code)"
              class="p-1 text-gray-400 hover:text-blue-600 transition-colors"
              title="Copy short link"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                />
              </svg>
            </button>
          </div>
        </div>

        <div class="pt-3 border-t border-gray-200">
          <router-link
            to="/admin/links"
            class="inline-flex items-center text-sm text-blue-600 hover:text-blue-800 font-medium"
          >
            View all links
            <svg class="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5l7 7-7 7"
              />
            </svg>
          </router-link>
        </div>
      </div>
    </div>

    <!-- 系统健康状态 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-5">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-lg font-semibold text-gray-900">System Health</h2>
        <button
          @click="refreshHealth"
          :disabled="healthLoading"
          class="inline-flex items-center px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded-lg text-sm font-medium text-gray-700 transition-colors disabled:opacity-50"
        >
          <svg
            :class="['w-4 h-4 mr-1', { 'animate-spin': healthLoading }]"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          Refresh
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div
          class="p-3 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg border border-blue-200"
        >
          <div class="flex items-center gap-2 mb-1">
            <svg
              class="w-4 h-4 text-blue-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"
              />
            </svg>
            <span class="text-sm font-medium text-blue-800">Storage</span>
          </div>
          <p class="text-lg font-bold text-blue-900">{{ storageType }}</p>
        </div>

        <div
          class="p-3 bg-gradient-to-r from-purple-50 to-violet-50 rounded-lg border border-purple-200"
        >
          <div class="flex items-center gap-2 mb-1">
            <svg
              class="w-4 h-4 text-purple-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span class="text-sm font-medium text-purple-800">Last Check</span>
          </div>
          <p class="text-sm font-medium text-purple-900">{{ lastHealthCheck }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useLinksStore } from '@/stores/links'
import { useHealthStore } from '@/stores/health'
import { storeToRefs } from 'pinia'
import { HealthAPI } from '@/services/api'

const linksStore = useLinksStore()
const healthStore = useHealthStore()
const { links, loading } = storeToRefs(linksStore)
const { status: healthData } = storeToRefs(healthStore)
const { fetchLinks } = linksStore
const { checkHealth } = healthStore

const healthLoading = ref(false)

const activeLinks = computed(() => {
  const now = new Date()
  return links.value.filter((link) => !link.expires_at || new Date(link.expires_at) > now).length
})

const recentLinks = computed(() => {
  return [...links.value]
    .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    .slice(0, 5)
})

const responseTime = computed(() => {
  return (healthData.value as any)?.response_time_ms || 0
})

const formattedUptime = computed(() => {
  const uptime = (healthData.value as any)?.uptime || 0
  const hours = Math.floor(uptime / 3600)
  if (hours > 24) {
    const days = Math.floor(hours / 24)
    return `${days}d ${hours % 24}h`
  }
  return `${hours}h`
})

const healthStatus = computed(() => {
  return healthData.value?.status || 'unknown'
})

const storageType = computed(() => {
  const checks = (healthData.value as any)?.checks
  return checks?.storage?.backend?.storage_type || 'Unknown'
})

const lastHealthCheck = computed(() => {
  if (!healthData.value?.timestamp) return 'Never'
  return new Date(healthData.value.timestamp).toLocaleTimeString('zh-CN')
})

function formatDate(dateString: string) {
  // 确保正确解析 RFC3339 格式的时间
  const date = new Date(dateString)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function copyShortLink(code: string) {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || window.location.origin
  const shortUrl = `${baseUrl}/${code}`

  try {
    await navigator.clipboard.writeText(shortUrl)
    // 可以添加一个简单的提示
    console.log('Link copied to clipboard')
  } catch (err) {
    console.error('Failed to copy link:', err)
  }
}

async function refreshHealth() {
  healthLoading.value = true
  try {
    await checkHealth()
  } catch (error) {
    console.error('Failed to refresh health:', error)
  } finally {
    healthLoading.value = false
  }
}

onMounted(() => {
  fetchLinks()
  checkHealth()
})
</script>
