import { act, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useLinksFilters } from '../useLinksFilters'

describe('useLinksFilters', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have empty search query', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))
      expect(result.current.searchQuery).toBe('')
    })

    it('should have "all" status filter', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))
      expect(result.current.statusFilter).toBe('all')
    })

    it('should have undefined date filters', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))
      expect(result.current.createdAfter).toBeUndefined()
      expect(result.current.createdBefore).toBeUndefined()
    })

    it('should call onFilterChange on mount', () => {
      const onFilterChange = vi.fn()
      renderHook(() => useLinksFilters({ onFilterChange }))

      expect(onFilterChange).toHaveBeenCalledTimes(1)
      expect(onFilterChange).toHaveBeenCalledWith({
        search: null,
        only_active: null,
        only_expired: null,
        created_after: null,
        created_before: null,
        page: 1,
        page_size: null,
      })
    })
  })

  // ==========================================================================
  // setSearchQuery
  // ==========================================================================

  describe('setSearchQuery', () => {
    it('should update search query', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setSearchQuery('test')
      })

      expect(result.current.searchQuery).toBe('test')
    })

    it('should debounce callback', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 300 }),
      )

      // Clear initial call
      onFilterChange.mockClear()

      act(() => {
        result.current.setSearchQuery('test')
      })

      // Should not call immediately
      expect(onFilterChange).not.toHaveBeenCalled()

      // Fast forward debounce time
      act(() => {
        vi.advanceTimersByTime(300)
      })

      expect(onFilterChange).toHaveBeenCalledWith(
        expect.objectContaining({ search: 'test' }),
      )
    })

    it('should cancel previous debounce on rapid changes', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 300 }),
      )

      onFilterChange.mockClear()

      act(() => {
        result.current.setSearchQuery('a')
      })
      act(() => {
        vi.advanceTimersByTime(100)
      })
      act(() => {
        result.current.setSearchQuery('ab')
      })
      act(() => {
        vi.advanceTimersByTime(100)
      })
      act(() => {
        result.current.setSearchQuery('abc')
      })
      act(() => {
        vi.advanceTimersByTime(300)
      })

      // Should only call once with final value
      expect(onFilterChange).toHaveBeenCalledTimes(1)
      expect(onFilterChange).toHaveBeenCalledWith(
        expect.objectContaining({ search: 'abc' }),
      )
    })
  })

  // ==========================================================================
  // setStatusFilter
  // ==========================================================================

  describe('setStatusFilter', () => {
    it('should update status filter to active', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setStatusFilter('active')
      })

      expect(result.current.statusFilter).toBe('active')
    })

    it('should update status filter to expired', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setStatusFilter('expired')
      })

      expect(result.current.statusFilter).toBe('expired')
    })

    it('should call onFilterChange with only_active for active filter', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 0 }),
      )

      onFilterChange.mockClear()

      act(() => {
        result.current.setStatusFilter('active')
      })
      act(() => {
        vi.advanceTimersByTime(0)
      })

      expect(onFilterChange).toHaveBeenCalledWith(
        expect.objectContaining({
          only_active: true,
          only_expired: null,
        }),
      )
    })

    it('should call onFilterChange with only_expired for expired filter', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 0 }),
      )

      onFilterChange.mockClear()

      act(() => {
        result.current.setStatusFilter('expired')
      })
      act(() => {
        vi.advanceTimersByTime(0)
      })

      expect(onFilterChange).toHaveBeenCalledWith(
        expect.objectContaining({
          only_active: null,
          only_expired: true,
        }),
      )
    })
  })

  // ==========================================================================
  // Date filters
  // ==========================================================================

  describe('date filters', () => {
    it('should update createdAfter', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      const date = new Date('2024-01-01')
      act(() => {
        result.current.setCreatedAfter(date)
      })

      expect(result.current.createdAfter).toEqual(date)
    })

    it('should update createdBefore', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      const date = new Date('2024-12-31')
      act(() => {
        result.current.setCreatedBefore(date)
      })

      expect(result.current.createdBefore).toEqual(date)
    })

    it('should call onFilterChange with ISO string dates', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 0 }),
      )

      onFilterChange.mockClear()

      const date = new Date('2024-01-15T00:00:00Z')
      act(() => {
        result.current.setCreatedAfter(date)
      })
      act(() => {
        vi.advanceTimersByTime(0)
      })

      expect(onFilterChange).toHaveBeenCalledWith(
        expect.objectContaining({
          created_after: date.toISOString(),
        }),
      )
    })
  })

  // ==========================================================================
  // clearDateFilters
  // ==========================================================================

  describe('clearDateFilters', () => {
    it('should clear both date filters', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      const afterDate = new Date('2024-01-01')
      const beforeDate = new Date('2024-12-31')

      act(() => {
        result.current.setCreatedAfter(afterDate)
        result.current.setCreatedBefore(beforeDate)
      })

      expect(result.current.createdAfter).toEqual(afterDate)
      expect(result.current.createdBefore).toEqual(beforeDate)

      act(() => {
        result.current.clearDateFilters()
      })

      expect(result.current.createdAfter).toBeUndefined()
      expect(result.current.createdBefore).toBeUndefined()
    })
  })

  // ==========================================================================
  // buildQuery
  // ==========================================================================

  describe('buildQuery', () => {
    it('should return default query', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      const query = result.current.buildQuery()
      expect(query).toEqual({
        search: null,
        only_active: null,
        only_expired: null,
        created_after: null,
        created_before: null,
        page: 1,
        page_size: null,
      })
    })

    it('should include search when set', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setSearchQuery('test')
      })

      const query = result.current.buildQuery()
      expect(query.search).toBe('test')
    })

    it('should include only_active when status is active', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setStatusFilter('active')
      })

      const query = result.current.buildQuery()
      expect(query.only_active).toBe(true)
      expect(query.only_expired).toBeNull()
    })

    it('should include only_expired when status is expired', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setStatusFilter('expired')
      })

      const query = result.current.buildQuery()
      expect(query.only_active).toBeNull()
      expect(query.only_expired).toBe(true)
    })

    it('should include date filters when set', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      const afterDate = new Date('2024-01-01')
      const beforeDate = new Date('2024-12-31')

      act(() => {
        result.current.setCreatedAfter(afterDate)
        result.current.setCreatedBefore(beforeDate)
      })

      const query = result.current.buildQuery()
      expect(query.created_after).toBe(afterDate.toISOString())
      expect(query.created_before).toBe(beforeDate.toISOString())
    })

    it('should return null for empty search string', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      act(() => {
        result.current.setSearchQuery('')
      })

      const query = result.current.buildQuery()
      expect(query.search).toBeNull()
    })
  })

  // ==========================================================================
  // Debounce configuration
  // ==========================================================================

  describe('debounce configuration', () => {
    it('should use default debounce of 300ms', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() => useLinksFilters({ onFilterChange }))

      onFilterChange.mockClear()

      act(() => {
        result.current.setSearchQuery('test')
      })

      act(() => {
        vi.advanceTimersByTime(200)
      })
      expect(onFilterChange).not.toHaveBeenCalled()

      act(() => {
        vi.advanceTimersByTime(100)
      })
      expect(onFilterChange).toHaveBeenCalled()
    })

    it('should use custom debounce time', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 500 }),
      )

      onFilterChange.mockClear()

      act(() => {
        result.current.setSearchQuery('test')
      })

      act(() => {
        vi.advanceTimersByTime(400)
      })
      expect(onFilterChange).not.toHaveBeenCalled()

      act(() => {
        vi.advanceTimersByTime(100)
      })
      expect(onFilterChange).toHaveBeenCalled()
    })
  })

  // ==========================================================================
  // No duplicate calls
  // ==========================================================================

  describe('no duplicate calls', () => {
    it('should not call onFilterChange when setting same value', () => {
      const onFilterChange = vi.fn()
      const { result } = renderHook(() =>
        useLinksFilters({ onFilterChange, debounceMs: 0 }),
      )

      onFilterChange.mockClear()

      act(() => {
        result.current.setSearchQuery('test')
      })
      act(() => {
        vi.advanceTimersByTime(0)
      })

      const callCount = onFilterChange.mock.calls.length

      // Set same value
      act(() => {
        result.current.setSearchQuery('test')
      })
      act(() => {
        vi.advanceTimersByTime(0)
      })

      // Should not have additional calls
      expect(onFilterChange.mock.calls.length).toBe(callCount)
    })
  })
})
