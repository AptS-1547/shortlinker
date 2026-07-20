import { act, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useDialog } from '../useDialog'

describe('useDialog', () => {
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
    it('should be closed by default', () => {
      const { result } = renderHook(() => useDialog())
      expect(result.current.isOpen).toBe(false)
      expect(result.current.data).toBe(null)
      expect(result.current.isEditMode).toBe(false)
    })

    it('should be open when defaultOpen is true', () => {
      const { result } = renderHook(() => useDialog(true))
      expect(result.current.isOpen).toBe(true)
    })
  })

  // ==========================================================================
  // open
  // ==========================================================================

  describe('open', () => {
    it('should open dialog without data', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.open()
      })

      expect(result.current.isOpen).toBe(true)
      expect(result.current.data).toBe(null)
      expect(result.current.isEditMode).toBe(false)
    })

    it('should open dialog with data', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())
      const testData = { id: 123 }

      act(() => {
        result.current.open(testData)
      })

      expect(result.current.isOpen).toBe(true)
      expect(result.current.data).toEqual(testData)
      expect(result.current.isEditMode).toBe(true)
    })

    it('should replace existing data when opening', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.open({ id: 1 })
      })

      act(() => {
        result.current.open({ id: 2 })
      })

      expect(result.current.data).toEqual({ id: 2 })
    })

    it('should clear data when opening without data after having data', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.open({ id: 1 })
      })

      act(() => {
        result.current.open()
      })

      expect(result.current.data).toBe(null)
      expect(result.current.isEditMode).toBe(false)
    })
  })

  // ==========================================================================
  // close
  // ==========================================================================

  describe('close', () => {
    it('should close dialog', () => {
      const { result } = renderHook(() => useDialog())

      act(() => {
        result.current.open()
      })

      act(() => {
        result.current.close()
      })

      expect(result.current.isOpen).toBe(false)
    })

    it('should clear data after delay', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.open({ id: 123 })
      })

      act(() => {
        result.current.close()
      })

      // Data should still exist immediately after close
      expect(result.current.data).toEqual({ id: 123 })

      // Advance timers by 150ms
      act(() => {
        vi.advanceTimersByTime(150)
      })

      // Data should be cleared after delay
      expect(result.current.data).toBe(null)
    })
  })

  // ==========================================================================
  // toggle
  // ==========================================================================

  describe('toggle', () => {
    it('should toggle from closed to open', () => {
      const { result } = renderHook(() => useDialog())

      act(() => {
        result.current.toggle()
      })

      expect(result.current.isOpen).toBe(true)
    })

    it('should toggle from open to closed', () => {
      const { result } = renderHook(() => useDialog(true))

      act(() => {
        result.current.toggle()
      })

      expect(result.current.isOpen).toBe(false)
    })

    it('should toggle multiple times', () => {
      const { result } = renderHook(() => useDialog())

      act(() => {
        result.current.toggle()
      })
      expect(result.current.isOpen).toBe(true)

      act(() => {
        result.current.toggle()
      })
      expect(result.current.isOpen).toBe(false)

      act(() => {
        result.current.toggle()
      })
      expect(result.current.isOpen).toBe(true)
    })
  })

  // ==========================================================================
  // setIsOpen
  // ==========================================================================

  describe('setIsOpen', () => {
    it('should set isOpen directly', () => {
      const { result } = renderHook(() => useDialog())

      act(() => {
        result.current.setIsOpen(true)
      })

      expect(result.current.isOpen).toBe(true)

      act(() => {
        result.current.setIsOpen(false)
      })

      expect(result.current.isOpen).toBe(false)
    })
  })

  // ==========================================================================
  // setData
  // ==========================================================================

  describe('setData', () => {
    it('should set data directly', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.setData({ id: 456 })
      })

      expect(result.current.data).toEqual({ id: 456 })
      expect(result.current.isEditMode).toBe(true)
    })

    it('should clear data when set to null', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.setData({ id: 123 })
      })

      act(() => {
        result.current.setData(null)
      })

      expect(result.current.data).toBe(null)
      expect(result.current.isEditMode).toBe(false)
    })
  })

  // ==========================================================================
  // isEditMode
  // ==========================================================================

  describe('isEditMode', () => {
    it('should be false when data is null', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())
      expect(result.current.isEditMode).toBe(false)
    })

    it('should be true when data is set', () => {
      const { result } = renderHook(() => useDialog<{ id: number }>())

      act(() => {
        result.current.open({ id: 1 })
      })

      expect(result.current.isEditMode).toBe(true)
    })

    it('should handle various data types', () => {
      // Test with string
      const { result: stringResult } = renderHook(() => useDialog<string>())
      act(() => {
        stringResult.current.open('test')
      })
      expect(stringResult.current.isEditMode).toBe(true)

      // Test with number
      const { result: numberResult } = renderHook(() => useDialog<number>())
      act(() => {
        numberResult.current.open(0)
      })
      expect(numberResult.current.isEditMode).toBe(true)

      // Test with empty string
      const { result: emptyStringResult } = renderHook(() =>
        useDialog<string>(),
      )
      act(() => {
        emptyStringResult.current.open('')
      })
      expect(emptyStringResult.current.isEditMode).toBe(true)
    })
  })

  // ==========================================================================
  // Type safety
  // ==========================================================================

  describe('type safety', () => {
    interface LinkData {
      id: string
      url: string
      code: string
    }

    it('should work with complex types', () => {
      const { result } = renderHook(() => useDialog<LinkData>())
      const linkData: LinkData = {
        id: '1',
        url: 'https://example.com',
        code: 'test',
      }

      act(() => {
        result.current.open(linkData)
      })

      expect(result.current.data).toEqual(linkData)
      expect(result.current.data?.id).toBe('1')
      expect(result.current.data?.url).toBe('https://example.com')
    })
  })
})
