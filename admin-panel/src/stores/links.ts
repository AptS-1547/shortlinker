import { defineStore } from 'pinia'
import { ref } from 'vue'
import { LinkAPI, type SerializableShortLink, type LinkPayload } from '@/services/api'

export const useLinksStore = defineStore('links', () => {
  const links = ref<SerializableShortLink[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAllLinks() {
    loading.value = true
    error.value = null
    try {
      const linksData = await LinkAPI.fetchAll()
      // 将 Record<string, SerializableShortLink> 转换为数组
      links.value = Object.values(linksData)
    } catch (err: any) {
      error.value = err.response?.data?.error || 'Failed to fetch links'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function createNewLink(data: LinkPayload) {
    loading.value = true
    error.value = null
    try {
      await LinkAPI.create(data)
      // 重新获取数据以确保同步
      await fetchAllLinks()
    } catch (err: any) {
      error.value = err.response?.data?.error || 'Failed to create link'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateExistingLink(code: string, data: LinkPayload) {
    loading.value = true
    error.value = null
    try {
      await LinkAPI.update(code, data)
      // 重新获取数据以确保同步
      await fetchAllLinks()
    } catch (err: any) {
      error.value = err.response?.data?.error || 'Failed to update link'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteExistingLink(code: string) {
    loading.value = true
    error.value = null
    try {
      await LinkAPI.delete(code)
      // 从本地数组中移除
      links.value = links.value.filter((link) => link.short_code !== code)
    } catch (err: any) {
      error.value = err.response?.data?.error || 'Failed to delete link'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    links,
    loading,
    error,
    fetchLinks: fetchAllLinks,
    createLink: createNewLink,
    updateLink: updateExistingLink,
    deleteLink: deleteExistingLink,
  }
})
