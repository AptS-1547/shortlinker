<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
          Shortlinker Admin
        </h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          请输入管理员 Token 以继续
        </p>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit">
        <div>
          <label for="token" class="sr-only">Admin Token</label>
          <input
            id="token"
            v-model="token"
            type="password"
            required
            class="appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-primary-500 focus:border-primary-500 focus:z-10 sm:text-sm"
            placeholder="Admin Token"
          />
        </div>

        <div v-if="error" class="text-red-600 text-sm text-center">
          {{ error }}
        </div>

        <div>
          <button
            type="submit"
            :disabled="loading"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50"
          >
            {{ loading ? '验证中...' : '登录' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { apiService } from '@/services/api'

const router = useRouter()
const authStore = useAuthStore()

const token = ref('')
const loading = ref(false)
const error = ref('')

async function handleSubmit() {
  if (!token.value.trim()) {
    error.value = 'Please enter admin token'
    return
  }

  loading.value = true
  error.value = ''

  try {
    // 临时设置 token 进行验证
    localStorage.setItem('admin_token', token.value)

    // 测试 API 连接
    await apiService.getAllLinks()

    // 验证成功，正式登录
    authStore.login(token.value)
    router.push('/admin/dashboard')
  } catch (err: any) {
    localStorage.removeItem('admin_token')
    error.value = err.response?.data?.error || 'Invalid token or connection failed'
  } finally {
    loading.value = false
  }
}
</script>
