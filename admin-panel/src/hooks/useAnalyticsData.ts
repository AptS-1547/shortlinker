import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { DateRange } from '@/components/analytics'
import { getDateRange } from '@/components/analytics'
import { analyticsService } from '@/services/analyticsService'
import type {
  DeviceAnalyticsResponse,
  GeoStats,
  GroupBy,
  ReferrerStats,
  TopLink,
  TrendData,
} from '@/services/types'
import { logger } from '@/utils/logger'

export function useAnalyticsData(dateRange: DateRange, groupBy: GroupBy) {
  const { t } = useTranslation()
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [trendData, setTrendData] = useState<TrendData | null>(null)
  const [topLinks, setTopLinks] = useState<TopLink[]>([])
  const [referrers, setReferrers] = useState<ReferrerStats[]>([])
  const [geoStats, setGeoStats] = useState<GeoStats[]>([])
  const [deviceData, setDeviceData] = useState<DeviceAnalyticsResponse | null>(
    null,
  )
  const requestIdRef = useRef(0)

  const reload = useCallback(async () => {
    const requestId = ++requestIdRef.current
    setLoading(true)
    setError(null)
    const { start, end } = getDateRange(dateRange)
    const params = {
      start_date: start,
      end_date: end,
      group_by: groupBy,
      limit: 10,
    }

    try {
      const [trends, top, refs, geo, devices] = await Promise.all([
        analyticsService.getTrends(params),
        analyticsService.getTopLinks(params),
        analyticsService.getReferrers(params),
        analyticsService.getGeoStats(params),
        analyticsService.getDeviceStats(params),
      ])
      if (requestId !== requestIdRef.current) return
      setTrendData(trends)
      setTopLinks(top)
      setReferrers(refs)
      setGeoStats(geo)
      setDeviceData(devices)
    } catch (requestError) {
      if (requestId !== requestIdRef.current) return
      logger.error('Failed to fetch analytics:', requestError)
      setError(t('analytics.error.fetchFailed'))
    } finally {
      if (requestId === requestIdRef.current) setLoading(false)
    }
  }, [dateRange, groupBy, t])

  useEffect(() => {
    void reload()
    return () => {
      requestIdRef.current += 1
    }
  }, [reload])

  const exportReport = useCallback(async () => {
    try {
      const { start, end } = getDateRange(dateRange)
      const blob = await analyticsService.exportReport({
        start_date: start,
        end_date: end,
        group_by: null,
        limit: null,
      })
      const url = URL.createObjectURL(blob)
      const anchor = document.createElement('a')
      anchor.href = url
      anchor.download = `analytics_${dateRange}.csv`
      document.body.appendChild(anchor)
      anchor.click()
      anchor.remove()
      URL.revokeObjectURL(url)
    } catch (requestError) {
      logger.error('Failed to export analytics:', requestError)
    }
  }, [dateRange])

  return {
    loading,
    error,
    trendData,
    topLinks,
    referrers,
    geoStats,
    deviceData,
    reload,
    exportReport,
  }
}
