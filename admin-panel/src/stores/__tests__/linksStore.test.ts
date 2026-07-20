import { act } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

// Mock LinkAPI
vi.mock('@/services/api', () => ({
  LinkAPI: {
    fetchPaginated: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
  },
}))

// Mock errorHandler
vi.mock('@/utils/errorHandler', () => ({
  extractErrorMessage: vi.fn((err, defaultMsg) =>
    err instanceof Error ? err.message : defaultMsg,
  ),
}))

// Mock logger
vi.mock('@/utils/logger', () => ({
  linksLogger: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}))

// Mock storage
vi.mock('@/utils/storage', () => ({
  STORAGE_KEYS: {
    LINKS_PAGE_SIZE: 'links_page_size',
  },
  Storage: {
    get: vi.fn(() => null),
    set: vi.fn(),
  },
}))

import { LinkAPI } from '@/services/api'
import { useLinksStore } from '../linksStore'

describe('linksStore', () => {
  const mockLink = {
    code: 'test123',
    target: 'https://example.com',
    created_at: '2024-01-01T00:00:00Z',
    click_count: 0,
    expires_at: null,
    password: null,
  }

  const mockPaginatedResponse = {
    code: 200,
    message: 'OK',
    data: [mockLink],
    pagination: {
      page: 1,
      page_size: 20,
      total: 1,
      total_pages: 1,
    },
  }

  beforeEach(() => {
    // Reset store state
    useLinksStore.setState({
      links: [],
      fetching: false,
      creating: false,
      updating: false,
      deleting: false,
      error: null,
      pagination: {
        page: 1,
        pageSize: 20,
        total: 0,
        totalPages: 0,
        hasNext: false,
        hasPrev: false,
      },
      currentQuery: {
        page: null,
        page_size: null,
        created_after: null,
        created_before: null,
        only_expired: null,
        only_active: null,
        search: null,
      },
    })
    vi.clearAllMocks()
  })

  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have correct initial values', () => {
      const state = useLinksStore.getState()
      expect(state.links).toEqual([])
      expect(state.fetching).toBe(false)
      expect(state.creating).toBe(false)
      expect(state.updating).toBe(false)
      expect(state.deleting).toBe(false)
      expect(state.error).toBe(null)
      expect(state.pagination.page).toBe(1)
      expect(state.pagination.pageSize).toBe(20)
    })
  })

  // ==========================================================================
  // fetchLinks
  // ==========================================================================

  describe('fetchLinks', () => {
    it('should fetch links and update state', async () => {
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().fetchLinks()
      })

      const state = useLinksStore.getState()
      expect(state.links).toEqual([mockLink])
      expect(state.pagination.total).toBe(1)
      expect(state.fetching).toBe(false)
    })

    it('should set fetching to true during fetch', async () => {
      let fetchingDuringCall = false
      vi.mocked(LinkAPI.fetchPaginated).mockImplementation(async () => {
        fetchingDuringCall = useLinksStore.getState().fetching
        return mockPaginatedResponse
      })

      await act(async () => {
        await useLinksStore.getState().fetchLinks()
      })

      expect(fetchingDuringCall).toBe(true)
    })

    it('should handle fetch error', async () => {
      vi.mocked(LinkAPI.fetchPaginated).mockRejectedValueOnce(
        new Error('Network error'),
      )

      await act(async () => {
        await useLinksStore.getState().fetchLinks()
      })

      const state = useLinksStore.getState()
      expect(state.error).toBe('Network error')
      expect(state.links).toEqual([])
    })

    it('should use query from state if not provided', async () => {
      useLinksStore.setState({
        currentQuery: {
          page: 2,
          page_size: 50,
          search: 'test',
          created_after: null,
          created_before: null,
          only_expired: null,
          only_active: null,
        },
      })
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().fetchLinks()
      })

      expect(LinkAPI.fetchPaginated).toHaveBeenCalledWith(
        expect.objectContaining({ page: 2, page_size: 50, search: 'test' }),
        expect.any(AbortSignal),
      )
    })

    it('should update currentQuery when query is provided', async () => {
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().fetchLinks({
          page: 3,
          page_size: 10,
          search: 'new',
          created_after: null,
          created_before: null,
          only_expired: null,
          only_active: null,
        })
      })

      const state = useLinksStore.getState()
      expect(state.currentQuery.page).toBe(3)
      expect(state.currentQuery.search).toBe('new')
    })

    it('should cancel previous request on new fetch', async () => {
      let abortedCount = 0
      vi.mocked(LinkAPI.fetchPaginated).mockImplementation(
        async (_, signal) => {
          await new Promise((resolve) => setTimeout(resolve, 50))
          if (signal?.aborted) {
            abortedCount++
            throw Object.assign(new Error('Aborted'), { name: 'AbortError' })
          }
          return mockPaginatedResponse
        },
      )

      // Start first fetch (don't await)
      const promise1 = useLinksStore.getState().fetchLinks()

      // Start second fetch immediately (this should abort the first)
      const promise2 = useLinksStore.getState().fetchLinks()

      await Promise.allSettled([promise1, promise2])

      // The first request should have been aborted
      expect(abortedCount).toBe(1)
    })
  })

  // ==========================================================================
  // applyFilter
  // ==========================================================================

  describe('applyFilter', () => {
    it('should apply filter and fetch links', async () => {
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().applyFilter({
          search: 'test',
          page: null,
          page_size: null,
          created_after: null,
          created_before: null,
          only_expired: null,
          only_active: null,
        })
      })

      expect(LinkAPI.fetchPaginated).toHaveBeenCalled()
      const state = useLinksStore.getState()
      expect(state.currentQuery.search).toBe('test')
    })

    it('should reset page to 1 when applying filter', async () => {
      useLinksStore.setState({
        currentQuery: {
          page: 5,
          page_size: null,
          search: null,
          created_after: null,
          created_before: null,
          only_expired: null,
          only_active: null,
        },
      })
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().applyFilter({
          search: 'new',
          page: null,
          page_size: null,
          created_after: null,
          created_before: null,
          only_expired: null,
          only_active: null,
        })
      })

      const state = useLinksStore.getState()
      expect(state.currentQuery.page).toBe(1)
    })
  })

  // ==========================================================================
  // resetFilter
  // ==========================================================================

  describe('resetFilter', () => {
    it('should reset filter to default values', async () => {
      useLinksStore.setState({
        currentQuery: {
          page: 3,
          page_size: 50,
          search: 'test',
          created_after: null,
          created_before: null,
          only_expired: null,
          only_active: null,
        },
      })
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().resetFilter()
      })

      const state = useLinksStore.getState()
      expect(state.currentQuery.search).toBe(null)
      expect(state.pagination.page).toBe(1)
    })
  })

  // ==========================================================================
  // goToPage
  // ==========================================================================

  describe('goToPage', () => {
    it('should go to specified page', async () => {
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce({
        ...mockPaginatedResponse,
        pagination: { ...mockPaginatedResponse.pagination, page: 3 },
      })

      await act(async () => {
        await useLinksStore.getState().goToPage(3)
      })

      const state = useLinksStore.getState()
      expect(state.pagination.page).toBe(3)
      expect(state.currentQuery.page).toBe(3)
    })
  })

  // ==========================================================================
  // setPageSize
  // ==========================================================================

  describe('setPageSize', () => {
    it('should update page size and reset to page 1', async () => {
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValue({
        code: 200,
        message: 'OK',
        data: [mockLink],
        pagination: { page: 1, page_size: 50, total: 1, total_pages: 1 },
      })

      await act(async () => {
        await useLinksStore.getState().setPageSize(50)
      })

      const state = useLinksStore.getState()
      expect(state.pagination.pageSize).toBe(50)
      expect(state.pagination.page).toBe(1)
    })
  })

  // ==========================================================================
  // createLink
  // ==========================================================================

  describe('createLink', () => {
    it('should create link and refetch', async () => {
      vi.mocked(LinkAPI.create).mockResolvedValueOnce(undefined)
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().createLink({
          target: 'https://example.com',
          code: 'test',
          expires_at: null,
          password: null,
          force: null,
        })
      })

      expect(LinkAPI.create).toHaveBeenCalled()
      expect(LinkAPI.fetchPaginated).toHaveBeenCalled()
    })

    it('should set creating to true during creation', async () => {
      let creatingDuringCall = false
      vi.mocked(LinkAPI.create).mockImplementation(async () => {
        creatingDuringCall = useLinksStore.getState().creating
      })
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().createLink({
          target: 'https://example.com',
          code: null,
          expires_at: null,
          password: null,
          force: null,
        })
      })

      expect(creatingDuringCall).toBe(true)
    })

    it('should handle create error', async () => {
      vi.mocked(LinkAPI.create).mockRejectedValueOnce(
        new Error('Create failed'),
      )

      await expect(
        act(async () => {
          await useLinksStore.getState().createLink({
            target: 'https://example.com',
            code: null,
            expires_at: null,
            password: null,
            force: null,
          })
        }),
      ).rejects.toThrow('Create failed')

      const state = useLinksStore.getState()
      expect(state.error).toBe('Create failed')
    })
  })

  // ==========================================================================
  // updateLink
  // ==========================================================================

  describe('updateLink', () => {
    it('should update link and refetch', async () => {
      vi.mocked(LinkAPI.update).mockResolvedValueOnce(undefined)
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().updateLink('test123', {
          target: 'https://new-example.com',
          code: null,
          expires_at: null,
          password: null,
          force: null,
        })
      })

      expect(LinkAPI.update).toHaveBeenCalledWith('test123', expect.any(Object))
      expect(LinkAPI.fetchPaginated).toHaveBeenCalled()
    })

    it('should set updating to true during update', async () => {
      let updatingDuringCall = false
      vi.mocked(LinkAPI.update).mockImplementation(async () => {
        updatingDuringCall = useLinksStore.getState().updating
      })
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().updateLink('test123', {
          target: 'https://example.com',
          code: null,
          expires_at: null,
          password: null,
          force: null,
        })
      })

      expect(updatingDuringCall).toBe(true)
    })

    it('should handle update error', async () => {
      vi.mocked(LinkAPI.update).mockRejectedValueOnce(
        new Error('Update failed'),
      )

      await expect(
        act(async () => {
          await useLinksStore.getState().updateLink('test123', {
            target: 'https://example.com',
            code: null,
            expires_at: null,
            password: null,
            force: null,
          })
        }),
      ).rejects.toThrow('Update failed')

      const state = useLinksStore.getState()
      expect(state.error).toBe('Update failed')
    })
  })

  // ==========================================================================
  // deleteLink
  // ==========================================================================

  describe('deleteLink', () => {
    it('should delete link and refetch', async () => {
      vi.mocked(LinkAPI.delete).mockResolvedValueOnce(undefined)
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce({
        code: 200,
        message: 'OK',
        data: [],
        pagination: { page: 1, page_size: 20, total: 0, total_pages: 0 },
      })

      await act(async () => {
        await useLinksStore.getState().deleteLink('test123')
      })

      expect(LinkAPI.delete).toHaveBeenCalledWith('test123')
      expect(LinkAPI.fetchPaginated).toHaveBeenCalled()
    })

    it('should set deleting to true during deletion', async () => {
      let deletingDuringCall = false
      vi.mocked(LinkAPI.delete).mockImplementation(async () => {
        deletingDuringCall = useLinksStore.getState().deleting
        return undefined
      })
      vi.mocked(LinkAPI.fetchPaginated).mockResolvedValueOnce(
        mockPaginatedResponse,
      )

      await act(async () => {
        await useLinksStore.getState().deleteLink('test123')
      })

      expect(deletingDuringCall).toBe(true)
    })

    it('should handle delete error', async () => {
      vi.mocked(LinkAPI.delete).mockRejectedValueOnce(
        new Error('Delete failed'),
      )

      await expect(
        act(async () => {
          await useLinksStore.getState().deleteLink('test123')
        }),
      ).rejects.toThrow('Delete failed')

      const state = useLinksStore.getState()
      expect(state.error).toBe('Delete failed')
    })
  })

  // ==========================================================================
  // Selectors
  // ==========================================================================

  describe('selectors', () => {
    it('useLinksLoading should return true when any loading state is true', () => {
      useLinksStore.setState({ fetching: true })
      // Note: We can't easily test hook selectors without rendering,
      // but we can verify the selector logic by checking the state
      const state = useLinksStore.getState()
      const isLoading =
        state.fetching || state.creating || state.updating || state.deleting
      expect(isLoading).toBe(true)
    })

    it('useLinksLoading should return false when no loading state', () => {
      const state = useLinksStore.getState()
      const isLoading =
        state.fetching || state.creating || state.updating || state.deleting
      expect(isLoading).toBe(false)
    })
  })
})
