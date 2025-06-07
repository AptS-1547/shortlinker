<template>
  <Transition
    enter-active-class="transition-all duration-300 ease-out"
    leave-active-class="transition-all duration-200 ease-in"
    enter-from-class="opacity-0"
    enter-to-class="opacity-100"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/20 backdrop-blur-md flex items-center justify-center z-50 p-4"
      @click="handleBackdropClick"
    >
      <Transition
        enter-active-class="transition-all duration-300 ease-out"
        leave-active-class="transition-all duration-200 ease-in"
        enter-from-class="scale-90 opacity-0 translate-y-8"
        enter-to-class="scale-100 opacity-100 translate-y-0"
        leave-from-class="scale-100 opacity-100 translate-y-0"
        leave-to-class="scale-100 opacity-0"
      >
        <div
          v-if="isOpen && !isClosing"
          class="bg-white/95 backdrop-blur-xl border border-white/20 rounded-2xl shadow-2xl max-w-lg w-full p-6 max-h-[90vh] overflow-y-auto"
        >
          <div class="flex items-center justify-between mb-6">
            <div>
              <h3 class="text-xl font-bold text-gray-900">{{ $t('health.title') }}</h3>
              <p class="text-sm text-gray-500 mt-1">{{ $t('health.description') }}</p>
            </div>
            <button
              @click="handleClose"
              class="p-2 hover:bg-gray-100/80 rounded-lg transition-all duration-200 hover:scale-110"
            >
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          <div v-if="healthData" class="space-y-6">
            <!-- 总体状态 -->
            <div
              class="p-4 bg-gradient-to-r from-blue-50/80 to-indigo-50/80 backdrop-blur-sm rounded-xl border border-blue-200/30"
            >
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="font-semibold text-gray-900 mb-1">{{ $t('health.overallStatus') }}</h4>
                  <div
                    :class="[
                      'inline-flex items-center gap-2 px-3 py-1 rounded-full text-sm font-medium',
                      getStatusColor(healthData.status),
                    ]"
                  >
                    <component :is="getStatusIcon(healthData.status)" class="w-4 h-4" />
                    {{ $t(`health.${healthData.status.toLowerCase()}`) }}
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-xs text-gray-500">{{ $t('health.responseTime') }}</p>
                  <p class="text-lg font-bold text-green-600">{{ responseTime }}ms</p>
                </div>
              </div>
            </div>

            <!-- 系统信息 -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div
                class="p-4 bg-gradient-to-br from-white/90 to-gray-50/90 backdrop-blur-sm rounded-xl border border-white/30 shadow-lg"
              >
                <h4 class="font-semibold text-gray-900 mb-3 flex items-center gap-2">
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
                      d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  {{ $t('health.uptime') }}
                </h4>
                <p class="text-2xl font-bold text-gray-900">{{ formattedUptime }}</p>
                <p class="text-xs text-gray-500 mt-1">{{ $t('health.sinceLast') }}</p>
              </div>

              <div
                class="p-4 bg-gradient-to-br from-white/90 to-gray-50/90 backdrop-blur-sm rounded-xl border border-white/30 shadow-lg"
              >
                <h4 class="font-semibold text-gray-900 mb-3 flex items-center gap-2">
                  <LinkIcon class="w-4 h-4 text-green-600" />
                  {{ $t('health.totalLinks') }}
                </h4>
                <p class="text-2xl font-bold text-gray-900">{{ linksCount }}</p>
                <p class="text-xs text-gray-500 mt-1">{{ $t('health.activeLinks') }}</p>
              </div>
            </div>

            <!-- 存储后端信息 -->
            <div
              class="p-4 bg-gradient-to-br from-white/90 to-gray-50/90 backdrop-blur-sm rounded-xl border border-white/30 shadow-lg"
            >
              <h4 class="font-semibold text-gray-900 mb-3 flex items-center gap-2">
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
                    d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"
                  />
                </svg>
                {{ $t('health.storageBackend') }}
              </h4>
              <div class="space-y-3">
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600">{{ $t('health.type') }}</span>
                  <span class="font-mono text-sm bg-gray-200 px-2 py-1 rounded">
                    {{ storageType }}
                  </span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600">{{ $t('health.status') }}</span>
                  <div
                    :class="[
                      'inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium',
                      getStatusColor(storageStatus),
                    ]"
                  >
                    <component :is="getStatusIcon(storageStatus)" class="w-3 h-3" />
                    {{ $t(`health.${storageStatus.toLowerCase()}`) }}
                  </div>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600">{{ $t('health.clickTracking') }}</span>
                  <span
                    :class="[
                      'inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium',
                      supportClick ? 'text-green-600 bg-green-100' : 'text-gray-600 bg-gray-100',
                    ]"
                  >
                    <CheckCircleIcon v-if="supportClick" class="w-3 h-3" />
                    <XCircleIcon v-else class="w-3 h-3" />
                    {{ $t(supportClick ? 'health.enabled' : 'health.disabled') }}
                  </span>
                </div>
              </div>
            </div>

            <!-- 时间戳 -->
            <div class="p-3 bg-gray-50/80 backdrop-blur-sm rounded-lg border border-gray-200/30">
              <div class="flex items-center justify-between text-sm">
                <span class="text-gray-600">{{ $t('health.lastUpdated') }}</span>
                <span class="font-mono text-gray-800">{{ formatTimestamp(healthData.timestamp as string) }}</span>
              </div>
            </div>

            <!-- 关闭按钮 -->
            <div class="flex justify-end pt-2">
              <button
                @click="handleClose"
                class="px-6 py-2 text-gray-700 bg-gray-100/70 hover:bg-gray-200/70 text-sm font-semibold rounded-xl transition-all duration-300 backdrop-blur-sm border border-gray-200/50 hover:border-gray-300/50 transform hover:scale-105 active:scale-95"
              >
                {{ $t('health.close') }}
              </button>
            </div>
          </div>

          <div v-else class="text-center py-8">
            <div
              class="animate-spin rounded-full h-8 w-8 border-4 border-blue-600 border-t-transparent mx-auto mb-4"
            ></div>
            <p class="text-gray-500">{{ $t('health.loadingData') }}</p>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { LinkIcon, CheckCircleIcon, XCircleIcon, ExclamationCircleIcon } from '@/components/icons'
import type { HealthResponse } from '@/services/api'

const { t } = useI18n()

interface Props {
  isOpen: boolean
  healthData: HealthResponse | null
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const isClosing = ref(false)

const handleClose = () => {
  isClosing.value = true
  setTimeout(() => {
    emit('close')
    isClosing.value = false
  }, 100) // 缩短延迟时间以配合更快的退出动画
}

const handleBackdropClick = (e: MouseEvent) => {
  if (e.target === e.currentTarget) {
    handleClose()
  }
}

const formatUptime = (seconds: number) => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60

  if (days > 0) {
    return `${days}d ${hours}h ${minutes}m ${secs}s`
  } else if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`
  } else {
    return `${secs}s`
  }
}

const formatTimestamp = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN', {
    hour12: false,
    timeZoneName: 'short',
  })
}

const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'healthy':
      return 'text-green-600 bg-green-100'
    case 'unhealthy':
      return 'text-red-600 bg-red-100'
    case 'degraded':
      return 'text-yellow-600 bg-yellow-100'
    default:
      return 'text-gray-600 bg-gray-100'
  }
}

const getStatusIcon = (status: string) => {
  switch (status.toLowerCase()) {
    case 'healthy':
      return CheckCircleIcon
    case 'unhealthy':
      return XCircleIcon
    default:
      return ExclamationCircleIcon
  }
}

// 计算属性
const formattedUptime = computed(() => {
  const uptime = (props.healthData as any)?.uptime || 0
  return formatUptime(uptime)
})

const responseTime = computed(() => {
  return (props.healthData as any)?.response_time_ms || 0
})

const linksCount = computed(() => {
  const checks = (props.healthData as any)?.checks
  return checks?.storage?.links_count || 0
})

const storageType = computed(() => {
  const checks = (props.healthData as any)?.checks
  return checks?.storage?.backend?.storage_type || 'Unknown'
})

const storageStatus = computed(() => {
  const checks = (props.healthData as any)?.checks
  return checks?.storage?.status || 'unknown'
})

const supportClick = computed(() => {
  const checks = (props.healthData as any)?.checks
  return checks?.storage?.backend?.support_click || false
})
</script>
