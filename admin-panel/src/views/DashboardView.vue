<template>
  <div class="space-y-6 mb-8">
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900">{{ $t('dashboard.title') }}</h1>
      <p class="text-gray-700">{{ $t('dashboard.description') }}</p>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
      <!-- 服务状态卡片 -->
      <div
        :class="[
          'p-4 rounded-xl shadow-sm border-2 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5',
          healthStatus === 'healthy'
            ? 'bg-gradient-to-r from-emerald-50 to-emerald-100 border-emerald-200'
            : healthStatus === 'unhealthy'
            ? 'bg-gradient-to-r from-red-50 to-red-100 border-red-200'
            : 'bg-gradient-to-r from-amber-50 to-amber-100 border-amber-200'
        ]"
      >
        <div class="flex items-center">
          <div
            :class="[
              'p-2 rounded-lg',
              healthStatus === 'healthy'
                ? 'bg-emerald-100'
                : healthStatus === 'unhealthy'
                ? 'bg-red-100'
                : 'bg-amber-100'
            ]"
          >
            <div class="flex items-center gap-2">
              <div
                :class="[
                  'w-2 h-2 rounded-full',
                  healthStatus === 'healthy'
                    ? 'bg-emerald-500 animate-pulse'
                    : healthStatus === 'unhealthy'
                    ? 'bg-red-500 animate-ping'
                    : 'bg-amber-500 animate-pulse'
                ]"
              ></div>
              <CheckCircleIcon
                v-if="healthStatus === 'healthy'"
                className="w-4 h-4 text-emerald-600"
              />
              <XCircleIcon
                v-else-if="healthStatus === 'unhealthy'"
                className="w-4 h-4 text-red-600"
              />
              <ExclamationCircleIcon
                v-else
                className="w-4 h-4 text-amber-600"
              />
            </div>
          </div>
          <div class="ml-3">
            <h3
              :class="[
                'text-sm font-medium',
                healthStatus === 'healthy'
                  ? 'text-emerald-700'
                  : healthStatus === 'unhealthy'
                  ? 'text-red-700'
                  : 'text-amber-700'
              ]"
            >
              {{ $t('dashboard.serviceStatus') }}
            </h3>
            <p
              :class="[
                'text-xl font-bold capitalize',
                healthStatus === 'healthy'
                  ? 'text-emerald-800'
                  : healthStatus === 'unhealthy'
                  ? 'text-red-800'
                  : 'text-amber-800'
              ]"
            >
              {{ $t(`layout.health.${healthStatus}`) }}
            </p>
          </div>
        </div>
      </div>

      <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <div class="flex items-center">
          <div class="p-2 bg-indigo-50 rounded-lg">
            <LinkIcon className="w-5 h-5 text-indigo-600" />
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-gray-500">{{ $t('dashboard.totalLinks') }}</h3>
            <p class="text-xl font-bold text-gray-900">{{ links.length }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <div class="flex items-center">
          <div class="p-2 bg-emerald-50 rounded-lg">
            <CheckCircleIcon className="w-5 h-5 text-emerald-600" />
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-gray-500">{{ $t('dashboard.activeLinks') }}</h3>
            <p class="text-xl font-bold text-gray-900">{{ activeLinks }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <div class="flex items-center">
          <div class="p-2 bg-amber-50 rounded-lg">
            <ClockIcon className="w-5 h-5 text-amber-600" />
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-gray-500">{{ $t('dashboard.responseTime') }}</h3>
            <p class="text-xl font-bold text-gray-900">{{ responseTime }}ms</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 最近的链接 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-5 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <h2 class="text-lg font-semibold text-gray-900 mb-3">{{ $t('dashboard.recentLinks') }}</h2>

      <div v-if="loading" class="text-center py-6">
        <div class="animate-spin mx-auto mb-3">
          <SpinnerIcon className="h-6 w-6 text-indigo-500" />
        </div>
        <p class="text-gray-500">{{ $t('common.loading') }}</p>
      </div>

      <div v-else-if="recentLinks.length === 0" class="text-center py-6">
        <div
          class="w-12 h-12 mx-auto mb-3 bg-gray-100 rounded-full flex items-center justify-center"
        >
          <LinkIcon className="w-6 h-6 text-gray-400" />
        </div>
        <h3 class="text-base font-medium text-gray-900 mb-1">{{ $t('dashboard.noLinks') }}</h3>
        <p class="text-gray-700">{{ $t('dashboard.noLinksDesc') }}</p>
        <router-link
          to="/links"
          class="inline-flex items-center mt-3 px-3 py-2 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 transition-colors"
        >
          <PlusIcon className="w-4 h-4 mr-2" />
          {{ $t('dashboard.createLink') }}
        </router-link>
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="link in recentLinks"
          :key="link.short_code"
          class="flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors group"
        >
          <div class="flex items-center gap-3 flex-1 min-w-0">
            <button
              @click="copyShortLink(link.short_code)"
              :class="[
                'font-mono text-sm px-3 py-2 rounded-lg transition-all duration-200 border-2',
                copiedLink === link.short_code
                  ? 'bg-emerald-100 text-emerald-700 border-emerald-300 scale-105'
                  : 'bg-gray-100 text-gray-800 border-gray-200 hover:bg-indigo-100 hover:text-indigo-700 hover:border-indigo-300 group-hover:scale-105'
              ]"
              :title="copiedLink === link.short_code ? 'Copied!' : 'Click to copy short link'"
            >
              <div class="flex items-center gap-2">
                <span>{{ link.short_code }}</span>
                <CheckCircleIcon
                  v-if="copiedLink === link.short_code"
                  className="w-3 h-3 text-emerald-600"
                />
                <CopyIcon
                  v-else
                  className="w-3 h-3 opacity-0 group-hover:opacity-100 transition-opacity"
                />
              </div>
            </button>
            <span class="text-gray-500">→</span>
            <span class="text-gray-700 truncate">
              {{ link.target_url }}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-500 flex-shrink-0">
              {{ formatDate(link.created_at) }}
            </span>
          </div>
        </div>

        <div class="pt-3 border-t border-gray-200">
          <router-link
            to="/links"
            class="inline-flex items-center text-sm text-indigo-600 hover:text-indigo-800 font-medium"
          >
            {{ $t('dashboard.viewAllLinks') }}
            <ChevronRightIcon className="w-4 h-4 ml-1" />
          </router-link>
        </div>
      </div>
    </div>

    <!-- 系统健康状态 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-5 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-lg font-semibold text-gray-900">{{ $t('dashboard.systemHealth') }}</h2>
        <button
          @click="refreshHealth"
          :disabled="healthLoading"
          class="inline-flex items-center px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded-lg text-sm font-medium text-gray-700 transition-colors disabled:opacity-50"
        >
          <RefreshIcon
            :class="['w-4 h-4 mr-1', { 'animate-spin': healthLoading }]"
          />
          {{ $t('common.refresh') }}
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div
          class="p-3 bg-gradient-to-r from-indigo-50 to-indigo-100 rounded-lg border border-indigo-200"
        >
          <div class="flex items-center gap-2 mb-1">
            <DatabaseIcon className="w-4 h-4 text-indigo-600" />
            <span class="text-sm font-medium text-indigo-800">{{ $t('dashboard.storage') }}</span>
          </div>
          <p class="text-lg font-bold text-indigo-900">{{ storageType }}</p>
        </div>

        <div
          class="p-3 bg-gradient-to-r from-amber-50 to-amber-100 rounded-lg border border-amber-200"
        >
          <div class="flex items-center gap-2 mb-1">
            <ClockIcon className="w-4 h-4 text-amber-600" />
            <span class="text-sm font-medium text-amber-800">{{ $t('dashboard.lastCheck') }}</span>
          </div>
          <p class="text-sm font-medium text-amber-900">{{ lastHealthCheck }}</p>
        </div>
      </div>
    </div>

    <!-- 复制成功提示 Toast -->
    <div
      v-if="showCopyToast"
      class="fixed top-4 right-4 z-50 bg-emerald-500 text-white px-4 py-3 rounded-lg shadow-xl transform transition-all duration-300 ease-out"
      :class="showCopyToast ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'"
    >
      <div class="flex items-center gap-2">
        <CheckCircleIcon className="w-5 h-5" />
        <span class="font-medium">{{ $t('links.linkCopied') }}</span>
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
import { LinkIcon, CheckCircleIcon, ClockIcon, SpinnerIcon, PlusIcon, CopyIcon, ChevronRightIcon, RefreshIcon, DatabaseIcon, XCircleIcon, ExclamationCircleIcon } from '@/components/icons'
import { useI18n } from 'vue-i18n'

const linksStore = useLinksStore()
const healthStore = useHealthStore()
const { links, loading } = storeToRefs(linksStore)
const { status: healthData } = storeToRefs(healthStore)
const { fetchLinks } = linksStore
const { checkHealth } = healthStore
const { t } = useI18n()

const healthLoading = ref(false)
const copiedLink = ref<string | null>(null)
const showCopyToast = ref(false)

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
  const timestamp = healthData.value?.timestamp
  if (!timestamp) return t('dashboard.never')

  try {
    return new Date(timestamp as string | number | Date).toLocaleTimeString('zh-CN')
  } catch {
    return t('dashboard.never')
  }
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

    // 设置当前复制的链接
    copiedLink.value = code
    showCopyToast.value = true

    // 2秒后重置状态
    setTimeout(() => {
      copiedLink.value = null
    }, 2000)

    // 3秒后隐藏 Toast
    setTimeout(() => {
      showCopyToast.value = false
    }, 3000)

    console.log('Link copied to clipboard:', shortUrl)
  } catch (err) {
    console.error('Failed to copy link:', err)
    // 可以添加错误提示
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
