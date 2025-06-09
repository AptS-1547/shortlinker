<template>
  <div class="min-h-screen bg-gradient-to-br from-gray-50 via-indigo-50/30 to-purple-50/50 flex items-center justify-center px-4 relative overflow-hidden">
    <!-- 背景装饰 -->
    <div class="absolute inset-0 overflow-hidden">
      <div class="absolute -top-40 -right-40 w-80 h-80 bg-gradient-to-br from-indigo-400/20 to-purple-400/20 rounded-full blur-3xl"></div>
      <div class="absolute -bottom-40 -left-40 w-80 h-80 bg-gradient-to-tr from-blue-400/20 to-indigo-400/20 rounded-full blur-3xl"></div>
    </div>

    <div class="relative max-w-md w-full space-y-8 z-10">
      <div class="text-center">
        <div class="w-20 h-20 mx-auto mb-8 relative group">
          <div class="absolute inset-0 bg-gradient-to-br from-indigo-100 via-indigo-50 to-purple-50 rounded-3xl border-2 border-indigo-200/50 shadow-xl group-hover:shadow-2xl transition-all duration-300"></div>
          <div class="absolute inset-2 bg-gradient-to-br from-indigo-500 via-indigo-600 to-purple-600 rounded-2xl flex items-center justify-center shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-105">
            <LinkIcon className="w-10 h-10 text-white" />
          </div>
        </div>
        <h2 class="text-3xl font-bold bg-gradient-to-r from-gray-900 via-indigo-900 to-purple-900 bg-clip-text text-transparent mb-3">
          {{ $t('auth.title') }}
        </h2>
        <p class="text-gray-600 text-lg">{{ $t('auth.description') }}</p>
      </div>

      <form @submit="handleSubmit" class="space-y-8">
        <div class="relative bg-white/80 backdrop-blur-sm rounded-2xl p-8 border-2 border-gray-200/50 shadow-2xl transition-all duration-300 hover:shadow-3xl hover:border-indigo-200/50">
          <div class="absolute inset-0 bg-gradient-to-br from-white/40 to-indigo-50/20 rounded-2xl"></div>
          <div class="relative space-y-6">
            <div>
              <label class="block text-sm font-bold text-gray-900 mb-3 uppercase tracking-wide">
                {{ $t('auth.password') }}
              </label>
              <input
                type="password"
                v-model="password"
                :placeholder="$t('auth.passwordPlaceholder')"
                :disabled="isSubmitting"
                class="w-full px-5 py-4 bg-gray-50/80 border-2 border-gray-200/50 rounded-xl text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm hover:shadow-md focus:shadow-lg backdrop-blur-sm"
                required
              />
            </div>

            <div
              v-if="error"
              :class="[
                'p-4 rounded-xl border-2 transition-all duration-300',
                error.includes($t('auth.authenticating')) || error.includes($t('auth.verifying'))
                  ? 'bg-gradient-to-r from-emerald-50/80 to-teal-50/60 border-emerald-200/70 shadow-sm'
                  : 'bg-gradient-to-r from-red-50/80 to-pink-50/60 border-red-200/70 shadow-sm',
              ]"
            >
              <div class="flex items-center gap-3">
                <div
                  v-if="error.includes($t('auth.authenticating')) || error.includes($t('auth.verifying'))"
                  class="animate-spin rounded-full h-5 w-5 border-2 border-emerald-500 border-t-transparent"
                ></div>
                <ExclamationTriangleIcon
                  v-else
                  className="w-5 h-5 text-red-600"
                />
                <p
                  :class="[
                    'text-sm font-medium',
                    error.includes($t('auth.authenticating')) || error.includes($t('auth.verifying'))
                      ? 'text-emerald-800'
                      : 'text-red-800',
                  ]"
                >
                  {{ error }}
                </p>
              </div>
            </div>

            <button
              type="submit"
              :disabled="isSubmitting"
              class="w-full px-8 py-4 bg-gradient-to-r from-indigo-500 via-indigo-600 to-purple-600 text-white font-bold rounded-xl hover:from-indigo-600 hover:via-indigo-700 hover:to-purple-700 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-300 shadow-lg hover:shadow-xl transform hover:scale-105 active:scale-95 uppercase tracking-wide"
            >
              <template v-if="isSubmitting">
                <RefreshIcon className="w-5 h-5 mr-3 animate-spin inline" />
                {{
                  error.includes($t('auth.authenticating'))
                    ? $t('auth.authenticating')
                    : error.includes($t('auth.verifying'))
                      ? $t('auth.verifying')
                      : $t('auth.processing')
                }}
              </template>
              <template v-else>{{ $t('auth.login') }}</template>
            </button>
          </div>
        </div>
      </form>

      <!-- 服务器信息提示 - 现代化设计 -->
      <div class="relative bg-white/80 backdrop-blur-sm rounded-xl p-6 border-2 border-gray-200/50 shadow-lg transition-all duration-300 hover:shadow-xl hover:border-indigo-200/50">
        <div class="absolute inset-0 bg-gradient-to-br from-white/40 to-blue-50/20 rounded-xl"></div>
        <div class="relative flex items-start gap-4">
          <div class="p-2 bg-gradient-to-br from-indigo-100 to-indigo-50 rounded-lg shadow-sm">
            <InfoIcon className="w-6 h-6 text-indigo-600 flex-shrink-0" />
          </div>
          <div>
            <h4 class="text-sm font-bold text-gray-900 uppercase tracking-wide mb-2">{{ $t('auth.serverConnection') }}</h4>
            <p class="text-sm text-gray-700 leading-relaxed" v-html="$t('auth.serverConnectionDesc', { url: `<span class='font-mono bg-gradient-to-r from-gray-100 to-gray-50 px-2 py-1 rounded text-indigo-700 font-semibold border border-gray-200/50'>${apiBaseUrl}</span>` })">
            </p>
          </div>
        </div>
      </div>

      <!-- 项目信息 -->
      <div class="text-center space-y-4">
        <div class="flex items-center justify-center gap-3">
          <div class="w-8 h-8 bg-gradient-to-br from-indigo-100 to-indigo-50 rounded-lg flex items-center justify-center shadow-sm">
            <LinkIcon className="w-5 h-5 text-indigo-700" />
          </div>
          <h3 class="text-xl font-bold bg-gradient-to-r from-gray-800 via-indigo-800 to-purple-800 bg-clip-text text-transparent">
            ShortLinker
          </h3>
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
import { LinkIcon, RefreshIcon, ExclamationTriangleIcon, InfoIcon } from '@/components/icons'
import { useI18n } from 'vue-i18n'

const router = useRouter()
const authStore = useAuthStore()
const { t } = useI18n()

const password = ref('')
const isSubmitting = ref(false)
const error = ref('')

const apiBaseUrl = computed(() => {
  return import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'
})

const handleSubmit = async (e: Event) => {
  e.preventDefault()

  if (!password.value.trim()) {
    error.value = t('auth.errors.passwordRequired')
    return
  }

  isSubmitting.value = true
  error.value = ''

  try {
    error.value = t('auth.authenticating')

    const authResponse = await AuthAPI.login({ password: password.value.trim() })
    authStore.login(authResponse.token)

    error.value = t('auth.verifying')
    await HealthAPI.check()

    router.push('/dashboard')
  } catch (err) {
    console.error('Authentication failed:', err)
    authStore.logout()

    if (err instanceof Error) {
      if (err.message.includes('Network Error') || err.message.includes('ECONNREFUSED')) {
        error.value = t('auth.errors.networkError')
      } else if (err.message.includes('401')) {
        error.value = t('auth.errors.unauthorized')
      } else if (err.message.includes('404')) {
        error.value = t('auth.errors.notFound')
      } else if (err.message.includes('500')) {
        error.value = t('auth.errors.serverError')
      } else {
        error.value = t('auth.errors.authFailed')
      }
    } else {
      error.value = t('auth.errors.authFailed')
    }
  } finally {
    isSubmitting.value = false
  }
}
</script>
