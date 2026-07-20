import { act, renderHook } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { usePagination } from '../usePagination'

describe('usePagination', () => {
  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have default values', () => {
      const { result } = renderHook(() => usePagination())
      expect(result.current.currentPage).toBe(1)
      expect(result.current.pageSize).toBe(20)
      expect(result.current.total).toBe(0)
    })

    it('should use custom initial values', () => {
      const { result } = renderHook(() =>
        usePagination({
          initialPage: 5,
          initialPageSize: 50,
          total: 1000,
        }),
      )
      expect(result.current.currentPage).toBe(5)
      expect(result.current.pageSize).toBe(50)
      expect(result.current.total).toBe(1000)
    })
  })

  // ==========================================================================
  // Computed values
  // ==========================================================================

  describe('computed values', () => {
    it('should calculate totalPages correctly', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20 }),
      )
      expect(result.current.totalPages).toBe(5)
    })

    it('should calculate totalPages with remainder', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 105, initialPageSize: 20 }),
      )
      expect(result.current.totalPages).toBe(6)
    })

    it('should return 0 totalPages for empty data', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 0, initialPageSize: 20 }),
      )
      expect(result.current.totalPages).toBe(0)
    })

    it('should calculate hasNext correctly', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 3 }),
      )
      expect(result.current.hasNext).toBe(true)
    })

    it('should return false for hasNext on last page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 5 }),
      )
      expect(result.current.hasNext).toBe(false)
    })

    it('should calculate hasPrev correctly', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 3 }),
      )
      expect(result.current.hasPrev).toBe(true)
    })

    it('should return false for hasPrev on first page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 1 }),
      )
      expect(result.current.hasPrev).toBe(false)
    })

    it('should calculate startIndex correctly', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 3 }),
      )
      expect(result.current.startIndex).toBe(41)
    })

    it('should calculate endIndex correctly', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 3 }),
      )
      expect(result.current.endIndex).toBe(60)
    })

    it('should cap endIndex at total', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 55, initialPageSize: 20, initialPage: 3 }),
      )
      expect(result.current.endIndex).toBe(55)
    })
  })

  // ==========================================================================
  // getVisiblePages
  // ==========================================================================

  describe('getVisiblePages', () => {
    it('should return visible pages around current page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 200, initialPageSize: 20, initialPage: 5 }),
      )
      const pages = result.current.getVisiblePages()
      expect(pages).toContain(3)
      expect(pages).toContain(4)
      expect(pages).toContain(5)
      expect(pages).toContain(6)
      expect(pages).toContain(7)
    })

    it('should handle first page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 200, initialPageSize: 20, initialPage: 1 }),
      )
      const pages = result.current.getVisiblePages()
      expect(pages[0]).toBe(1)
      expect(pages.length).toBeGreaterThanOrEqual(1)
    })

    it('should handle last page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 200, initialPageSize: 20, initialPage: 10 }),
      )
      const pages = result.current.getVisiblePages()
      expect(pages[pages.length - 1]).toBe(10)
    })

    it('should handle small total pages', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 60, initialPageSize: 20, initialPage: 2 }),
      )
      const pages = result.current.getVisiblePages()
      expect(pages).toEqual([1, 2, 3])
    })

    it('should use custom delta', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 200, initialPageSize: 20, initialPage: 5 }),
      )
      const pages = result.current.getVisiblePages(1)
      expect(pages.length).toBeLessThanOrEqual(5)
    })
  })

  // ==========================================================================
  // Navigation functions
  // ==========================================================================

  describe('goToPage', () => {
    it('should go to valid page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20 }),
      )
      act(() => {
        result.current.goToPage(3)
      })
      expect(result.current.currentPage).toBe(3)
    })

    it('should not go to page less than 1', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20 }),
      )
      act(() => {
        result.current.goToPage(0)
      })
      expect(result.current.currentPage).toBe(1)
    })

    it('should not go to page greater than totalPages', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20 }),
      )
      act(() => {
        result.current.goToPage(10)
      })
      expect(result.current.currentPage).toBe(1)
    })
  })

  describe('nextPage', () => {
    it('should go to next page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 2 }),
      )
      act(() => {
        result.current.nextPage()
      })
      expect(result.current.currentPage).toBe(3)
    })

    it('should not go past last page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 5 }),
      )
      act(() => {
        result.current.nextPage()
      })
      expect(result.current.currentPage).toBe(5)
    })
  })

  describe('prevPage', () => {
    it('should go to previous page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 3 }),
      )
      act(() => {
        result.current.prevPage()
      })
      expect(result.current.currentPage).toBe(2)
    })

    it('should not go before first page', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 1 }),
      )
      act(() => {
        result.current.prevPage()
      })
      expect(result.current.currentPage).toBe(1)
    })
  })

  // ==========================================================================
  // setPageSize
  // ==========================================================================

  describe('setPageSize', () => {
    it('should change page size', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20 }),
      )
      act(() => {
        result.current.setPageSize(50)
      })
      expect(result.current.pageSize).toBe(50)
    })

    it('should adjust current page when it exceeds new total pages', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 5 }),
      )
      act(() => {
        result.current.setPageSize(50)
      })
      // 100 / 50 = 2 pages, so page 5 should adjust to page 2
      expect(result.current.currentPage).toBe(2)
    })

    it('should not adjust current page when it is still valid', () => {
      const { result } = renderHook(() =>
        usePagination({ total: 100, initialPageSize: 20, initialPage: 2 }),
      )
      act(() => {
        result.current.setPageSize(50)
      })
      expect(result.current.currentPage).toBe(2)
    })
  })

  // ==========================================================================
  // setTotal
  // ==========================================================================

  describe('setTotal', () => {
    it('should update total', () => {
      const { result } = renderHook(() => usePagination())
      act(() => {
        result.current.setTotal(500)
      })
      expect(result.current.total).toBe(500)
    })

    it('should update totalPages accordingly', () => {
      const { result } = renderHook(() =>
        usePagination({ initialPageSize: 20 }),
      )
      act(() => {
        result.current.setTotal(100)
      })
      expect(result.current.totalPages).toBe(5)
    })
  })

  // ==========================================================================
  // reset
  // ==========================================================================

  describe('reset', () => {
    it('should reset to initial values', () => {
      const { result } = renderHook(() =>
        usePagination({
          initialPage: 2,
          initialPageSize: 30,
          total: 100,
        }),
      )

      // Change values
      act(() => {
        result.current.goToPage(5)
        result.current.setPageSize(50)
        result.current.setTotal(500)
      })

      // Reset
      act(() => {
        result.current.reset()
      })

      expect(result.current.currentPage).toBe(2)
      expect(result.current.pageSize).toBe(30)
      expect(result.current.total).toBe(100)
    })

    it('should reset to default values when no initial options', () => {
      const { result } = renderHook(() => usePagination())

      // Change values
      act(() => {
        result.current.setTotal(100)
        result.current.goToPage(3)
        result.current.setPageSize(50)
      })

      // Reset
      act(() => {
        result.current.reset()
      })

      expect(result.current.currentPage).toBe(1)
      expect(result.current.pageSize).toBe(20)
      expect(result.current.total).toBe(0)
    })
  })
})
