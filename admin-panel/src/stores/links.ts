import { defineStore } from 'pinia'
import { ref } from 'vue'
import { LinkAPI } from '@/services/api'
import type { SerializableShortLink, LinkPayload, GetLinksQuery } from '@/services/api'

export const useLinksStore = defineStore('links', () => {
  const links = ref<SerializableShortLink[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 筛选状态
  const currentQuery = ref<GetLinksQuery>({})
  const totalCount = ref(0)
  const currentPage = ref(1)
  const pageSize = ref(20)
  const hasNext = ref(false)
  const hasPrev = ref(false)

  async function createNewLink(data: LinkPayload) {
    loading.value = true
    error.value = null
    try {
      await LinkAPI.create(data)
      // 重新获取数据以确保同步
      await fetchLinks(currentQuery.value)
    } catch (err: unknown) {
      if (err && typeof err === 'object' && 'response' in err) {
        const apiError = err as { response?: { data?: { error?: string } } }
        error.value = apiError.response?.data?.error || 'Failed to create link'
      } else {
        error.value = 'Failed to create link'
      }
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
      await fetchLinks(currentQuery.value)
    } catch (err: unknown) {
      if (err && typeof err === 'object' && 'response' in err) {
        const apiError = err as { response?: { data?: { error?: string } } }
        error.value = apiError.response?.data?.error || 'Failed to update link'
      } else {
        error.value = 'Failed to update link'
      }
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
    } catch (err: unknown) {
      if (err && typeof err === 'object' && 'response' in err) {
        const apiError = err as { response?: { data?: { error?: string } } }
        error.value = apiError.response?.data?.error || 'Failed to delete link'
      } else {
        error.value = 'Failed to delete link'
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  async function fetchLinks(query?: GetLinksQuery) {
    loading.value = true
    error.value = null

    try {
      if (query) {
        currentQuery.value = { ...query }
      }

      const response = await LinkAPI.fetchPaginated(query || currentQuery.value)

      // 处理API响应格式: { data, pagination }
      if (response && response.data && response.pagination) {
        // 安全地提取链接数据
        const linksData = response.data || {}
        links.value = Object.values(linksData)

        // 提取分页信息
        totalCount.value = response.pagination.total || 0
        currentPage.value = response.pagination.page || 1
        pageSize.value = response.pagination.page_size || 10
        hasNext.value = response.pagination.page < response.pagination.total_pages
        hasPrev.value = response.pagination.page > 1
      } else {
        // 如果响应格式不正确，设置空数据
        links.value = []
        totalCount.value = 0
        currentPage.value = 1
        pageSize.value = 10
        hasNext.value = false
        hasPrev.value = false

        console.warn('Unexpected API response format:', response)
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch links'
      console.error('Failed to fetch links:', err)

      // 在错误情况下重置数据
      links.value = []
      totalCount.value = 0
      currentPage.value = 1
      hasNext.value = false
      hasPrev.value = false
    } finally {
      loading.value = false
    }
  }

  // 应用筛选
  async function applyFilter(query: GetLinksQuery) {
    currentQuery.value = { ...query, page: 1 }
    currentPage.value = 1
    await fetchLinks(currentQuery.value)
  }

  // 重置筛选
  async function resetFilter() {
    currentQuery.value = {}
    currentPage.value = 1
    await fetchLinks()
  }

  // 分页
  async function goToPage(page: number) {
    currentQuery.value.page = page
    currentPage.value = page
    await fetchLinks(currentQuery.value)
  }

  return {
    links,
    loading,
    error,
    currentQuery,
    totalCount,
    currentPage,
    pageSize,
    hasNext,
    hasPrev,
    fetchLinks,
    applyFilter,
    resetFilter,
    goToPage,
    createLink: createNewLink,
    updateLink: updateExistingLink,
    deleteLink: deleteExistingLink,
  }
})
