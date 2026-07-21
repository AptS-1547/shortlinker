import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { DateRange } from '@/components/analytics'
import { analyticsService } from '@/services/analyticsService'
import type { GroupBy } from '@/services/types'
import { useAnalyticsData } from '../useAnalyticsData'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('@/components/analytics', () => ({
  getDateRange: (range: string) => ({
    start: `${range}-start`,
    end: `${range}-end`,
  }),
}))

vi.mock('@/services/analyticsService', () => ({
  analyticsService: {
    getTrends: vi.fn(),
    getTopLinks: vi.fn(),
    getReferrers: vi.fn(),
    getGeoStats: vi.fn(),
    getDeviceStats: vi.fn(),
    exportReport: vi.fn(),
  },
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

describe('useAnalyticsData', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(analyticsService.getTrends).mockResolvedValue({
      labels: ['2026-07-21'],
      values: [4],
    })
    vi.mocked(analyticsService.getTopLinks).mockResolvedValue([
      { code: 'docs', clicks: 4 },
    ])
    vi.mocked(analyticsService.getReferrers).mockResolvedValue([])
    vi.mocked(analyticsService.getGeoStats).mockResolvedValue([])
    vi.mocked(analyticsService.getDeviceStats).mockResolvedValue(deviceData)
  })

  it('loads all analytics with one shared query', async () => {
    const { result } = renderHook(() => useAnalyticsData('30d', 'day'))

    await waitFor(() => expect(result.current.loading).toBe(false))

    const expectedQuery = {
      start_date: '30d-start',
      end_date: '30d-end',
      group_by: 'day',
      limit: 10,
    }
    expect(analyticsService.getTrends).toHaveBeenCalledWith(expectedQuery)
    expect(analyticsService.getTopLinks).toHaveBeenCalledWith(expectedQuery)
    expect(analyticsService.getReferrers).toHaveBeenCalledWith(expectedQuery)
    expect(analyticsService.getGeoStats).toHaveBeenCalledWith(expectedQuery)
    expect(analyticsService.getDeviceStats).toHaveBeenCalledWith(expectedQuery)
    expect(result.current.topLinks).toEqual([{ code: 'docs', clicks: 4 }])
    expect(result.current.error).toBeNull()
  })

  it('reloads when the date range or grouping changes', async () => {
    const { result, rerender } = renderHook(
      ({ range, groupBy }: { range: DateRange; groupBy: GroupBy }) =>
        useAnalyticsData(range, groupBy),
      { initialProps: { range: '7d', groupBy: 'day' } },
    )
    await waitFor(() => expect(result.current.loading).toBe(false))

    rerender({ range: '30d', groupBy: 'week' })

    await waitFor(() =>
      expect(analyticsService.getTrends).toHaveBeenLastCalledWith({
        start_date: '30d-start',
        end_date: '30d-end',
        group_by: 'week',
        limit: 10,
      }),
    )
  })

  it('surfaces a shared fetch failure', async () => {
    vi.mocked(analyticsService.getTrends).mockRejectedValue(new Error('failed'))
    const { result } = renderHook(() => useAnalyticsData('90d', 'month'))

    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.error).toBe('analytics.error.fetchFailed')
  })

  it('delegates report generation to the analytics service', async () => {
    const blob = new Blob(['report'])
    vi.mocked(analyticsService.exportReport).mockResolvedValue(blob)
    const createObjectURL = vi
      .spyOn(URL, 'createObjectURL')
      .mockReturnValue('blob:report')
    const revokeObjectURL = vi
      .spyOn(URL, 'revokeObjectURL')
      .mockImplementation(() => undefined)
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, 'click')
      .mockImplementation(() => undefined)
    const { result } = renderHook(() => useAnalyticsData('7d', 'day'))
    await waitFor(() => expect(result.current.loading).toBe(false))

    await act(async () => result.current.exportReport())

    expect(analyticsService.exportReport).toHaveBeenCalledWith({
      start_date: '7d-start',
      end_date: '7d-end',
      group_by: null,
      limit: null,
    })
    expect(click).toHaveBeenCalled()
    expect(createObjectURL).toHaveBeenCalledWith(blob)
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:report')
  })
})
