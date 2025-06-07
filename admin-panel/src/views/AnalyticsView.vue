<template>
  <div class="space-y-6">
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900">Analytics</h1>
      <p class="text-gray-600">View insights and statistics about your short links.</p>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div class="flex items-center">
          <div class="p-3 bg-blue-50 rounded-lg">
            <svg
              class="w-6 h-6 text-blue-600"
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
          <div class="ml-4">
            <h3 class="text-sm font-medium text-gray-500">Total Links</h3>
            <p class="text-2xl font-bold text-gray-900">{{ totalLinks }}</p>
            <p class="text-xs text-green-600">Active links</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div class="flex items-center">
          <div class="p-3 bg-green-50 rounded-lg">
            <svg
              class="w-6 h-6 text-green-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
              />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-sm font-medium text-gray-500">Total Clicks</h3>
            <p class="text-2xl font-bold text-gray-900">N/A</p>
            <p class="text-xs text-gray-500">Feature coming soon</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
        <div class="flex items-center">
          <div class="p-3 bg-purple-50 rounded-lg">
            <svg
              class="w-6 h-6 text-purple-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
              />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-sm font-medium text-gray-500">This Week</h3>
            <p class="text-2xl font-bold text-gray-900">{{ recentLinksCount }}</p>
            <p class="text-xs text-blue-600">Recent activity</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 链接活动图表 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 mb-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">Recent Activity</h2>

      <div v-if="loading" class="text-center py-8">
        <div
          class="animate-spin rounded-full h-8 w-8 border-4 border-blue-600 border-t-transparent mx-auto mb-4"
        ></div>
        <p class="text-gray-500">Loading activity...</p>
      </div>

      <div v-else-if="recentLinks.length > 0" class="space-y-4">
        <div
          v-for="(link, index) in recentLinks"
          :key="link.short_code"
          class="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
        >
          <div class="flex items-center gap-4">
            <div class="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
              <span class="text-blue-600 font-semibold text-sm">{{ index + 1 }}</span>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="font-mono text-sm bg-blue-100 text-blue-800 px-2 py-1 rounded">
                  {{ link.short_code }}
                </span>
                <span class="text-gray-400">→</span>
                <span class="text-gray-600 truncate max-w-md">
                  {{ link.target_url }}
                </span>
              </div>
              <p class="text-xs text-gray-500">
                Created: {{ formatDate(link.created_at) }}
                <span v-if="link.expires_at" class="ml-2">
                  • Expires: {{ formatDate(link.expires_at) }}
                </span>
              </p>
            </div>
          </div>
          <div class="text-right">
            <p class="text-sm text-gray-500">Clicks</p>
            <p class="text-lg font-semibold text-gray-900">N/A</p>
          </div>
        </div>
      </div>

      <div v-else class="text-center py-12">
        <div
          class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center"
        >
          <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            />
          </svg>
        </div>
        <h3 class="text-lg font-medium text-gray-900 mb-2">No activity yet</h3>
        <p class="text-gray-600 mb-4">Create some links to see analytics here.</p>
        <router-link
          to="/admin/links"
          class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 6v6m0 0v6m0-6h6m-6 0H6"
            />
          </svg>
          Create Your First Link
        </router-link>
      </div>
    </div>

    <!-- 链接状态分布 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 mb-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">Link Status Distribution</h2>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="p-4 bg-green-50 rounded-lg border border-green-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-green-800">Active Links</p>
              <p class="text-2xl font-bold text-green-900">{{ activeLinksCount }}</p>
            </div>
            <div class="p-2 bg-green-100 rounded-lg">
              <svg
                class="w-6 h-6 text-green-600"
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
          <div class="mt-2">
            <div class="bg-green-200 rounded-full h-2">
              <div
                class="bg-green-500 h-2 rounded-full transition-all duration-500"
                :style="{ width: `${activeLinksPercentage}%` }"
              ></div>
            </div>
            <p class="text-xs text-green-700 mt-1">
              {{ activeLinksPercentage.toFixed(1) }}% of total
            </p>
          </div>
        </div>

        <div class="p-4 bg-red-50 rounded-lg border border-red-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-red-800">Expired Links</p>
              <p class="text-2xl font-bold text-red-900">{{ expiredLinksCount }}</p>
            </div>
            <div class="p-2 bg-red-100 rounded-lg">
              <svg
                class="w-6 h-6 text-red-600"
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
          </div>
          <div class="mt-2">
            <div class="bg-red-200 rounded-full h-2">
              <div
                class="bg-red-500 h-2 rounded-full transition-all duration-500"
                :style="{ width: `${expiredLinksPercentage}%` }"
              ></div>
            </div>
            <p class="text-xs text-red-700 mt-1">
              {{ expiredLinksPercentage.toFixed(1) }}% of total
            </p>
          </div>
        </div>

        <div class="p-4 bg-blue-50 rounded-lg border border-blue-200">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-blue-800">Permanent Links</p>
              <p class="text-2xl font-bold text-blue-900">{{ permanentLinksCount }}</p>
            </div>
            <div class="p-2 bg-blue-100 rounded-lg">
              <svg
                class="w-6 h-6 text-blue-600"
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
          </div>
          <div class="mt-2">
            <div class="bg-blue-200 rounded-full h-2">
              <div
                class="bg-blue-500 h-2 rounded-full transition-all duration-500"
                :style="{ width: `${permanentLinksPercentage}%` }"
              ></div>
            </div>
            <p class="text-xs text-blue-700 mt-1">
              {{ permanentLinksPercentage.toFixed(1) }}% of total
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示信息 -->
    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
      <div class="flex items-start gap-3">
        <svg
          class="w-5 h-5 text-blue-600 mt-0.5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <div>
          <h4 class="text-sm font-semibold text-blue-800">Analytics Features</h4>
          <p class="text-sm text-blue-700 mt-1">
            Click tracking and detailed analytics are coming soon! Currently showing basic link
            statistics.
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useLinksStore } from '@/stores/links'
import { storeToRefs } from 'pinia'

const linksStore = useLinksStore()
const { links, loading } = storeToRefs(linksStore)
const { fetchLinks } = linksStore

const totalLinks = computed(() => links.value.length)

const recentLinks = computed(() => {
  return [...links.value]
    .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    .slice(0, 7)
})

const recentLinksCount = computed(() => {
  const oneWeekAgo = new Date()
  oneWeekAgo.setDate(oneWeekAgo.getDate() - 7)

  return links.value.filter((link) => new Date(link.created_at) > oneWeekAgo).length
})

const activeLinksCount = computed(() => {
  const now = new Date()
  return links.value.filter((link) => !link.expires_at || new Date(link.expires_at) > now).length
})

const expiredLinksCount = computed(() => {
  const now = new Date()
  return links.value.filter((link) => link.expires_at && new Date(link.expires_at) <= now).length
})

const permanentLinksCount = computed(() => {
  return links.value.filter((link) => !link.expires_at).length
})

const activeLinksPercentage = computed(() => {
  return totalLinks.value > 0 ? (activeLinksCount.value / totalLinks.value) * 100 : 0
})

const expiredLinksPercentage = computed(() => {
  return totalLinks.value > 0 ? (expiredLinksCount.value / totalLinks.value) * 100 : 0
})

const permanentLinksPercentage = computed(() => {
  return totalLinks.value > 0 ? (permanentLinksCount.value / totalLinks.value) * 100 : 0
})

function formatDate(dateString: string) {
  // 确保正确解析 RFC3339 格式的时间
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

onMounted(() => {
  fetchLinks()
})
</script>
