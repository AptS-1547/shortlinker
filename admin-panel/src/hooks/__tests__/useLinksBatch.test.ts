import { act, renderHook } from '@testing-library/react'
import { toast } from 'sonner'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { batchService } from '@/services/batchService'
import { useLinksBatch } from '../useLinksBatch'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    warning: vi.fn(),
    error: vi.fn(),
  },
}))

vi.mock('@/services/batchService', () => ({
  batchService: {
    deleteLinks: vi.fn(),
    exportLinks: vi.fn(),
  },
}))

vi.mock('@/utils/logger', () => ({
  logger: { error: vi.fn() },
}))

describe('useLinksBatch', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('selects individual links and all visible links', () => {
    const { result } = renderHook(() => useLinksBatch(['a', 'b']))

    act(() => result.current.handleSelectChange('a', true))
    expect(result.current.selectedCodes).toEqual(new Set(['a']))

    act(() => result.current.handleSelectAll(true))
    expect(result.current.selectedCodes).toEqual(new Set(['a', 'b']))

    act(() => result.current.handleClearSelection())
    expect(result.current.selectedCount).toBe(0)
  })

  it('deletes the selected links and refreshes the list', async () => {
    const onBatchDeleteSuccess = vi.fn()
    vi.mocked(batchService.deleteLinks).mockResolvedValue({
      success: ['a', 'b'],
      failed: [],
    })
    const { result } = renderHook(() =>
      useLinksBatch(['a', 'b'], { onBatchDeleteSuccess }),
    )

    act(() => result.current.handleSelectAll(true))
    act(() => result.current.setBatchDeleteOpen(true))
    await act(async () => result.current.handleBatchDelete())

    expect(batchService.deleteLinks).toHaveBeenCalledWith(['a', 'b'])
    expect(toast.success).toHaveBeenCalled()
    expect(onBatchDeleteSuccess).toHaveBeenCalledOnce()
    expect(result.current.selectedCount).toBe(0)
    expect(result.current.batchDeleteOpen).toBe(false)
    expect(result.current.batchDeleting).toBe(false)
  })

  it('keeps the selection when batch deletion fails', async () => {
    vi.mocked(batchService.deleteLinks).mockRejectedValue(new Error('failed'))
    const { result } = renderHook(() => useLinksBatch(['a']))

    act(() => result.current.handleSelectChange('a', true))
    await act(async () => result.current.handleBatchDelete())

    expect(toast.error).toHaveBeenCalled()
    expect(result.current.selectedCodes).toEqual(new Set(['a']))
    expect(result.current.batchDeleting).toBe(false)
  })

  it('owns the export request lifecycle', async () => {
    vi.mocked(batchService.exportLinks).mockResolvedValue()
    const { result } = renderHook(() => useLinksBatch([]))
    const query = { search: 'docs', only_active: true }

    await act(async () => result.current.handleExport(query))

    expect(batchService.exportLinks).toHaveBeenCalledWith(query)
    expect(toast.success).toHaveBeenCalled()
    expect(result.current.exporting).toBe(false)
  })
})
