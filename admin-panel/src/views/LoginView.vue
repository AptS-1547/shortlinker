<template>
  <div
    class="min-h-screen bg-gradient-to-br from-blue-600 via-indigo-700 to-purple-800 flex items-center justify-center px-4"
  >
    <div class="max-w-md w-full space-y-8">
      <div class="text-center">
        <div class="w-20 h-20 mx-auto mb-6 relative">
          <div
            class="absolute inset-0 bg-white/20 rounded-2xl backdrop-blur-sm border border-white/30 animate-pulse"
          ></div>
          <div
            class="absolute inset-2 bg-gradient-to-br from-white to-blue-100 rounded-xl flex items-center justify-center shadow-2xl"
          >
            <svg
              class="w-10 h-10 text-blue-600"
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
        <h2 class="text-3xl font-bold text-white mb-2">Admin Login</h2>
        <p class="text-blue-100">Enter your password to access the admin panel</p>
      </div>

      <form @submit="handleSubmit" class="space-y-6">
        <div class="bg-white/10 backdrop-blur-xl rounded-2xl p-6 border border-white/20 shadow-2xl">
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-white mb-2"> Password </label>
              <input
                type="password"
                v-model="password"
                placeholder="Enter admin password"
                :disabled="isSubmitting"
                class="w-full px-4 py-3 bg-white/20 border border-white/30 rounded-xl text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-white/50 focus:border-transparent transition-all duration-200 backdrop-blur-sm disabled:opacity-50 disabled:cursor-not-allowed"
                required
              />
            </div>

            <div
              v-if="error"
              :class="[
                'p-3 rounded-lg border',
                error.includes('Authenticating') || error.includes('Verifying')
                  ? 'bg-blue-500/20 border-blue-400/30'
                  : 'bg-red-500/20 border-red-400/30',
              ]"
            >
              <div class="flex items-center gap-2">
                <div
                  v-if="error.includes('Authenticating') || error.includes('Verifying')"
                  class="animate-spin rounded-full h-4 w-4 border-2 border-blue-200 border-t-transparent"
                ></div>
                <svg
                  v-else
                  class="w-4 h-4 text-red-200"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.314 16.5c-.77.833.192 2.5 1.732 2.5z"
                  />
                </svg>
                <p
                  :class="[
                    'text-sm',
                    error.includes('Authenticating') || error.includes('Verifying')
                      ? 'text-blue-100'
                      : 'text-red-100',
                  ]"
                >
                  {{ error }}
                </p>
              </div>
            </div>

            <button
              type="submit"
              :disabled="isSubmitting"
              class="w-full px-6 py-3 bg-gradient-to-r from-white to-blue-50 text-blue-600 font-semibold rounded-xl hover:from-blue-50 hover:to-white focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105 active:scale-95"
            >
              <template v-if="isSubmitting">
                <svg
                  class="w-5 h-5 mr-2 animate-spin inline"
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
                {{
                  error.includes('Authenticating')
                    ? 'Authenticating...'
                    : error.includes('Verifying')
                      ? 'Verifying...'
                      : 'Processing...'
                }}
              </template>
              <template v-else> Login </template>
            </button>
          </div>
        </div>
      </form>

      <!-- 服务器信息提示 -->
      <div class="bg-white/5 backdrop-blur-xl rounded-lg p-4 border border-white/10">
        <div class="flex items-start gap-3">
          <svg
            class="w-5 h-5 text-blue-200 mt-0.5 flex-shrink-0"
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
            <h4 class="text-sm font-semibold text-white">Server Connection</h4>
            <p class="text-sm text-blue-100 mt-1">
              Make sure the ShortLinker service is running on
              <span class="font-mono bg-white/10 px-1 rounded">
                {{ apiBaseUrl }}
              </span>
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { AuthAPI, HealthAPI } from '@/services/api'

const router = useRouter()
const authStore = useAuthStore()

const password = ref('')
const isSubmitting = ref(false)
const error = ref('')

const apiBaseUrl = computed(() => {
  return import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'
})

const handleSubmit = async (e: Event) => {
  e.preventDefault()

  if (!password.value.trim()) {
    error.value = 'Please enter a password'
    return
  }

  isSubmitting.value = true
  error.value = ''

  try {
    // 调用登录API验证密码
    error.value = 'Authenticating...'

    const authResponse = await AuthAPI.login({ password: password.value.trim() })

    // 存储验证通过的密码作为token
    authStore.login(authResponse.token)

    // 验证访问权限
    error.value = 'Verifying access...'
    await HealthAPI.check()

    // 跳转到管理页面
    router.push('/admin/dashboard')
  } catch (err) {
    console.error('Authentication failed:', err)

    // 清除可能已存储的token
    authStore.logout()

    if (err instanceof Error) {
      if (err.message.includes('Network Error') || err.message.includes('ECONNREFUSED')) {
        error.value = 'Cannot connect to server. Please check if the service is running.'
      } else if (err.message.includes('401')) {
        error.value = 'Invalid password or unauthorized access'
      } else if (err.message.includes('404')) {
        error.value = 'Authentication endpoint not found. Please check server configuration.'
      } else if (err.message.includes('500')) {
        error.value = 'Server error. Please try again later.'
      } else {
        error.value = `Server error: ${err.message}`
      }
    } else {
      error.value = 'Authentication failed. Please try again.'
    }
  } finally {
    isSubmitting.value = false
  }
}
</script>
