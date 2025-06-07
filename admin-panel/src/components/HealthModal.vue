<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black/20 backdrop-blur-md flex items-center justify-center z-50 p-4 transition-all duration-300"
    @click="handleBackdropClick"
  >
    <div
      class="bg-white/95 backdrop-blur-xl border border-white/20 rounded-2xl shadow-2xl max-w-lg w-full p-6 transform transition-all duration-300 ease-out max-h-[90vh] overflow-y-auto"
      :class="
        isClosing ? 'scale-90 opacity-0 translate-y-8' : 'scale-100 opacity-100 translate-y-0'
      "
    >
      <div class="flex items-center justify-between mb-6">
        <div>
          <h3 class="text-xl font-bold text-gray-900">System Health Status</h3>
          <p class="text-sm text-gray-500 mt-1">Real-time service monitoring</p>
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
              <h4 class="font-semibold text-gray-900 mb-1">Overall Status</h4>
              <div
                :class="[
                  'inline-flex items-center gap-2 px-3 py-1 rounded-full text-sm font-medium',
                  getStatusColor(healthData.status),
                ]"
              >
                <component :is="getStatusIcon(healthData.status)" class="w-4 h-4" />
                {{ healthData.status.charAt(0).toUpperCase() + healthData.status.slice(1) }}
              </div>
            </div>
            <div class="text-right">
              <p class="text-xs text-gray-500">Response Time</p>
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
              Uptime
            </h4>
            <p class="text-2xl font-bold text-gray-900">{{ formattedUptime }}</p>
            <p class="text-xs text-gray-500 mt-1">Since last restart</p>
          </div>

          <div
            class="p-4 bg-gradient-to-br from-white/90 to-gray-50/90 backdrop-blur-sm rounded-xl border border-white/30 shadow-lg"
          >
            <h4 class="font-semibold text-gray-900 mb-3 flex items-center gap-2">
              <LinkIcon class="w-4 h-4 text-green-600" />
              Total Links
            </h4>
            <p class="text-2xl font-bold text-gray-900">{{ linksCount }}</p>
            <p class="text-xs text-gray-500 mt-1">Active short links</p>
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
            Storage Backend
          </h4>
          <div class="space-y-3">
            <div class="flex justify-between items-center">
              <span class="text-sm text-gray-600">Type</span>
              <span class="font-mono text-sm bg-gray-200 px-2 py-1 rounded">
                {{ storageType }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-sm text-gray-600">Status</span>
              <div
                :class="[
                  'inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium',
                  getStatusColor(storageStatus),
                ]"
              >
                <component :is="getStatusIcon(storageStatus)" class="w-3 h-3" />
                {{ storageStatus.charAt(0).toUpperCase() + storageStatus.slice(1) }}
              </div>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-sm text-gray-600">Click Tracking</span>
              <span
                :class="[
                  'inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium',
                  supportClick ? 'text-green-600 bg-green-100' : 'text-gray-600 bg-gray-100',
                ]"
              >
                <CheckCircleIcon v-if="supportClick" class="w-3 h-3" />
                <XCircleIcon v-else class="w-3 h-3" />
                {{ supportClick ? 'Enabled' : 'Disabled' }}
              </span>
            </div>
          </div>
        </div>

        <!-- 时间戳 -->
        <div class="p-3 bg-gray-50/80 backdrop-blur-sm rounded-lg border border-gray-200/30">
          <div class="flex items-center justify-between text-sm">
            <span class="text-gray-600">Last Updated</span>
            <span class="font-mono text-gray-800">{{ formatTimestamp(healthData.timestamp) }}</span>
          </div>
        </div>

        <!-- 关闭按钮 -->
        <div class="flex justify-end pt-2">
          <button
            @click="handleClose"
            class="px-6 py-2 text-gray-700 bg-gray-100/70 hover:bg-gray-200/70 text-sm font-semibold rounded-xl transition-all duration-300 backdrop-blur-sm border border-gray-200/50 hover:border-gray-300/50 transform hover:scale-105 active:scale-95"
          >
            Close
          </button>
        </div>
      </div>

      <div v-else class="text-center py-8">
        <div
          class="animate-spin rounded-full h-8 w-8 border-4 border-blue-600 border-t-transparent mx-auto mb-4"
        ></div>
        <p class="text-gray-500">Loading health data...</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, h } from 'vue'
import { LinkIcon, CheckCircleIcon, XCircleIcon } from '@heroicons/vue/24/outline'
import type { HealthResponse } from '@/services/api'

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
  }, 300)
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
      return h(
        'svg',
        {
          class: 'w-4 h-4',
          fill: 'currentColor',
          viewBox: '0 0 20 20',
        },
        [
          h('path', {
            fillRule: 'evenodd',
            d: 'M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z',
            clipRule: 'evenodd',
          }),
        ],
      )
    case 'unhealthy':
      return h(
        'svg',
        {
          class: 'w-4 h-4',
          fill: 'currentColor',
          viewBox: '0 0 20 20',
        },
        [
          h('path', {
            fillRule: 'evenodd',
            d: 'M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z',
            clipRule: 'evenodd',
          }),
        ],
      )
    default:
      return h(
        'svg',
        {
          class: 'w-4 h-4',
          fill: 'currentColor',
          viewBox: '0 0 20 20',
        },
        [
          h('path', {
            fillRule: 'evenodd',
            d: 'M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z',
            clipRule: 'evenodd',
          }),
        ],
      )
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
