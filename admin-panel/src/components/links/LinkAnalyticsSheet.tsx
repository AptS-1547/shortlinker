import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiExternalLink as ExternalLink,
  FiMonitor as Monitor,
  FiTrendingUp as TrendingUp,
} from 'react-icons/fi'
import { useNavigate } from 'react-router-dom'
import {
  CartesianGrid,
  Line,
  LineChart,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { COLORS } from '@/components/analytics/utils'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Skeleton } from '@/components/ui/skeleton'
import { analyticsService } from '@/services/analyticsService'
import type {
  DeviceAnalyticsResponse,
  LinkAnalytics,
} from '@/services/types.generated'

interface LinkAnalyticsSheetProps {
  code: string | null
  onClose: () => void
}

export function LinkAnalyticsSheet({ code, onClose }: LinkAnalyticsSheetProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [data, setData] = useState<LinkAnalytics | null>(null)
  const [deviceData, setDeviceData] = useState<DeviceAnalyticsResponse | null>(
    null,
  )
  const [loading, setLoading] = useState(false)

  const fetchData = useCallback(async () => {
    if (!code) return
    setLoading(true)
    try {
      const [result, devices] = await Promise.all([
        analyticsService.getLinkAnalytics(code),
        analyticsService.getLinkDeviceStats(code),
      ])
      setData(result)
      setDeviceData(devices)
    } catch (err) {
      console.error('Failed to fetch link analytics:', err)
    } finally {
      setLoading(false)
    }
  }, [code])

  useEffect(() => {
    if (code) {
      fetchData()
    } else {
      setData(null)
      setDeviceData(null)
    }
  }, [code, fetchData])

  const handleViewDetail = () => {
    onClose()
    navigate(`/links/${code}`)
  }

  const trendChartData =
    data?.trend.labels.map((label, i) => ({
      date: label,
      clicks: Number(data.trend.values[i]),
    })) ?? []

  const referrerChartData =
    data?.top_referrers.map((ref, index) => ({
      name: ref.referrer,
      value: Number(ref.count),
      percentage: ref.percentage,
      fill: COLORS[index % COLORS.length],
    })) ?? []

  return (
    <Sheet open={!!code} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="w-full sm:max-w-lg overflow-y-auto px-4">
        <SheetHeader className="pr-8">
          <div className="flex items-center justify-between">
            <div>
              <SheetTitle className="font-mono">/{code}</SheetTitle>
              <SheetDescription>
                {loading ? (
                  <Skeleton className="h-5 w-32" />
                ) : (
                  `${Number(data?.total_clicks ?? 0).toLocaleString()} ${t('analytics.clicks')}`
                )}
              </SheetDescription>
            </div>
            <Button variant="outline" size="sm" onClick={handleViewDetail}>
              <ExternalLink className="w-4 h-4 mr-1" />
              {t('linkDetail.viewFull')}
            </Button>
          </div>
        </SheetHeader>

        <div className="space-y-4">
          {/* Trend Chart */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <TrendingUp className="w-4 h-4" />
                {t('analytics.clickTrends')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {loading ? (
                <Skeleton className="h-50 w-full" />
              ) : trendChartData.length > 0 ? (
                <ResponsiveContainer width="100%" height={200}>
                  <LineChart data={trendChartData}>
                    <CartesianGrid
                      strokeDasharray="3 3"
                      className="stroke-muted"
                    />
                    <XAxis
                      dataKey="date"
                      className="text-xs"
                      tick={{
                        fill: 'var(--muted-foreground)',
                        fontSize: 10,
                      }}
                    />
                    <YAxis
                      className="text-xs"
                      tick={{
                        fill: 'var(--muted-foreground)',
                        fontSize: 10,
                      }}
                      width={30}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--popover)',
                        border: '1px solid var(--border)',
                        borderRadius: '6px',
                      }}
                      labelStyle={{ color: 'var(--popover-foreground)' }}
                      itemStyle={{ color: 'var(--popover-foreground)' }}
                    />
                    <Line
                      type="monotone"
                      dataKey="clicks"
                      stroke="var(--primary)"
                      strokeWidth={2}
                      dot={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-50 flex items-center justify-center text-muted-foreground text-sm">
                  {t('analytics.noData')}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Referrer Chart - compact */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">
                {t('analytics.referrers')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {loading ? (
                <Skeleton className="h-40 w-full" />
              ) : referrerChartData.length > 0 ? (
                <>
                  <ResponsiveContainer width="100%" height={160}>
                    <PieChart>
                      <Pie
                        data={referrerChartData}
                        cx="50%"
                        cy="50%"
                        innerRadius={40}
                        outerRadius={60}
                        paddingAngle={5}
                        dataKey="value"
                      />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'var(--popover)',
                          border: '1px solid var(--border)',
                          borderRadius: '6px',
                        }}
                        labelStyle={{ color: 'var(--popover-foreground)' }}
                        itemStyle={{ color: 'var(--popover-foreground)' }}
                        formatter={(value, _name, entry) => {
                          const payload =
                            entry.payload as (typeof referrerChartData)[number]
                          return [
                            `${value} (${payload.percentage.toFixed(1)}%)`,
                            payload.name,
                          ]
                        }}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                  <div className="space-y-1.5 mt-2">
                    {referrerChartData.slice(0, 5).map((item, index) => (
                      <div key={item.name} className="flex items-center gap-2">
                        <div
                          className="w-2.5 h-2.5 rounded-full shrink-0"
                          style={{
                            backgroundColor: COLORS[index % COLORS.length],
                          }}
                        />
                        <span className="text-xs text-muted-foreground truncate flex-1">
                          {item.name}
                        </span>
                        <span className="text-xs font-medium">
                          {item.percentage.toFixed(1)}%
                        </span>
                      </div>
                    ))}
                  </div>
                </>
              ) : (
                <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                  {t('analytics.noData')}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Geo Distribution - compact */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">
                {t('analytics.geoDistribution')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {loading ? (
                <Skeleton className="h-40 w-full" />
              ) : (data?.geo_distribution ?? []).length > 0 ? (
                <div className="grid grid-cols-2 gap-2">
                  {data?.geo_distribution.slice(0, 6).map((geo, index) => (
                    <div
                      key={`${geo.country}-${geo.city ?? 'all'}`}
                      className="flex items-center gap-2 p-2 rounded-lg border border-border/50"
                    >
                      <div
                        className="w-1.5 h-6 rounded-full shrink-0"
                        style={{
                          backgroundColor: COLORS[index % COLORS.length],
                        }}
                      />
                      <div className="min-w-0">
                        <div className="text-xs font-medium truncate">
                          {geo.city
                            ? `${geo.city}, ${geo.country}`
                            : geo.country}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {Number(geo.count).toLocaleString()}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                  {t('analytics.noGeoData')}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Device Analytics - compact */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <Monitor className="w-4 h-4" />
                {t('analytics.devices')}
                {deviceData && deviceData.bot_percentage > 0 && (
                  <Badge variant="secondary" className="ml-auto text-xs">
                    Bot: {deviceData.bot_percentage.toFixed(1)}%
                  </Badge>
                )}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {loading ? (
                <Skeleton className="h-40 w-full" />
              ) : deviceData &&
                (deviceData.browsers.length > 0 ||
                  deviceData.operating_systems.length > 0) ? (
                <div className="grid grid-cols-2 gap-4">
                  {/* Browsers */}
                  <div>
                    <p className="text-xs text-muted-foreground mb-2">
                      {t('analytics.browsers')}
                    </p>
                    <div className="space-y-1.5">
                      {deviceData.browsers.slice(0, 4).map((item, index) => (
                        <div
                          key={item.name}
                          className="flex items-center gap-2"
                        >
                          <div
                            className="w-2 h-2 rounded-full shrink-0"
                            style={{
                              backgroundColor: COLORS[index % COLORS.length],
                            }}
                          />
                          <span className="text-xs text-muted-foreground truncate flex-1">
                            {item.name}
                          </span>
                          <span className="text-xs font-medium">
                            {item.percentage.toFixed(0)}%
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                  {/* OS */}
                  <div>
                    <p className="text-xs text-muted-foreground mb-2">
                      {t('analytics.operatingSystems')}
                    </p>
                    <div className="space-y-1.5">
                      {deviceData.operating_systems
                        .slice(0, 4)
                        .map((item, index) => (
                          <div
                            key={item.name}
                            className="flex items-center gap-2"
                          >
                            <div
                              className="w-2 h-2 rounded-full shrink-0"
                              style={{
                                backgroundColor: COLORS[index % COLORS.length],
                              }}
                            />
                            <span className="text-xs text-muted-foreground truncate flex-1">
                              {item.name}
                            </span>
                            <span className="text-xs font-medium">
                              {item.percentage.toFixed(0)}%
                            </span>
                          </div>
                        ))}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="h-24 flex items-center justify-center text-muted-foreground text-sm">
                  {t('analytics.noData')}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </SheetContent>
    </Sheet>
  )
}
