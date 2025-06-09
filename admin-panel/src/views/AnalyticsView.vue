<template>
  <div class="space-y-6">
    <!-- 页面标题 - 现代化设计 -->
    <div class="relative mb-6">
      <div class="absolute inset-0 bg-gradient-to-r from-indigo-500/5 via-purple-500/5 to-pink-500/5 rounded-xl"></div>
      <div class="relative p-4">
        <h1 class="text-2xl font-bold bg-gradient-to-r from-gray-900 via-indigo-900 to-purple-900 bg-clip-text text-transparent">
          {{ $t('analytics.title') }}
        </h1>
        <p class="text-gray-600 mt-1">{{ $t('analytics.description') }}</p>
      </div>
    </div>

    <!-- 统计卡片 - 现代化设计 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
      <div class="group relative bg-white/80 backdrop-blur-sm p-4 rounded-xl shadow-md border border-gray-200/50 transition-all duration-300 hover:shadow-lg hover:-translate-y-0.5 hover:bg-white">
        <div class="absolute inset-0 bg-gradient-to-br from-indigo-50/50 to-purple-50/30 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        <div class="relative flex items-center">
          <div class="p-3 bg-gradient-to-br from-indigo-100 to-indigo-50 rounded-lg shadow-sm group-hover:shadow-md transition-shadow duration-300">
            <LinkIcon className="w-5 h-5 text-indigo-600" />
          </div>
          <div class="ml-3">
            <h3 class="text-xs font-semibold text-gray-600 uppercase tracking-wide">{{ $t('analytics.totalLinks') }}</h3>
            <p class="text-xl font-bold text-gray-900 mt-0.5">{{ totalLinks }}</p>
            <p class="text-xs text-emerald-600 font-medium mt-0.5">{{ $t('analytics.activeLinks') }}</p>
          </div>
        </div>
      </div>

      <div class="group relative bg-white/80 backdrop-blur-sm p-4 rounded-xl shadow-md border border-gray-200/50 transition-all duration-300 hover:shadow-lg hover:-translate-y-0.5 hover:bg-white">
        <div class="absolute inset-0 bg-gradient-to-br from-emerald-50/50 to-teal-50/30 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        <div class="relative flex items-center">
          <div class="p-3 bg-gradient-to-br from-emerald-100 to-emerald-50 rounded-lg shadow-sm group-hover:shadow-md transition-shadow duration-300">
            <EyeIcon className="w-5 h-5 text-emerald-600" />
          </div>
          <div class="ml-3">
            <h3 class="text-xs font-semibold text-gray-600 uppercase tracking-wide">{{ $t('analytics.totalClicks') }}</h3>
            <p class="text-xl font-bold text-gray-900 mt-0.5">{{ $t('analytics.na') }}</p>
            <p class="text-xs text-gray-500 font-medium mt-0.5">{{ $t('analytics.featureComingSoon') }}</p>
          </div>
        </div>
      </div>

      <div class="group relative bg-white/80 backdrop-blur-sm p-4 rounded-xl shadow-md border border-gray-200/50 transition-all duration-300 hover:shadow-lg hover:-translate-y-0.5 hover:bg-white">
        <div class="absolute inset-0 bg-gradient-to-br from-amber-50/50 to-orange-50/30 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        <div class="relative flex items-center">
          <div class="p-3 bg-gradient-to-br from-amber-100 to-amber-50 rounded-lg shadow-sm group-hover:shadow-md transition-shadow duration-300">
            <ChartBarIcon className="w-5 h-5 text-amber-600" />
          </div>
          <div class="ml-3">
            <h3 class="text-xs font-semibold text-gray-600 uppercase tracking-wide">{{ $t('analytics.thisWeek') }}</h3>
            <p class="text-xl font-bold text-gray-900 mt-0.5">{{ recentLinksCount }}</p>
            <p class="text-xs text-indigo-600 font-medium mt-0.5">{{ $t('analytics.recentActivity') }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 链接活动图表 - 现代化设计 -->
    <div class="relative bg-white/90 backdrop-blur-sm rounded-xl shadow-lg border border-gray-200/50 p-5 mb-6 transition-all duration-300 hover:shadow-xl hover:-translate-y-0.5">
      <div class="absolute inset-0 bg-gradient-to-br from-blue-50/30 via-indigo-50/20 to-purple-50/30 rounded-xl"></div>
      <div class="relative">
        <h2 class="text-lg font-bold text-gray-900 mb-4 bg-gradient-to-r from-gray-900 to-indigo-900 bg-clip-text text-transparent">
          {{ $t('analytics.recentActivityTitle') }}
        </h2>

        <div v-if="loading" class="text-center py-8">
          <div class="animate-spin mx-auto mb-4">
            <SpinnerIcon className="h-6 w-6 text-indigo-500" />
          </div>
          <p class="text-gray-600">{{ $t('analytics.loadingActivity') }}</p>
        </div>

        <div v-else-if="recentLinks.length > 0" class="space-y-3">
          <div
            v-for="(link, index) in recentLinks"
            :key="link.short_code"
            class="group flex items-center justify-between p-3 bg-gradient-to-r from-gray-50/80 to-gray-100/40 rounded-lg hover:from-indigo-50/80 hover:to-purple-50/40 transition-all duration-300 border border-gray-200/50 hover:border-indigo-200/70 hover:shadow-sm hover:-translate-y-0.5"
          >
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 bg-gradient-to-br from-indigo-100 to-indigo-50 rounded-full flex items-center justify-center shadow-sm group-hover:shadow-md transition-shadow duration-300">
                <span class="text-indigo-700 font-bold text-xs">{{ index + 1 }}</span>
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span class="font-mono text-xs bg-gradient-to-r from-indigo-100 to-indigo-50 text-indigo-800 px-2 py-1 rounded border border-indigo-200/50 shadow-sm">
                    {{ link.short_code }}
                  </span>
                  <span class="text-gray-400 text-sm">→</span>
                  <span class="text-gray-700 truncate max-w-md font-medium text-sm">
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
              <p class="text-xs text-gray-500 font-medium">{{ $t('analytics.clicks') }}</p>
              <p class="text-lg font-bold text-gray-900">{{ $t('analytics.na') }}</p>
            </div>
          </div>
        </div>

        <div v-else class="text-center py-8">
          <div
            class="w-12 h-12 mx-auto mb-3 bg-gray-100 rounded-full flex items-center justify-center"
          >
            <ChartBarIcon className="w-6 h-6 text-gray-400" />
          </div>
          <h3 class="text-base font-medium text-gray-900 mb-2">{{ $t('analytics.noActivityYet') }}</h3>
          <p class="text-gray-700 mb-3">{{ $t('analytics.noActivityDesc') }}</p>
          <router-link
            to="/links"
            class="inline-flex items-center px-3 py-2 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 transition-colors text-sm"
          >
            <PlusIcon className="w-4 h-4 mr-2" />
            {{ $t('links.createFirstLink') }}
          </router-link>
        </div>
      </div>
    </div>

    <!-- 链接状态分布 - 现代化设计 -->
    <div class="relative bg-white/90 backdrop-blur-sm rounded-xl shadow-lg border border-gray-200/50 p-5 mb-6 transition-all duration-300 hover:shadow-xl hover:-translate-y-0.5">
      <div class="absolute inset-0 bg-gradient-to-br from-green-50/30 via-blue-50/20 to-purple-50/30 rounded-xl"></div>
      <div class="relative">
        <h2 class="text-lg font-bold text-gray-900 mb-4 bg-gradient-to-r from-gray-900 to-emerald-900 bg-clip-text text-transparent">
          {{ $t('analytics.linkStatusDistribution') }}
        </h2>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="group relative p-4 bg-gradient-to-br from-emerald-50/80 to-emerald-100/40 rounded-lg border-2 border-emerald-200/50 transition-all duration-300 hover:shadow-md hover:-translate-y-0.5 hover:border-emerald-300/70">
            <div class="absolute inset-0 bg-gradient-to-br from-emerald-100/20 to-emerald-50/10 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <p class="text-xs font-bold text-emerald-800 uppercase tracking-wide">{{ $t('analytics.activeLinksCard') }}</p>
                  <p class="text-xl font-bold text-emerald-900 mt-1">{{ activeLinksCount }}</p>
                </div>
                <div class="p-2 bg-gradient-to-br from-emerald-200 to-emerald-100 rounded-lg shadow-sm">
                  <CheckCircleIcon className="w-5 h-5 text-emerald-700" />
                </div>
              </div>
              <div class="mt-3">
                <div class="bg-emerald-200/70 rounded-full h-2 shadow-inner">
                  <div
                    class="bg-gradient-to-r from-emerald-500 to-emerald-400 h-2 rounded-full transition-all duration-700 shadow-sm"
                    :style="{ width: `${activeLinksPercentage}%` }"
                  ></div>
                </div>
                <p class="text-xs text-emerald-800 mt-1 font-medium">
                  {{ activeLinksPercentage.toFixed(1) }}% {{ $t('analytics.ofTotal') }}
                </p>
              </div>
            </div>
          </div>

          <div class="group relative p-4 bg-gradient-to-br from-red-50/80 to-red-100/40 rounded-lg border-2 border-red-200/50 transition-all duration-300 hover:shadow-md hover:-translate-y-0.5 hover:border-red-300/70">
            <div class="absolute inset-0 bg-gradient-to-br from-red-100/20 to-red-50/10 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <p class="text-xs font-bold text-red-800 uppercase tracking-wide">{{ $t('analytics.expiredLinksCard') }}</p>
                  <p class="text-xl font-bold text-red-900 mt-1">{{ expiredLinksCount }}</p>
                </div>
                <div class="p-2 bg-gradient-to-br from-red-200 to-red-100 rounded-lg shadow-sm">
                  <ClockIcon className="w-5 h-5 text-red-700" />
                </div>
              </div>
              <div class="mt-3">
                <div class="bg-red-200/70 rounded-full h-2 shadow-inner">
                  <div
                    class="bg-gradient-to-r from-red-500 to-red-400 h-2 rounded-full transition-all duration-700 shadow-sm"
                    :style="{ width: `${expiredLinksPercentage}%` }"
                  ></div>
                </div>
                <p class="text-xs text-red-800 mt-1 font-medium">
                  {{ expiredLinksPercentage.toFixed(1) }}% {{ $t('analytics.ofTotal') }}
                </p>
              </div>
            </div>
          </div>

          <div class="group relative p-4 bg-gradient-to-br from-indigo-50/80 to-indigo-100/40 rounded-lg border-2 border-indigo-200/50 transition-all duration-300 hover:shadow-md hover:-translate-y-0.5 hover:border-indigo-300/70">
            <div class="absolute inset-0 bg-gradient-to-br from-indigo-100/20 to-indigo-50/10 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <p class="text-xs font-bold text-indigo-800 uppercase tracking-wide">{{ $t('analytics.permanentLinksCard') }}</p>
                  <p class="text-xl font-bold text-indigo-900 mt-1">{{ permanentLinksCount }}</p>
                </div>
                <div class="p-2 bg-gradient-to-br from-indigo-200 to-indigo-100 rounded-lg shadow-sm">
                  <LinkIcon className="w-5 h-5 text-indigo-700" />
                </div>
              </div>
              <div class="mt-3">
                <div class="bg-indigo-200/70 rounded-full h-2 shadow-inner">
                  <div
                    class="bg-gradient-to-r from-indigo-500 to-indigo-400 h-2 rounded-full transition-all duration-700 shadow-sm"
                    :style="{ width: `${permanentLinksPercentage}%` }"
                  ></div>
                </div>
                <p class="text-xs text-indigo-800 mt-1 font-medium">
                  {{ permanentLinksPercentage.toFixed(1) }}% {{ $t('analytics.ofTotal') }}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示信息 - 现代化设计 -->
    <div class="relative bg-gradient-to-r from-indigo-50/80 via-blue-50/60 to-purple-50/80 border-2 border-indigo-200/50 rounded-lg p-4 transition-all duration-300 hover:shadow-md hover:-translate-y-0.5 hover:border-indigo-300/70">
      <div class="absolute inset-0 bg-gradient-to-br from-indigo-100/20 to-purple-100/10 rounded-lg"></div>
      <div class="relative flex items-start gap-3">
        <div class="p-1.5 bg-gradient-to-br from-indigo-100 to-indigo-50 rounded-lg shadow-sm">
          <InfoIcon className="w-5 h-5 text-indigo-700" />
        </div>
        <div>
          <h4 class="text-sm font-bold text-indigo-900 mb-1">{{ $t('analytics.analyticsFeatures') }}</h4>
          <p class="text-indigo-800 leading-relaxed text-sm">
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
