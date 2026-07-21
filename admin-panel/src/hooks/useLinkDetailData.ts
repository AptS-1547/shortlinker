import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { DateRange } from '@/components/analytics'
import { getDateRange } from '@/components/analytics'
import { analyticsService } from '@/services/analyticsService'
import { linkService } from '@/services/linkService'
import type {
  DeviceAnalyticsResponse,
  GroupBy,
  LinkAnalytics,
  LinkResponse,
} from '@/services/types'
import { logger } from '@/utils/logger'

export function useLinkDetailData(
  code: string | undefined,
  dateRange: DateRange,
  groupBy: GroupBy,
) {
  const { t } = useTranslation()
  const [loading, setLoading] = useState(Boolean(code))
  const [error, setError] = useState<string | null>(null)
  const [linkInfo, setLinkInfo] = useState<LinkResponse | null>(null)
  const [analytics, setAnalytics] = useState<LinkAnalytics | null>(null)
  const [deviceData, setDeviceData] = useState<DeviceAnalyticsResponse | null>(
    null,
  )
  const requestIdRef = useRef(0)

  const reload = useCallback(async () => {
    const requestId = ++requestIdRef.current
    if (!code) {
      setLoading(false)
      setLinkInfo(null)
      setAnalytics(null)
      setDeviceData(null)
      return
    }

    setLoading(true)
    setError(null)
    setLinkInfo(null)
    setAnalytics(null)
    setDeviceData(null)
    const { start, end } = getDateRange(dateRange)
    const params = {
      start_date: start,
      end_date: end,
      group_by: groupBy,
      limit: 10,
    }

    try {
      const [info, analyticsData, devices] = await Promise.all([
        linkService.fetchOne(code),
        analyticsService.getLinkAnalytics(code, params),
        analyticsService.getLinkDeviceStats(code, params),
      ])
      if (requestId !== requestIdRef.current) return
      if (!info) {
        setError(t('linkDetail.notFound'))
        return
      }
      setLinkInfo(info)
      setAnalytics(analyticsData)
      setDeviceData(devices)
    } catch (requestError) {
      if (requestId !== requestIdRef.current) return
      logger.error('Failed to fetch link data:', requestError)
      setError(t('linkDetail.fetchFailed'))
    } finally {
      if (requestId === requestIdRef.current) setLoading(false)
    }
  }, [code, dateRange, groupBy, t])

  useEffect(() => {
    void reload()
    return () => {
      requestIdRef.current += 1
    }
  }, [reload])

  return { loading, error, linkInfo, analytics, deviceData, reload }
}
