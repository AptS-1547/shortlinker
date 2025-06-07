<template>
  <div class="space-y-6">
    <!-- 页面标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900">{{ $t('analytics.title') }}</h1>
      <p class="text-gray-700">{{ $t('analytics.description') }}</p>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <div class="flex items-center">
          <div class="p-3 bg-indigo-50 rounded-lg">
            <LinkIcon className="w-6 h-6 text-indigo-500" />
          </div>
          <div class="ml-4">
            <h3 class="text-sm font-medium text-gray-500">{{ $t('analytics.totalLinks') }}</h3>
            <p class="text-2xl font-bold text-gray-900">{{ totalLinks }}</p>
            <p class="text-xs text-emerald-600">{{ $t('analytics.activeLinks') }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <div class="flex items-center">
          <div class="p-3 bg-emerald-50 rounded-lg">
            <EyeIcon className="w-6 h-6 text-emerald-500" />
          </div>
          <div class="ml-4">
            <h3 class="text-sm font-medium text-gray-500">{{ $t('analytics.totalClicks') }}</h3>
            <p class="text-2xl font-bold text-gray-900">{{ $t('analytics.na') }}</p>
            <p class="text-xs text-gray-500">{{ $t('analytics.featureComingSoon') }}</p>
          </div>
        </div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
        <div class="flex items-center">
          <div class="p-3 bg-amber-50 rounded-lg">
            <ChartBarIcon className="w-6 h-6 text-amber-500" />
          </div>
          <div class="ml-4">
            <h3 class="text-sm font-medium text-gray-500">{{ $t('analytics.thisWeek') }}</h3>
            <p class="text-2xl font-bold text-gray-900">{{ recentLinksCount }}</p>
            <p class="text-xs text-indigo-600">{{ $t('analytics.recentActivity') }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 链接活动图表 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 mb-6 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{{ $t('analytics.recentActivityTitle') }}</h2>

      <div v-if="loading" class="text-center py-8">
        <div class="animate-spin mx-auto mb-4">
          <SpinnerIcon className="h-8 w-8 text-indigo-500" />
        </div>
        <p class="text-gray-500">{{ $t('analytics.loadingActivity') }}</p>
      </div>

      <div v-else-if="recentLinks.length > 0" class="space-y-4">
        <div
          v-for="(link, index) in recentLinks"
          :key="link.short_code"
          class="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
        >
          <div class="flex items-center gap-4">
            <div class="w-8 h-8 bg-indigo-100 rounded-full flex items-center justify-center">
              <span class="text-indigo-600 font-semibold text-sm">{{ index + 1 }}</span>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="font-mono text-sm bg-indigo-100 text-indigo-700 px-2 py-1 rounded">
                  {{ link.short_code }}
                </span>
                <span class="text-gray-500">→</span>
                <span class="text-gray-700 truncate max-w-md">
                  {{ link.target_url }}
                </span>
              </div>
              <p class="text-xs text-gray-500">
                {{ $t('analytics.created') }}: {{ formatDate(link.created_at) }}
                <span v-if="link.expires_at" class="ml-2">
                  • {{ $t('analytics.expires') }}: {{ formatDate(link.expires_at) }}
                </span>
              </p>
            </div>
          </div>
          <div class="text-right">
            <p class="text-sm text-gray-500">{{ $t('analytics.clicks') }}</p>
            <p class="text-lg font-semibold text-gray-900">{{ $t('analytics.na') }}</p>
          </div>
        </div>
      </div>

      <div v-else class="text-center py-12">
        <div
          class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center"
        >
          <ChartBarIcon className="w-8 h-8 text-gray-400" />
        </div>
        <h3 class="text-lg font-medium text-gray-900 mb-2">{{ $t('analytics.noActivityYet') }}</h3>
        <p class="text-gray-700 mb-4">{{ $t('analytics.noActivityDesc') }}</p>
        <router-link
          to="/links"
          class="inline-flex items-center px-4 py-2 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 transition-colors"
        >
          <PlusIcon className="w-4 h-4 mr-2" />
          {{ $t('links.createFirstLink') }}
        </router-link>
      </div>
    </div>

    <!-- 链接状态分布 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 mb-6 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{{ $t('analytics.linkStatusDistribution') }}</h2>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="p-4 bg-emerald-50 rounded-lg border border-emerald-200 transition-all duration-200 hover:shadow-sm hover:-translate-y-0.5">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-emerald-700">{{ $t('analytics.activeLinksCard') }}</p>
              <p class="text-2xl font-bold text-emerald-800">{{ activeLinksCount }}</p>
            </div>
            <div class="p-2 bg-emerald-100 rounded-lg">
              <CheckCircleIcon className="w-6 h-6 text-emerald-600" />
            </div>
          </div>
          <div class="mt-2">
            <div class="bg-emerald-200 rounded-full h-2">
              <div
                class="bg-emerald-500 h-2 rounded-full transition-all duration-500"
                :style="{ width: `${activeLinksPercentage}%` }"
              ></div>
            </div>
            <p class="text-xs text-emerald-700 mt-1">
              {{ activeLinksPercentage.toFixed(1) }}% {{ $t('analytics.ofTotal') }}
            </p>
          </div>
        </div>

        <div class="p-4 bg-red-50 rounded-lg border border-red-200 transition-all duration-200 hover:shadow-sm hover:-translate-y-0.5">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-red-700">{{ $t('analytics.expiredLinksCard') }}</p>
              <p class="text-2xl font-bold text-red-800">{{ expiredLinksCount }}</p>
            </div>
            <div class="p-2 bg-red-100 rounded-lg">
              <ClockIcon className="w-6 h-6 text-red-500" />
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
              {{ expiredLinksPercentage.toFixed(1) }}% {{ $t('analytics.ofTotal') }}
            </p>
          </div>
        </div>

        <div class="p-4 bg-indigo-50 rounded-lg border border-indigo-200 transition-all duration-200 hover:shadow-sm hover:-translate-y-0.5">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-indigo-700">{{ $t('analytics.permanentLinksCard') }}</p>
              <p class="text-2xl font-bold text-indigo-800">{{ permanentLinksCount }}</p>
            </div>
            <div class="p-2 bg-indigo-100 rounded-lg">
              <LinkIcon className="w-6 h-6 text-indigo-600" />
            </div>
          </div>
          <div class="mt-2">
            <div class="bg-indigo-200 rounded-full h-2">
              <div
                class="bg-indigo-500 h-2 rounded-full transition-all duration-500"
                :style="{ width: `${permanentLinksPercentage}%` }"
              ></div>
            </div>
            <p class="text-xs text-indigo-700 mt-1">
              {{ permanentLinksPercentage.toFixed(1) }}% {{ $t('analytics.ofTotal') }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示信息 -->
    <div class="bg-indigo-50 border border-indigo-200 rounded-lg p-4 transition-all duration-200 hover:shadow-sm hover:-translate-y-0.5">
      <div class="flex items-start gap-3">
        <InfoIcon className="w-5 h-5 text-indigo-600 mt-0.5" />
        <div>
          <h4 class="text-sm font-semibold text-indigo-800">{{ $t('analytics.analyticsFeatures') }}</h4>
          <p class="text-sm text-indigo-700 mt-1">
            {{ $t('analytics.analyticsFeaturesDesc') }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useLinksStore } from '@/stores/links'
import { storeToRefs } from 'pinia'
import { LinkIcon, EyeIcon, ChartBarIcon, CheckCircleIcon, ClockIcon, PlusIcon, SpinnerIcon, InfoIcon } from '@/components/icons'

const { t } = useI18n()
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
