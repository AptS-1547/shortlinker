import { useEffect, useRef, useState } from 'react'
import { analyticsService } from '@/services/analyticsService'
import { linkService } from '@/services/linkService'
import type { LinkResponse, StatsResponse } from '@/services/types'
import { dashboardLogger } from '@/utils/logger'

const EMPTY_STATS: StatsResponse = {
  total_links: 0,
  total_clicks: 0,
  active_links: 0,
}

export function useDashboardData() {
  const [recentLinks, setRecentLinks] = useState<LinkResponse[]>([])
  const [stats, setStats] = useState<StatsResponse>(EMPTY_STATS)
  const [trendData, setTrendData] = useState<
    { date: string; clicks: number }[]
  >([])
  const [trendLoading, setTrendLoading] = useState(true)
  const [clickChange, setClickChange] = useState<number | null>(null)
  const fetchedOverview = useRef(false)
  const fetchedTrend = useRef(false)

  useEffect(() => {
    if (fetchedOverview.current) return
    fetchedOverview.current = true

    const loadOverview = async () => {
      try {
        const [nextStats, recent] = await Promise.all([
          linkService.fetchStats(),
          linkService.fetchPaginated({ page: 1, page_size: 5 }),
        ])
        setStats(nextStats)
        setRecentLinks(recent.data)
      } catch (error) {
        dashboardLogger.error('Failed to fetch dashboard data:', error)
      }
    }

    void loadOverview()
  }, [])

  useEffect(() => {
    if (fetchedTrend.current) return
    fetchedTrend.current = true

    const loadTrend = async () => {
      try {
        const end = new Date()
        const start = new Date()
        start.setDate(end.getDate() - 7)
        const trends = await analyticsService.getTrends({
          start_date: start.toISOString(),
          end_date: end.toISOString(),
          group_by: 'day',
          limit: null,
        })
        const chartData = trends.labels.map((label, index) => ({
          date: label,
          clicks: Number(trends.values[index]),
        }))
        setTrendData(chartData)

        if (chartData.length < 2) {
          setClickChange(null)
          return
        }

        const today = chartData.at(-1)?.clicks ?? 0
        const yesterday = chartData.at(-2)?.clicks ?? 0
        setClickChange(
          yesterday > 0
            ? ((today - yesterday) / yesterday) * 100
            : today > 0
              ? 100
              : 0,
        )
      } catch (error) {
        dashboardLogger.error('Failed to fetch trend data:', error)
      } finally {
        setTrendLoading(false)
      }
    }

    void loadTrend()
  }, [])

  return { recentLinks, stats, trendData, trendLoading, clickChange }
}
