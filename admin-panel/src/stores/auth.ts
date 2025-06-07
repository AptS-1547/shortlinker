import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('adminToken'))

  const isAuthenticated = computed(() => !!token.value)

  function login(adminToken: string) {
    token.value = adminToken
    localStorage.setItem('adminToken', adminToken)
  }

  function logout() {
    token.value = null
    localStorage.removeItem('adminToken')
  }

  return {
    token,
    isAuthenticated,
    login,
    logout,
  }
})
