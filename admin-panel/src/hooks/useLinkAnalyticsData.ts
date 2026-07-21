import { useCallback, useEffect, useRef, useState } from 'react'
import { analyticsService } from '@/services/analyticsService'
import type { DeviceAnalyticsResponse, LinkAnalytics } from '@/services/types'
import { logger } from '@/utils/logger'

export function useLinkAnalyticsData(code: string | null) {
  const [data, setData] = useState<LinkAnalytics | null>(null)
  const [deviceData, setDeviceData] = useState<DeviceAnalyticsResponse | null>(
    null,
  )
  const [loading, setLoading] = useState(false)
  const requestIdRef = useRef(0)

  const reload = useCallback(async () => {
    const requestId = ++requestIdRef.current
    if (!code) {
      setData(null)
      setDeviceData(null)
      setLoading(false)
      return
    }

    setLoading(true)
    setData(null)
    setDeviceData(null)
    try {
      const [analytics, devices] = await Promise.all([
        analyticsService.getLinkAnalytics(code),
        analyticsService.getLinkDeviceStats(code),
      ])
      if (requestId !== requestIdRef.current) return
      setData(analytics)
      setDeviceData(devices)
    } catch (error) {
      if (requestId !== requestIdRef.current) return
      logger.error('Failed to fetch link analytics:', error)
    } finally {
      if (requestId === requestIdRef.current) setLoading(false)
    }
  }, [code])

  useEffect(() => {
    void reload()
    return () => {
      requestIdRef.current += 1
    }
  }, [reload])

  return { data, deviceData, loading, reload }
}
