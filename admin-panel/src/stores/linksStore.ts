import { create } from 'zustand'
import type { GetLinksQuery, LinkResponse, PostNewLink } from '@/services/api'
import { LinkAPI } from '@/services/api'
import { extractErrorMessage } from '@/utils/errorHandler'
import { linksLogger } from '@/utils/logger'
import { STORAGE_KEYS, Storage } from '@/utils/storage'

// 模块级 AbortController
let fetchController: AbortController | null = null

// 读取持久化的 pageSize
const getPersistedPageSize = (): number => {
  const saved = Storage.get(STORAGE_KEYS.LINKS_PAGE_SIZE)
  const parsed = saved ? parseInt(saved, 10) : 20
  return [10, 20, 50, 100].includes(parsed) ? parsed : 20
}

interface PaginationState {
  page: number
  pageSize: number
  total: number
  totalPages: number
  hasNext: boolean
  hasPrev: boolean
}

interface LinksState {
  links: LinkResponse[]
  fetching: boolean
  creating: boolean
  updating: boolean
  deleting: boolean
  error: string | null
  pagination: PaginationState
  currentQuery: GetLinksQuery

  // Actions
  fetchLinks: (query?: GetLinksQuery) => Promise<void>
  applyFilter: (query: GetLinksQuery) => Promise<void>
  resetFilter: () => Promise<void>
  goToPage: (page: number) => Promise<void>
  setPageSize: (pageSize: number) => Promise<void>
  createLink: (data: PostNewLink) => Promise<void>
  updateLink: (code: string, data: PostNewLink) => Promise<void>
  deleteLink: (code: string) => Promise<void>
}

const initialPagination: PaginationState = {
  page: 1,
  pageSize: getPersistedPageSize(),
  total: 0,
  totalPages: 0,
  hasNext: false,
  hasPrev: false,
}

// 默认查询参数，所有可选字段显式为 null
const defaultQuery: GetLinksQuery = {
  page: null,
  page_size: null,
  created_after: null,
  created_before: null,
  only_expired: null,
  only_active: null,
  search: null,
}

export const useLinksStore = create<LinksState>((set, get) => ({
  links: [],
  fetching: false,
  creating: false,
  updating: false,
  deleting: false,
  error: null,
  pagination: { ...initialPagination },
  currentQuery: { ...defaultQuery },

  fetchLinks: async (query?: GetLinksQuery) => {
    const state = get()
    let targetQuery = query || state.currentQuery

    // 确保 page_size 总是存在，优先使用持久化的值
    if (!targetQuery.page_size) {
      targetQuery = { ...targetQuery, page_size: state.pagination.pageSize }
    }

    // 取消之前的请求
    if (fetchController) {
      fetchController.abort()
    }
    fetchController = new AbortController()
    const signal = fetchController.signal

    set({ fetching: true, error: null })

    try {
      if (query) {
        set({ currentQuery: { ...query } })
      }

      const response = await LinkAPI.fetchPaginated(targetQuery, signal)

      // 检查是否被取消
      if (signal.aborted) {
        return
      }

      if (response?.data && response.pagination) {
        const linksData = response.data || []
        set({
          links: Array.isArray(linksData) ? linksData : [],
          pagination: {
            page: response.pagination.page || 1,
            pageSize: response.pagination.page_size || 10,
            total: response.pagination.total || 0,
            totalPages: response.pagination.total_pages || 0,
            hasNext: response.pagination.page < response.pagination.total_pages,
            hasPrev: response.pagination.page > 1,
          },
        })
      } else {
        set({
          links: [],
          pagination: { ...initialPagination },
        })
        linksLogger.warn('Unexpected API response format:', response)
      }
    } catch (err) {
      // 忽略被取消的请求
      if (err instanceof Error && err.name === 'AbortError') {
        linksLogger.info('[Links] Request aborted')
        return
      }

      set({
        error: extractErrorMessage(err, 'Failed to fetch links'),
        links: [],
        pagination: { ...initialPagination },
      })
      linksLogger.error('Failed to fetch links:', err)
    } finally {
      // 只有当前请求没被取消时才更新 fetching 状态
      if (!signal.aborted) {
        set({ fetching: false })
      }
    }
  },

  applyFilter: async (query: GetLinksQuery) => {
    const newQuery = { ...query, page: 1 }
    set({ currentQuery: newQuery })
    await get().fetchLinks(newQuery)
  },

  resetFilter: async () => {
    set({
      currentQuery: { ...defaultQuery },
      pagination: { ...initialPagination, page: 1 },
    })
    await get().fetchLinks({ ...defaultQuery })
  },

  goToPage: async (page: number) => {
    const currentQuery = get().currentQuery
    const newQuery = { ...currentQuery, page }
    set({ currentQuery: newQuery, pagination: { ...get().pagination, page } })
    await get().fetchLinks(newQuery)
  },

  setPageSize: async (pageSize: number) => {
    Storage.set(STORAGE_KEYS.LINKS_PAGE_SIZE, String(pageSize))
    const currentQuery = get().currentQuery
    const newQuery = { ...currentQuery, page: 1, page_size: pageSize }
    set({
      currentQuery: newQuery,
      pagination: { ...get().pagination, pageSize, page: 1 },
    })
    await get().fetchLinks(newQuery)
  },

  createLink: async (data: PostNewLink) => {
    set({ creating: true, error: null })
    try {
      await LinkAPI.create(data)
      await get().fetchLinks(get().currentQuery)
    } catch (err) {
      set({ error: extractErrorMessage(err, 'Failed to create link') })
      throw err
    } finally {
      set({ creating: false })
    }
  },

  updateLink: async (code: string, data: PostNewLink) => {
    set({ updating: true, error: null })
    try {
      await LinkAPI.update(code, data)
      await get().fetchLinks(get().currentQuery)
    } catch (err) {
      set({ error: extractErrorMessage(err, 'Failed to update link') })
      throw err
    } finally {
      set({ updating: false })
    }
  },

  deleteLink: async (code: string) => {
    set({ deleting: true, error: null })
    try {
      await LinkAPI.delete(code)

      // ✅ 新实现：重新请求列表（和批量删除保持一致，修复分页 bug）
      const { currentQuery, pagination } = get()
      await get().fetchLinks({
        ...currentQuery,
        page: pagination.page,
        page_size: pagination.pageSize,
      })
    } catch (err) {
      set({ error: extractErrorMessage(err, 'Failed to delete link') })
      throw err
    } finally {
      set({ deleting: false })
    }
  },
}))

// Selector for loading state
export const useLinksLoading = () =>
  useLinksStore(
    (state) =>
      state.fetching || state.creating || state.updating || state.deleting,
  )

// 细粒度 Selectors - 避免不必要的重渲染
export const useLinks = () => useLinksStore((state) => state.links)

export const usePagination = () => useLinksStore((state) => state.pagination)

export const useLinksFetching = () => useLinksStore((state) => state.fetching)

export const useLinksError = () => useLinksStore((state) => state.error)

export const useCurrentQuery = () =>
  useLinksStore((state) => state.currentQuery)

// Actions selector - 稳定引用，不会导致重渲染
export const useLinkActions = () =>
  useLinksStore((state) => ({
    fetchLinks: state.fetchLinks,
    applyFilter: state.applyFilter,
    resetFilter: state.resetFilter,
    goToPage: state.goToPage,
    setPageSize: state.setPageSize,
    createLink: state.createLink,
    updateLink: state.updateLink,
    deleteLink: state.deleteLink,
  }))
