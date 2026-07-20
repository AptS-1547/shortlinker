import { act } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

// Mock HealthAPI
vi.mock('@/services/api', () => ({
  HealthAPI: {
    check: vi.fn(),
  },
}))

import { HealthAPI } from '@/services/api'
import { useHealthStore } from '../healthStore'

describe('healthStore', () => {
  const mockHealthResponse = {
    status: 'healthy' as const,
    timestamp: '2024-01-01T00:00:00Z',
    uptime: 3600,
    response_time_ms: 5,
    checks: {
      storage: {
        status: 'healthy' as const,
        links_count: 100,
        backend: {
          storage_type: 'sqlite',
          support_click: true,
        },
        error: null,
      },
      cache: {
        status: 'healthy' as const,
        cache_type: 'memory',
        bloom_filter_enabled: true,
        negative_cache_enabled: true,
        error: null,
      },
    },
  }

  beforeEach(() => {
    // Reset store state
    useHealthStore.setState({
      status: null,
      loading: false,
      lastFetchTime: null,
      error: null,
    })
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have correct initial values', () => {
      const state = useHealthStore.getState()
      expect(state.status).toBe(null)
      expect(state.loading).toBe(false)
      expect(state.lastFetchTime).toBe(null)
      expect(state.error).toBe(null)
    })
  })

  // ==========================================================================
  // checkHealth
  // ==========================================================================

  describe('checkHealth', () => {
    it('should fetch health status and update state', async () => {
      vi.mocked(HealthAPI.check).mockResolvedValueOnce(mockHealthResponse)

      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      const state = useHealthStore.getState()
      expect(state.status).toEqual(mockHealthResponse)
      expect(state.loading).toBe(false)
      expect(state.lastFetchTime).not.toBe(null)
    })

    it('should set loading to true during fetch', async () => {
      let loadingDuringCall = false
      vi.mocked(HealthAPI.check).mockImplementation(async () => {
        loadingDuringCall = useHealthStore.getState().loading
        return mockHealthResponse
      })

      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(loadingDuringCall).toBe(true)
    })

    it('should skip if already loading', async () => {
      useHealthStore.setState({ loading: true })

      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(HealthAPI.check).not.toHaveBeenCalled()
    })

    it('should use cached result within TTL', async () => {
      vi.mocked(HealthAPI.check).mockResolvedValueOnce(mockHealthResponse)

      // First call
      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(HealthAPI.check).toHaveBeenCalledTimes(1)

      // Advance time by 10 seconds (within 30s TTL)
      vi.advanceTimersByTime(10000)

      // Second call - should use cache
      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(HealthAPI.check).toHaveBeenCalledTimes(1) // Still 1, not 2
    })

    it('should fetch fresh data after TTL expires', async () => {
      vi.mocked(HealthAPI.check).mockResolvedValue(mockHealthResponse)

      // First call
      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(HealthAPI.check).toHaveBeenCalledTimes(1)

      // Advance time by 31 seconds (past 30s TTL)
      vi.advanceTimersByTime(31000)

      // Second call - should fetch fresh
      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(HealthAPI.check).toHaveBeenCalledTimes(2)
    })

    it('should force fetch when force=true', async () => {
      vi.mocked(HealthAPI.check).mockResolvedValue(mockHealthResponse)

      // First call
      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      expect(HealthAPI.check).toHaveBeenCalledTimes(1)

      // Force fetch immediately
      await act(async () => {
        await useHealthStore.getState().checkHealth(true)
      })

      expect(HealthAPI.check).toHaveBeenCalledTimes(2)
    })

    it('should handle error and set unhealthy status', async () => {
      vi.mocked(HealthAPI.check).mockRejectedValueOnce(
        new Error('Network error'),
      )

      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      const state = useHealthStore.getState()
      expect(state.error).toBe('Network error')
      expect(state.status?.status).toBe('unhealthy')
      expect(state.status?.checks.storage.status).toBe('unhealthy')
    })

    it('should set default unhealthy response on error', async () => {
      vi.mocked(HealthAPI.check).mockRejectedValueOnce(new Error('Failed'))

      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      const state = useHealthStore.getState()
      expect(state.status).toEqual({
        status: 'unhealthy',
        timestamp: expect.any(String),
        uptime: 0,
        response_time_ms: 0,
        checks: {
          storage: {
            status: 'unhealthy',
            links_count: 0,
            backend: {
              storage_type: 'unknown',
              support_click: false,
            },
            error: null,
          },
          cache: null,
        },
      })
    })

    it('should use default error message for non-Error objects', async () => {
      vi.mocked(HealthAPI.check).mockRejectedValueOnce('Unknown error')

      await act(async () => {
        await useHealthStore.getState().checkHealth()
      })

      const state = useHealthStore.getState()
      expect(state.error).toBe('Health check failed')
    })
  })
})
