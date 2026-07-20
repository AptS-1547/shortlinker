import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { DateRange } from '@/components/analytics'
import {
  AnalyticsSummaryCards,
  COLORS,
  DateRangeSelector,
  DeviceAnalytics,
  GeoDistribution,
  getDateRange,
  ReferrerChart,
  TopLinksChart,
  TrendChart,
} from '@/components/analytics'
import PageHeader from '@/components/layout/PageHeader'
import { Card, CardContent } from '@/components/ui/card'
import { analyticsService } from '@/services/analyticsService'
import type {
  DeviceAnalyticsResponse,
  GeoStats,
  GroupBy,
  ReferrerStats,
  TopLink,
  TrendData,
} from '@/services/types.generated'

export default function AnalyticsPage() {
  const { t } = useTranslation()

  // State
  const [dateRange, setDateRange] = useState<DateRange>('30d')
  const [groupBy, setGroupBy] = useState<GroupBy>('day')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Data
  const [trendData, setTrendData] = useState<TrendData | null>(null)
  const [topLinks, setTopLinks] = useState<TopLink[]>([])
  const [referrers, setReferrers] = useState<ReferrerStats[]>([])
  const [geoStats, setGeoStats] = useState<GeoStats[]>([])
  const [deviceData, setDeviceData] = useState<DeviceAnalyticsResponse | null>(
    null,
  )

  // Fetch data
  const fetchData = useCallback(async () => {
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

      setTrendData(trends)
      setTopLinks(top)
      setReferrers(refs)
      setGeoStats(geo)
      setDeviceData(devices)
    } catch (err) {
      console.error('Failed to fetch analytics:', err)
      setError(t('analytics.error.fetchFailed'))
    } finally {
      setLoading(false)
    }
  }, [dateRange, groupBy, t])

  useEffect(() => {
    fetchData()
  }, [fetchData])

  // Export handler
  const handleExport = async () => {
    try {
      const { start, end } = getDateRange(dateRange)
      const blob = await analyticsService.exportReport({
        start_date: start,
        end_date: end,
        group_by: null,
        limit: null,
      })

      // Download
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `analytics_${dateRange}.csv`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    } catch (err) {
      console.error('Export failed:', err)
    }
  }

  // Calculate total clicks
  const totalClicks =
    trendData?.values.reduce((sum, v) => sum + Number(v), 0) ?? 0

  // Transform data for charts
  const trendChartData =
    trendData?.labels.map((label, i) => ({
      date: label,
      clicks: Number(trendData.values[i]),
    })) ?? []

  const topLinksChartData = topLinks.map((link) => ({
    code: link.code,
    clicks: Number(link.clicks),
  }))

  const referrerChartData = referrers.map((ref, index) => ({
    name: ref.referrer,
    value: Number(ref.count),
    percentage: ref.percentage,
    fill: COLORS[index % COLORS.length],
  }))

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('analytics.title')}
        description={t('analytics.description')}
        actions={
          <DateRangeSelector
            dateRange={dateRange}
            setDateRange={setDateRange}
            groupBy={groupBy}
            setGroupBy={setGroupBy}
            onExport={handleExport}
          />
        }
      />

      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-6">
            <p className="text-destructive">{error}</p>
          </CardContent>
        </Card>
      )}

      <AnalyticsSummaryCards
        totalClicks={totalClicks}
        topLinks={topLinks}
        referrers={referrers}
        geoStats={geoStats}
        loading={loading}
      />

      <TrendChart data={trendChartData} loading={loading} />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <TopLinksChart data={topLinksChartData} loading={loading} />
        <ReferrerChart data={referrerChartData} loading={loading} />
      </div>

      <DeviceAnalytics data={deviceData} loading={loading} />

      <GeoDistribution data={geoStats} loading={loading} />
    </div>
  )
}
