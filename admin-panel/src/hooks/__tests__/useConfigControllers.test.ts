import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { systemConfigService } from '@/services/systemConfigService'
import { useConfigAction } from '../useConfigAction'
import { useConfigHistory } from '../useConfigHistory'

vi.mock('@/services/systemConfigService', () => ({
  systemConfigService: {
    fetchHistory: vi.fn(),
    executeAndSave: vi.fn(),
  },
}))

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

describe('config controllers', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('loads history only while the dialog is enabled', async () => {
    vi.mocked(systemConfigService.fetchHistory).mockResolvedValue([
      {
        id: 1,
        config_key: 'server.port',
        old_value: '8080',
        new_value: '8081',
        changed_at: '2026-07-21T00:00:00Z',
        changed_by: null,
      },
    ])
    const { result, rerender } = renderHook(
      ({ enabled }) => useConfigHistory('server.port', enabled),
      { initialProps: { enabled: false } },
    )
    expect(systemConfigService.fetchHistory).not.toHaveBeenCalled()

    rerender({ enabled: true })
    await waitFor(() => expect(result.current.loading).toBe(false))

    expect(systemConfigService.fetchHistory).toHaveBeenCalledWith(
      'server.port',
      50,
    )
    expect(result.current.history).toHaveLength(1)

    rerender({ enabled: false })
    await waitFor(() => expect(result.current.history).toEqual([]))
  })

  it('exposes history request errors', async () => {
    vi.mocked(systemConfigService.fetchHistory).mockRejectedValue(
      new Error('history failed'),
    )
    const { result } = renderHook(() => useConfigHistory('server.port', true))

    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.error).toBe('history failed')
  })

  it('tracks execution until the config action completes', async () => {
    const pending = deferred<{
      success: boolean
      requires_restart: boolean
      message: string | null
    }>()
    vi.mocked(systemConfigService.executeAndSave).mockReturnValue(
      pending.promise,
    )
    const { result } = renderHook(() => useConfigAction())
    let actionPromise!: ReturnType<typeof result.current.executeAndSave>

    act(() => {
      actionPromise = result.current.executeAndSave(
        'jwt.secret',
        'generate_token',
      )
    })
    expect(result.current.executing).toBe(true)

    await act(async () => {
      pending.resolve({
        success: true,
        requires_restart: true,
        message: null,
      })
      await actionPromise
    })

    expect(systemConfigService.executeAndSave).toHaveBeenCalledWith(
      'jwt.secret',
      'generate_token',
    )
    expect(result.current.executing).toBe(false)
  })
})
