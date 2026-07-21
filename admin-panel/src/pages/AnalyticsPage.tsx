import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { DateRange } from '@/components/analytics'
import {
  AnalyticsSummaryCards,
  COLORS,
  DateRangeSelector,
  DeviceAnalytics,
  GeoDistribution,
  ReferrerChart,
  TopLinksChart,
  TrendChart,
} from '@/components/analytics'
import PageHeader from '@/components/layout/PageHeader'
import { Card, CardContent } from '@/components/ui/card'
import { useAnalyticsData } from '@/hooks/useAnalyticsData'
import type { GroupBy } from '@/services/types'

export default function AnalyticsPage() {
  const { t } = useTranslation()

  // State
  const [dateRange, setDateRange] = useState<DateRange>('30d')
  const [groupBy, setGroupBy] = useState<GroupBy>('day')
  const {
    loading,
    error,
    trendData,
    topLinks,
    referrers,
    geoStats,
    deviceData,
    exportReport,
  } = useAnalyticsData(dateRange, groupBy)

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
            onExport={exportReport}
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
