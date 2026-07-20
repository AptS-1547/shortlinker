import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useConfirm } from '../useConfirm'

// Mock logger
vi.mock('@/utils/logger', () => ({
  hookLogger: {
    error: vi.fn(),
  },
}))

describe('useConfirm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have correct initial values', () => {
      const { result } = renderHook(() => useConfirm())
      expect(result.current.isOpen).toBe(false)
      expect(result.current.loading).toBe(false)
      expect(result.current.currentOptions).toBe(null)
    })
  })

  // ==========================================================================
  // confirm
  // ==========================================================================

  describe('confirm', () => {
    it('should open dialog with options', () => {
      const { result } = renderHook(() => useConfirm())

      act(() => {
        result.current.confirm({
          title: 'Delete Item',
          message: 'Are you sure?',
        })
      })

      expect(result.current.isOpen).toBe(true)
      expect(result.current.currentOptions).toEqual({
        title: 'Delete Item',
        message: 'Are you sure?',
      })
    })

    it('should accept all options', () => {
      const { result } = renderHook(() => useConfirm())

      act(() => {
        result.current.confirm({
          title: 'Delete Item',
          message: 'Are you sure you want to delete this?',
          confirmText: 'Yes, delete',
          cancelText: 'No, keep',
          variant: 'danger',
        })
      })

      expect(result.current.currentOptions).toEqual({
        title: 'Delete Item',
        message: 'Are you sure you want to delete this?',
        confirmText: 'Yes, delete',
        cancelText: 'No, keep',
        variant: 'danger',
      })
    })
  })

  // ==========================================================================
  // handleConfirm
  // ==========================================================================

  describe('handleConfirm', () => {
    it('should resolve true and close dialog without callback', async () => {
      const { result } = renderHook(() => useConfirm())
      let confirmPromise: Promise<boolean>

      act(() => {
        confirmPromise = result.current.confirm({
          title: 'Test',
          message: 'Test message',
        })
      })

      expect(result.current.isOpen).toBe(true)

      await act(async () => {
        await result.current.handleConfirm()
      })

      expect(result.current.isOpen).toBe(false)
      // biome-ignore lint/style/noNonNullAssertion: test context guarantees assignment
      expect(await confirmPromise!).toBe(true)
    })

    it('should execute callback and resolve true on success', async () => {
      const { result } = renderHook(() => useConfirm())
      const onConfirm = vi.fn()
      let confirmPromise: Promise<boolean>

      act(() => {
        confirmPromise = result.current.confirm({
          title: 'Test',
          message: 'Test message',
        })
      })

      await act(async () => {
        await result.current.handleConfirm(onConfirm)
      })

      expect(onConfirm).toHaveBeenCalled()
      expect(result.current.isOpen).toBe(false)
      // biome-ignore lint/style/noNonNullAssertion: test context guarantees assignment
      expect(await confirmPromise!).toBe(true)
    })

    it('should set loading to true and then false during async callback', async () => {
      const { result } = renderHook(() => useConfirm())

      act(() => {
        result.current.confirm({
          title: 'Test',
          message: 'Test message',
        })
      })

      // Before handleConfirm, loading should be false
      expect(result.current.loading).toBe(false)

      const asyncCallback = vi.fn().mockImplementation(async () => {
        await new Promise((resolve) => setTimeout(resolve, 10))
      })

      await act(async () => {
        await result.current.handleConfirm(asyncCallback)
      })

      // After completion, loading should be false
      expect(result.current.loading).toBe(false)
      expect(asyncCallback).toHaveBeenCalled()
    })

    it('should resolve false and close on callback error', async () => {
      const { result } = renderHook(() => useConfirm())
      const failingCallback = vi.fn().mockRejectedValue(new Error('Failed'))
      let confirmPromise: Promise<boolean>

      act(() => {
        confirmPromise = result.current.confirm({
          title: 'Test',
          message: 'Test message',
        })
      })

      await act(async () => {
        await result.current.handleConfirm(failingCallback)
      })

      // biome-ignore lint/style/noNonNullAssertion: test context guarantees assignment
      expect(await confirmPromise!).toBe(false)
      expect(result.current.isOpen).toBe(false)
    })
  })

  // ==========================================================================
  // handleCancel
  // ==========================================================================

  describe('handleCancel', () => {
    it('should resolve false and close dialog', async () => {
      const { result } = renderHook(() => useConfirm())
      let confirmPromise: Promise<boolean>

      act(() => {
        confirmPromise = result.current.confirm({
          title: 'Test',
          message: 'Test message',
        })
      })

      expect(result.current.isOpen).toBe(true)

      act(() => {
        result.current.handleCancel()
      })

      expect(result.current.isOpen).toBe(false)
      // biome-ignore lint/style/noNonNullAssertion: test context guarantees assignment
      expect(await confirmPromise!).toBe(false)
    })
  })

  // ==========================================================================
  // Integration tests
  // ==========================================================================

  describe('integration', () => {
    it('should handle multiple confirm cycles', async () => {
      const { result } = renderHook(() => useConfirm())

      // First confirm
      let confirmPromise1: Promise<boolean>
      act(() => {
        confirmPromise1 = result.current.confirm({
          title: 'First',
          message: 'First message',
        })
      })

      await act(async () => {
        await result.current.handleConfirm()
      })

      // biome-ignore lint/style/noNonNullAssertion: test context guarantees assignment
      expect(await confirmPromise1!).toBe(true)

      // Second confirm (cancel)
      let confirmPromise2: Promise<boolean>
      act(() => {
        confirmPromise2 = result.current.confirm({
          title: 'Second',
          message: 'Second message',
        })
      })

      act(() => {
        result.current.handleCancel()
      })

      // biome-ignore lint/style/noNonNullAssertion: test context guarantees assignment
      expect(await confirmPromise2!).toBe(false)
    })

    it('should update options correctly between confirms', () => {
      const { result } = renderHook(() => useConfirm())

      act(() => {
        result.current.confirm({
          title: 'First Title',
          message: 'First message',
          variant: 'primary',
        })
      })

      expect(result.current.currentOptions?.title).toBe('First Title')
      expect(result.current.currentOptions?.variant).toBe('primary')

      act(() => {
        result.current.handleCancel()
      })

      act(() => {
        result.current.confirm({
          title: 'Second Title',
          message: 'Second message',
          variant: 'danger',
        })
      })

      expect(result.current.currentOptions?.title).toBe('Second Title')
      expect(result.current.currentOptions?.variant).toBe('danger')
    })
  })
})
