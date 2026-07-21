import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { batchService } from '@/services/batchService'
import { useLinkImport } from '../useLinkImport'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('@/services/batchService', () => ({
  batchService: {
    importLinks: vi.fn(),
  },
}))

const importResult = {
  total_rows: 3,
  success_count: 2,
  skipped_count: 1,
  failed_count: 0,
  failed_items: [],
}

describe('useLinkImport', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('tracks upload progress and reports a successful import', async () => {
    const onSuccess = vi.fn()
    vi.mocked(batchService.importLinks).mockImplementation(
      async (_file, _mode, onProgress) => {
        onProgress?.(45)
        return importResult
      },
    )
    const { result } = renderHook(() => useLinkImport(onSuccess))
    const file = new File(['code,target'], 'links.csv', { type: 'text/csv' })

    await act(async () => {
      await result.current.importLinks(file, 'overwrite')
    })

    expect(batchService.importLinks).toHaveBeenCalledWith(
      file,
      'overwrite',
      expect.any(Function),
    )
    expect(result.current.uploadProgress).toBe(45)
    expect(result.current.result).toEqual(importResult)
    expect(result.current.state).toBe('success')
    expect(result.current.error).toBeNull()
    expect(onSuccess).toHaveBeenCalledOnce()
  })

  it('exposes request errors without reporting success', async () => {
    const onSuccess = vi.fn()
    vi.mocked(batchService.importLinks).mockRejectedValue(
      new Error('invalid csv'),
    )
    const { result } = renderHook(() => useLinkImport(onSuccess))

    await act(async () => {
      await result.current.importLinks(new File(['bad'], 'bad.csv'), 'skip')
    })

    expect(result.current.state).toBe('error')
    expect(result.current.error).toBe('invalid csv')
    expect(result.current.result).toBeNull()
    expect(onSuccess).not.toHaveBeenCalled()
  })

  it('supports local validation failures and resets all request state', () => {
    const { result } = renderHook(() => useLinkImport(vi.fn()))

    act(() => {
      result.current.fail('file too large')
    })
    expect(result.current.state).toBe('error')
    expect(result.current.error).toBe('file too large')

    act(() => {
      result.current.reset()
    })
    expect(result.current.state).toBe('idle')
    expect(result.current.error).toBeNull()
    expect(result.current.result).toBeNull()
    expect(result.current.uploadProgress).toBe(0)
  })
})
