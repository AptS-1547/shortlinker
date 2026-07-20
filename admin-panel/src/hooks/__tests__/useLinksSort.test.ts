import { act, renderHook } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { LinkResponse } from '@/services/types'
import { useLinksSort } from '../useLinksSort'

// Helper to create mock links
function createMockLinks(): LinkResponse[] {
  return [
    {
      code: 'link1',
      target: 'https://example1.com',
      click_count: 100,
      created_at: '2024-01-15T10:00:00Z',
      expires_at: null,
      password: null,
    },
    {
      code: 'link2',
      target: 'https://example2.com',
      click_count: 50,
      created_at: '2024-01-10T10:00:00Z',
      expires_at: null,
      password: null,
    },
    {
      code: 'link3',
      target: 'https://example3.com',
      click_count: 200,
      created_at: '2024-01-20T10:00:00Z',
      expires_at: null,
      password: null,
    },
  ]
}

describe('useLinksSort', () => {
  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have no sort field by default', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))
      expect(result.current.sortField).toBeNull()
    })

    it('should have desc direction by default', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))
      expect(result.current.sortDirection).toBe('desc')
    })

    it('should return unsorted links when no sort field', () => {
      const links = createMockLinks()
      const { result } = renderHook(() => useLinksSort(links))
      expect(result.current.sortedLinks).toEqual(links)
    })
  })

  // ==========================================================================
  // Sorting by clicks
  // ==========================================================================

  describe('sorting by clicks', () => {
    it('should sort by clicks descending', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))

      act(() => {
        result.current.handleSort('clicks')
      })

      expect(result.current.sortField).toBe('clicks')
      expect(result.current.sortDirection).toBe('desc')
      expect(result.current.sortedLinks[0].click_count).toBe(200)
      expect(result.current.sortedLinks[1].click_count).toBe(100)
      expect(result.current.sortedLinks[2].click_count).toBe(50)
    })

    it('should toggle to ascending on second click', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))

      act(() => {
        result.current.handleSort('clicks')
      })
      act(() => {
        result.current.handleSort('clicks')
      })

      expect(result.current.sortDirection).toBe('asc')
      expect(result.current.sortedLinks[0].click_count).toBe(50)
      expect(result.current.sortedLinks[1].click_count).toBe(100)
      expect(result.current.sortedLinks[2].click_count).toBe(200)
    })

    it('should toggle back to descending on third click', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))

      act(() => {
        result.current.handleSort('clicks')
      })
      act(() => {
        result.current.handleSort('clicks')
      })
      act(() => {
        result.current.handleSort('clicks')
      })

      expect(result.current.sortDirection).toBe('desc')
    })

    it('should handle null click_count', () => {
      const linksWithNull: LinkResponse[] = [
        {
          code: 'link1',
          target: 'https://example.com',
          click_count: null as unknown as number,
          created_at: '2024-01-15T10:00:00Z',
          expires_at: null,
          password: null,
        },
        {
          code: 'link2',
          target: 'https://example.com',
          click_count: 10,
          created_at: '2024-01-15T10:00:00Z',
          expires_at: null,
          password: null,
        },
      ]
      const { result } = renderHook(() => useLinksSort(linksWithNull))

      act(() => {
        result.current.handleSort('clicks')
      })

      // Should not throw, null treated as 0
      expect(result.current.sortedLinks.length).toBe(2)
    })
  })

  // ==========================================================================
  // Sorting by created_at
  // ==========================================================================

  describe('sorting by created_at', () => {
    it('should sort by created_at descending', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))

      act(() => {
        result.current.handleSort('created_at')
      })

      expect(result.current.sortField).toBe('created_at')
      expect(result.current.sortDirection).toBe('desc')
      // Most recent first (2024-01-20)
      expect(result.current.sortedLinks[0].code).toBe('link3')
      expect(result.current.sortedLinks[1].code).toBe('link1')
      expect(result.current.sortedLinks[2].code).toBe('link2')
    })

    it('should sort by created_at ascending', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))

      act(() => {
        result.current.handleSort('created_at')
      })
      act(() => {
        result.current.handleSort('created_at')
      })

      expect(result.current.sortDirection).toBe('asc')
      // Oldest first (2024-01-10)
      expect(result.current.sortedLinks[0].code).toBe('link2')
      expect(result.current.sortedLinks[1].code).toBe('link1')
      expect(result.current.sortedLinks[2].code).toBe('link3')
    })
  })

  // ==========================================================================
  // Switching sort fields
  // ==========================================================================

  describe('switching sort fields', () => {
    it('should reset to desc when switching fields', () => {
      const { result } = renderHook(() => useLinksSort(createMockLinks()))

      // Sort by clicks asc
      act(() => {
        result.current.handleSort('clicks')
      })
      act(() => {
        result.current.handleSort('clicks')
      })
      expect(result.current.sortDirection).toBe('asc')

      // Switch to created_at
      act(() => {
        result.current.handleSort('created_at')
      })
      expect(result.current.sortField).toBe('created_at')
      expect(result.current.sortDirection).toBe('desc')
    })
  })

  // ==========================================================================
  // Handling null sort field
  // ==========================================================================

  describe('handling null sort field', () => {
    it('should handle null field in handleSort', () => {
      const links = createMockLinks()
      const { result } = renderHook(() => useLinksSort(links))

      act(() => {
        result.current.handleSort(null)
      })

      // Should still return original order
      expect(result.current.sortedLinks).toEqual(links)
    })
  })

  // ==========================================================================
  // Empty links array
  // ==========================================================================

  describe('empty links array', () => {
    it('should handle empty array', () => {
      const { result } = renderHook(() => useLinksSort([]))

      act(() => {
        result.current.handleSort('clicks')
      })

      expect(result.current.sortedLinks).toEqual([])
    })
  })

  // ==========================================================================
  // Links update
  // ==========================================================================

  describe('links update', () => {
    it('should re-sort when links change', () => {
      const initialLinks = createMockLinks()
      const { result, rerender } = renderHook(
        ({ links }) => useLinksSort(links),
        { initialProps: { links: initialLinks } },
      )

      // Apply sort
      act(() => {
        result.current.handleSort('clicks')
      })

      // New links with different click counts
      const newLinks: LinkResponse[] = [
        { ...initialLinks[0], click_count: 5 },
        { ...initialLinks[1], click_count: 500 },
      ]

      rerender({ links: newLinks })

      expect(result.current.sortedLinks[0].click_count).toBe(500)
      expect(result.current.sortedLinks[1].click_count).toBe(5)
    })
  })
})
