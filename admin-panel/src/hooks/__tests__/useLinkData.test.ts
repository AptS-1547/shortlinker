import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { DateRange } from '@/components/analytics'
import { analyticsService } from '@/services/analyticsService'
import { linkService } from '@/services/linkService'
import type { GroupBy, LinkAnalytics } from '@/services/types'
import { useLinkAnalyticsData } from '../useLinkAnalyticsData'
import { useLinkDetailData } from '../useLinkDetailData'

const translate = vi.hoisted(() => (key: string) => key)

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: translate }),
}))

vi.mock('@/components/analytics', () => ({
  getDateRange: (range: string) => ({
    start: `${range}-start`,
    end: `${range}-end`,
  }),
}))

vi.mock('@/services/analyticsService', () => ({
  analyticsService: {
    getLinkAnalytics: vi.fn(),
    getLinkDeviceStats: vi.fn(),
  },
}))

vi.mock('@/services/linkService', () => ({
  linkService: { fetchOne: vi.fn() },
}))

vi.mock('@/utils/logger', () => ({
  logger: { error: vi.fn() },
}))

const deviceData = {
  browsers: [],
  devices: [],
  operating_systems: [],
  total_with_ua: 0,
  bot_percentage: 0,
}

function analyticsFor(code: string, clicks = 1): LinkAnalytics {
  return {
    code,
    total_clicks: clicks,
    trend: { labels: [], values: [] },
    top_referrers: [],
    geo_distribution: [],
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

describe('link data controllers', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(linkService.fetchOne).mockImplementation(async (code) => ({
      code,
      target: `https://example.com/${code}`,
      created_at: '2026-07-21T00:00:00Z',
      expires_at: null,
      password: null,
      click_count: 1,
    }))
    vi.mocked(analyticsService.getLinkAnalytics).mockImplementation(
      async (code) => analyticsFor(code),
    )
    vi.mocked(analyticsService.getLinkDeviceStats).mockResolvedValue(deviceData)
  })

  it('loads link metadata and analytics with the selected range', async () => {
    const { result } = renderHook(() =>
      useLinkDetailData('docs', '30d', 'week'),
    )

    await waitFor(() => expect(result.current.loading).toBe(false))

    expect(linkService.fetchOne).toHaveBeenCalledWith('docs')
    expect(analyticsService.getLinkAnalytics).toHaveBeenCalledWith('docs', {
      start_date: '30d-start',
      end_date: '30d-end',
      group_by: 'week',
      limit: 10,
    })
    expect(result.current.linkInfo?.code).toBe('docs')
    expect(result.current.analytics?.code).toBe('docs')
    expect(result.current.deviceData).toEqual(deviceData)
  })

  it('reports a missing link without leaving analytics behind', async () => {
    vi.mocked(linkService.fetchOne).mockResolvedValue(null)
    const { result } = renderHook(() =>
      useLinkDetailData('missing', '7d', 'day'),
    )

    await waitFor(() => expect(result.current.loading).toBe(false))

    expect(result.current.error).toBe('linkDetail.notFound')
    expect(result.current.linkInfo).toBeNull()
    expect(result.current.analytics).toBeNull()
    expect(result.current.deviceData).toBeNull()
  })

  it('discards a stale link-detail response after the code changes', async () => {
    const oldAnalytics = deferred<LinkAnalytics>()
    vi.mocked(analyticsService.getLinkAnalytics).mockImplementation((code) =>
      code === 'old'
        ? oldAnalytics.promise
        : Promise.resolve(analyticsFor(code, 2)),
    )
    const { result, rerender } = renderHook(
      ({
        code,
        range,
        groupBy,
      }: {
        code: string
        range: DateRange
        groupBy: GroupBy
      }) => useLinkDetailData(code, range, groupBy),
      { initialProps: { code: 'old', range: '7d', groupBy: 'day' } },
    )

    rerender({ code: 'new', range: '30d', groupBy: 'week' })
    await waitFor(() => expect(result.current.analytics?.code).toBe('new'))

    await act(async () => oldAnalytics.resolve(analyticsFor('old', 99)))
    expect(result.current.analytics?.code).toBe('new')
    expect(result.current.analytics?.total_clicks).toBe(2)
  })

  it('loads compact analytics and clears them when the sheet closes', async () => {
    const { result, rerender } = renderHook(
      ({ code }: { code: string | null }) => useLinkAnalyticsData(code),
      { initialProps: { code: 'docs' as string | null } },
    )
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.data?.code).toBe('docs')

    rerender({ code: null })

    await waitFor(() => expect(result.current.data).toBeNull())
    expect(result.current.deviceData).toBeNull()
    expect(result.current.loading).toBe(false)
  })
})
