import { defineStore } from 'pinia'
import { ref } from 'vue'
import { HealthAPI, type HealthResponse } from '@/services/api'

export const useHealthStore = defineStore('health', () => {
  const status = ref<HealthResponse | null>(null)
  const loading = ref(false)

  async function checkHealth() {
    loading.value = true
    try {
      status.value = await HealthAPI.check()
    } catch (err) {
      status.value = {
        status: 'unhealthy',
        timestamp: new Date().toISOString(),
      }
    } finally {
      loading.value = false
    }
  }

  return {
    status,
    loading,
    checkHealth,
  }
})
