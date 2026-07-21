import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { healthService } from '@/services/healthService'
import { useLoginFlow } from '../useLoginFlow'

const navigate = vi.fn()
const login = vi.fn()

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('react-router-dom', () => ({
  useNavigate: () => navigate,
}))

vi.mock('@/services/healthService', () => ({
  healthService: { check: vi.fn() },
}))

vi.mock('@/stores/authStore', () => ({
  useAuthStore: (selector: (state: { login: typeof login }) => unknown) =>
    selector({ login }),
}))

vi.mock('@/utils/logger', () => ({
  authLogger: { error: vi.fn() },
}))

describe('useLoginFlow', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('requires a password before calling the auth store', async () => {
    const { result } = renderHook(() => useLoginFlow())

    await act(async () => result.current.authenticate(''))

    expect(result.current.error).toBe('auth.errors.passwordRequired')
    expect(login).not.toHaveBeenCalled()
    expect(healthService.check).not.toHaveBeenCalled()
  })

  it('logs in, verifies health, and navigates to the dashboard', async () => {
    login.mockResolvedValue(undefined)
    vi.mocked(healthService.check).mockResolvedValue({
      status: 'healthy',
      timestamp: '2026-07-21T00:00:00Z',
      uptime: 1,
      response_time_ms: 1,
      checks: {
        storage: {
          status: 'healthy',
          links_count: 0,
          backend: { storage_type: 'sqlite', support_click: true },
        },
        cache: null,
      },
    })
    const { result } = renderHook(() => useLoginFlow())

    await act(async () => result.current.authenticate('secret'))

    expect(login).toHaveBeenCalledWith('secret')
    expect(healthService.check).toHaveBeenCalledOnce()
    expect(navigate).toHaveBeenCalledWith('/dashboard')
    expect(result.current.isSubmitting).toBe(false)
  })

  it('maps authentication failures and stays on the login page', async () => {
    login.mockRejectedValue(new Error('401 INVALID_CREDENTIALS'))
    const { result } = renderHook(() => useLoginFlow())

    await act(async () => result.current.authenticate('wrong'))

    expect(result.current.error).toBe('auth.errors.unauthorized')
    expect(healthService.check).not.toHaveBeenCalled()
    expect(navigate).not.toHaveBeenCalled()
    expect(result.current.isSubmitting).toBe(false)
  })
})
