import { renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { analyticsService } from '@/services/analyticsService'
import { linkService } from '@/services/linkService'
import { useDashboardData } from '../useDashboardData'

vi.mock('@/services/analyticsService', () => ({
  analyticsService: { getTrends: vi.fn() },
}))

vi.mock('@/services/linkService', () => ({
  linkService: {
    fetchStats: vi.fn(),
    fetchPaginated: vi.fn(),
  },
}))

vi.mock('@/utils/logger', () => ({
  dashboardLogger: { error: vi.fn() },
}))

describe('useDashboardData', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(linkService.fetchStats).mockResolvedValue({
      total_links: 10,
      total_clicks: 25,
      active_links: 8,
    })
    vi.mocked(linkService.fetchPaginated).mockResolvedValue({
      code: 0,
      message: '',
      data: [
        {
          code: 'docs',
          target: 'https://example.com/docs',
          created_at: '2026-07-20T00:00:00Z',
          expires_at: null,
          password: null,
          click_count: 3,
        },
      ],
      pagination: { page: 1, page_size: 5, total: 1, total_pages: 1 },
    })
  })

  it('loads the overview and computes the latest click change', async () => {
    vi.mocked(analyticsService.getTrends).mockResolvedValue({
      labels: ['yesterday', 'today'],
      values: [10, 15],
    })
    const { result } = renderHook(() => useDashboardData())

    await waitFor(() => expect(result.current.trendLoading).toBe(false))
    await waitFor(() => expect(result.current.recentLinks).toHaveLength(1))

    expect(linkService.fetchPaginated).toHaveBeenCalledWith({
      page: 1,
      page_size: 5,
    })
    expect(result.current.stats.total_links).toBe(10)
    expect(result.current.trendData).toEqual([
      { date: 'yesterday', clicks: 10 },
      { date: 'today', clicks: 15 },
    ])
    expect(result.current.clickChange).toBe(50)
  })

  it('uses a 100 percent increase when the previous bucket is zero', async () => {
    vi.mocked(analyticsService.getTrends).mockResolvedValue({
      labels: ['yesterday', 'today'],
      values: [0, 2],
    })
    const { result } = renderHook(() => useDashboardData())

    await waitFor(() => expect(result.current.trendLoading).toBe(false))
    expect(result.current.clickChange).toBe(100)
  })

  it('finishes trend loading after a request error', async () => {
    vi.mocked(analyticsService.getTrends).mockRejectedValue(new Error('failed'))
    const { result } = renderHook(() => useDashboardData())

    await waitFor(() => expect(result.current.trendLoading).toBe(false))
    expect(result.current.trendData).toEqual([])
    expect(result.current.clickChange).toBeNull()
  })
})
