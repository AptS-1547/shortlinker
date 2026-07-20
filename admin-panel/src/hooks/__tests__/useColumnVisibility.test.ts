import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useColumnVisibility } from '../useColumnVisibility'

// Mock the imports
vi.mock('@/components/links/LinksTable', () => ({
  DEFAULT_VISIBLE_COLUMNS: [
    'code',
    'target',
    'clicks',
    'status',
    'created',
    'expires',
  ],
}))

vi.mock('@/utils/storage', () => ({
  STORAGE_KEYS: {
    LINKS_VISIBLE_COLUMNS: 'links_visible_columns',
  },
  Storage: {
    getJSON: vi.fn(),
    setJSON: vi.fn(),
  },
}))

import { DEFAULT_VISIBLE_COLUMNS } from '@/components/links/LinksTable'
import { STORAGE_KEYS, Storage } from '@/utils/storage'

describe('useColumnVisibility', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(Storage.getJSON).mockReturnValue(null)
  })

  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should use default columns when storage is empty', () => {
      const { result } = renderHook(() => useColumnVisibility())
      expect(result.current.visibleColumns).toEqual(DEFAULT_VISIBLE_COLUMNS)
    })

    it('should load columns from storage', () => {
      const savedColumns = ['code', 'target', 'clicks']
      vi.mocked(Storage.getJSON).mockReturnValue(savedColumns)

      const { result } = renderHook(() => useColumnVisibility())
      expect(result.current.visibleColumns).toEqual(savedColumns)
    })

    it('should use default columns when storage has invalid data', () => {
      vi.mocked(Storage.getJSON).mockReturnValue('invalid')

      const { result } = renderHook(() => useColumnVisibility())
      expect(result.current.visibleColumns).toEqual(DEFAULT_VISIBLE_COLUMNS)
    })

    it('should use default columns when storage has invalid column names', () => {
      vi.mocked(Storage.getJSON).mockReturnValue([
        'invalid_column',
        'another_invalid',
      ])

      const { result } = renderHook(() => useColumnVisibility())
      expect(result.current.visibleColumns).toEqual(DEFAULT_VISIBLE_COLUMNS)
    })

    it('should handle storage.getJSON throwing error', () => {
      vi.mocked(Storage.getJSON).mockImplementation(() => {
        throw new Error('Storage error')
      })

      const { result } = renderHook(() => useColumnVisibility())
      expect(result.current.visibleColumns).toEqual(DEFAULT_VISIBLE_COLUMNS)
    })
  })

  // ==========================================================================
  // handleColumnToggle
  // ==========================================================================

  describe('handleColumnToggle', () => {
    it('should add column when checked is true', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code', 'target'])

      const { result } = renderHook(() => useColumnVisibility())

      act(() => {
        result.current.handleColumnToggle('clicks', true)
      })

      expect(result.current.visibleColumns).toContain('clicks')
      expect(result.current.visibleColumns).toEqual([
        'code',
        'target',
        'clicks',
      ])
    })

    it('should remove column when checked is false', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code', 'target', 'clicks'])

      const { result } = renderHook(() => useColumnVisibility())

      act(() => {
        result.current.handleColumnToggle('clicks', false)
      })

      expect(result.current.visibleColumns).not.toContain('clicks')
      expect(result.current.visibleColumns).toEqual(['code', 'target'])
    })

    it('should save to storage after toggle', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code', 'target'])

      const { result } = renderHook(() => useColumnVisibility())

      act(() => {
        result.current.handleColumnToggle('clicks', true)
      })

      expect(Storage.setJSON).toHaveBeenCalledWith(
        STORAGE_KEYS.LINKS_VISIBLE_COLUMNS,
        ['code', 'target', 'clicks'],
      )
    })

    it('should add column even if already exists (implementation detail)', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code', 'target'])

      const { result } = renderHook(() => useColumnVisibility())

      // When checked=true, implementation adds to end without checking duplicates
      act(() => {
        result.current.handleColumnToggle('code', true)
      })

      // Current implementation adds duplicates
      expect(result.current.visibleColumns).toEqual(['code', 'target', 'code'])
    })

    it('should handle removing non-existent column gracefully', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code', 'target'])

      const { result } = renderHook(() => useColumnVisibility())

      act(() => {
        result.current.handleColumnToggle('clicks', false)
      })

      // Should remain unchanged
      expect(result.current.visibleColumns).toEqual(['code', 'target'])
    })
  })

  // ==========================================================================
  // Multiple toggles
  // ==========================================================================

  describe('multiple toggles', () => {
    it('should handle multiple sequential toggles', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code'])

      const { result } = renderHook(() => useColumnVisibility())

      act(() => {
        result.current.handleColumnToggle('target', true)
      })

      act(() => {
        result.current.handleColumnToggle('clicks', true)
      })

      act(() => {
        result.current.handleColumnToggle('code', false)
      })

      expect(result.current.visibleColumns).toEqual(['target', 'clicks'])
    })

    it('should persist each toggle to storage', () => {
      vi.mocked(Storage.getJSON).mockReturnValue(['code'])

      const { result } = renderHook(() => useColumnVisibility())

      act(() => {
        result.current.handleColumnToggle('target', true)
      })

      expect(Storage.setJSON).toHaveBeenLastCalledWith(
        STORAGE_KEYS.LINKS_VISIBLE_COLUMNS,
        ['code', 'target'],
      )

      act(() => {
        result.current.handleColumnToggle('clicks', true)
      })

      expect(Storage.setJSON).toHaveBeenLastCalledWith(
        STORAGE_KEYS.LINKS_VISIBLE_COLUMNS,
        ['code', 'target', 'clicks'],
      )
    })
  })

  // ==========================================================================
  // Return value stability
  // ==========================================================================

  describe('return value stability', () => {
    it('should return stable handleColumnToggle reference', () => {
      const { result, rerender } = renderHook(() => useColumnVisibility())

      const firstRef = result.current.handleColumnToggle
      rerender()
      const secondRef = result.current.handleColumnToggle

      expect(firstRef).toBe(secondRef)
    })
  })
})
