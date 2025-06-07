<template>
  <div
    class="min-h-screen bg-gradient-to-br from-gray-50 via-gray-100 to-gray-200 flex items-center justify-center px-4"
  >
    <div class="max-w-md w-full space-y-8">
      <div class="text-center">
        <div class="w-20 h-20 mx-auto mb-6 relative">
          <div
            class="absolute inset-0 bg-indigo-100 rounded-2xl border border-indigo-200 shadow-lg"
          ></div>
          <div
            class="absolute inset-2 bg-gradient-to-br from-indigo-500 to-indigo-600 rounded-xl flex items-center justify-center shadow-lg"
          >
            <LinkIcon className="w-10 h-10 text-white" />
          </div>
        </div>
        <h2 class="text-3xl font-bold text-gray-900 mb-2">{{ $t('auth.title') }}</h2>
        <p class="text-gray-700">{{ $t('auth.description') }}</p>
      </div>

      <form @submit="handleSubmit" class="space-y-6">
        <div class="bg-white rounded-2xl p-6 border border-gray-200 shadow-xl">
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-900 mb-2">{{ $t('auth.password') }}</label>
              <input
                type="password"
                v-model="password"
                :placeholder="$t('auth.passwordPlaceholder')"
                :disabled="isSubmitting"
                class="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                required
              />
            </div>

            <div
              v-if="error"
              :class="[
                'p-3 rounded-lg border',
                error.includes($t('auth.authenticating')) || error.includes($t('auth.verifying'))
                  ? 'bg-emerald-50 border-emerald-200'
                  : 'bg-red-50 border-red-200',
              ]"
            >
              <div class="flex items-center gap-2">
                <div
                  v-if="error.includes($t('auth.authenticating')) || error.includes($t('auth.verifying'))"
                  class="animate-spin rounded-full h-4 w-4 border-2 border-emerald-500 border-t-transparent"
                ></div>
                <ExclamationTriangleIcon
                  v-else
                  className="w-4 h-4 text-red-500"
                />
                <p
                  :class="[
                    'text-sm',
                    error.includes($t('auth.authenticating')) || error.includes($t('auth.verifying'))
                      ? 'text-emerald-700'
                      : 'text-red-700',
                  ]"
                >
                  {{ error }}
                </p>
              </div>
            </div>

            <button
              type="submit"
              :disabled="isSubmitting"
              class="w-full px-6 py-3 bg-gradient-to-r from-indigo-500 to-indigo-600 text-white font-semibold rounded-xl hover:from-indigo-600 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105 active:scale-95"
            >
              <template v-if="isSubmitting">
                <RefreshIcon className="w-5 h-5 mr-2 animate-spin inline" />
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

      <!-- 服务器信息提示 -->
      <div class="bg-white rounded-lg p-4 border border-gray-200 shadow-lg">
        <div class="flex items-start gap-3">
          <InfoIcon className="w-5 h-5 text-indigo-500 mt-0.5 flex-shrink-0" />
          <div>
            <h4 class="text-sm font-semibold text-gray-900">{{ $t('auth.serverConnection') }}</h4>
            <p class="text-sm text-gray-700 mt-1" v-html="$t('auth.serverConnectionDesc', { url: `<span class='font-mono bg-gray-100 px-1 rounded text-indigo-600'>${apiBaseUrl}</span>` })">
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
