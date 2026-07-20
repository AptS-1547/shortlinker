import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiArrowLeft as ArrowLeft,
  FiCheck as Check,
  FiCopy as Copy,
  FiDownload as Download,
  FiExternalLink as ExternalLink,
} from 'react-icons/fi'
import { useNavigate, useParams } from 'react-router-dom'
import type { DateRange } from '@/components/analytics'
import {
  COLORS,
  DateRangeSelector,
  DeviceAnalytics,
  GeoDistribution,
  getDateRange,
  ReferrerChart,
  TrendChart,
} from '@/components/analytics'
import PageHeader from '@/components/layout/PageHeader'
import { QrCodeDialog } from '@/components/links'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { analyticsService } from '@/services/analyticsService'
import { linkService } from '@/services/linkService'
import type { LinkResponse } from '@/services/types'
import type {
  DeviceAnalyticsResponse,
  GeoStats,
  GroupBy,
  LinkAnalytics,
} from '@/services/types.generated'

export default function LinkDetailPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { code } = useParams<{ code: string }>()

  // State
  const [dateRange, setDateRange] = useState<DateRange>('30d')
  const [groupBy, setGroupBy] = useState<GroupBy>('day')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  // Data
  const [linkInfo, setLinkInfo] = useState<LinkResponse | null>(null)
  const [analytics, setAnalytics] = useState<LinkAnalytics | null>(null)
  const [deviceData, setDeviceData] = useState<DeviceAnalyticsResponse | null>(
    null,
  )
  const [qrCodeDialogOpen, setQrCodeDialogOpen] = useState(false)

  const fetchData = useCallback(async () => {
    if (!code) return

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
      const [info, analyticsData, devices] = await Promise.all([
        linkService.fetchOne(code),
        analyticsService.getLinkAnalytics(code, params),
        analyticsService.getLinkDeviceStats(code, params),
      ])

      if (!info) {
        setError(t('linkDetail.notFound'))
        return
      }

      setLinkInfo(info)
      setAnalytics(analyticsData)
      setDeviceData(devices)
    } catch (err) {
      console.error('Failed to fetch link data:', err)
      setError(t('linkDetail.fetchFailed'))
    } finally {
      setLoading(false)
    }
  }, [code, dateRange, groupBy, t])

  useEffect(() => {
    fetchData()
  }, [fetchData])

  const handleCopy = async () => {
    if (!linkInfo) return
    const url = `${window.location.origin}/${code}`
    await navigator.clipboard.writeText(url)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleOpenQrCode = () => {
    setQrCodeDialogOpen(true)
  }

  // Transform data for charts
  const trendChartData =
    analytics?.trend.labels.map((label, i) => ({
      date: label,
      clicks: Number(analytics.trend.values[i]),
    })) ?? []

  const referrerChartData =
    analytics?.top_referrers.map((ref, index) => ({
      name: ref.referrer,
      value: Number(ref.count),
      percentage: ref.percentage,
      fill: COLORS[index % COLORS.length],
    })) ?? []

  const geoStats: GeoStats[] =
    analytics?.geo_distribution.map((geo) => ({
      country: geo.country,
      city: geo.city,
      count: geo.count,
    })) ?? []

  if (!code) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <p className="text-muted-foreground">{t('linkDetail.noCode')}</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={
          <div className="flex items-center gap-3">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => navigate('/links')}
            >
              <ArrowLeft className="w-4 h-4" />
            </Button>
            <span className="font-mono">/{code}</span>
          </div>
        }
        description={t('linkDetail.description')}
        actions={
          <DateRangeSelector
            dateRange={dateRange}
            setDateRange={setDateRange}
            groupBy={groupBy}
            setGroupBy={setGroupBy}
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

      {/* Link Info Card */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">{t('linkDetail.info')}</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="space-y-3">
              <Skeleton className="h-5 w-full" />
              <Skeleton className="h-5 w-3/4" />
              <Skeleton className="h-5 w-1/2" />
            </div>
          ) : linkInfo ? (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <p className="text-sm text-muted-foreground">
                    {t('linkDetail.targetUrl')}
                  </p>
                  <a
                    href={linkInfo.target}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline flex items-center gap-1"
                  >
                    {linkInfo.target}
                    <ExternalLink className="w-3 h-3" />
                  </a>
                </div>
                <Button variant="outline" size="sm" onClick={handleCopy}>
                  {copied ? (
                    <Check className="w-4 h-4 mr-1" />
                  ) : (
                    <Copy className="w-4 h-4 mr-1" />
                  )}
                  {copied ? t('common.copied') : t('common.copy')}
                </Button>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <p className="text-sm text-muted-foreground">
                    {t('linkDetail.totalClicks')}
                  </p>
                  <p className="text-2xl font-semibold">
                    {Number(analytics?.total_clicks ?? 0).toLocaleString()}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">
                    {t('linkDetail.createdAt')}
                  </p>
                  <p className="font-medium">
                    {new Date(linkInfo.created_at).toLocaleDateString()}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">
                    {t('linkDetail.status')}
                  </p>
                  {linkInfo.expires_at &&
                  new Date(linkInfo.expires_at) < new Date() ? (
                    <Badge variant="secondary">{t('links.expired')}</Badge>
                  ) : (
                    <Badge variant="default">{t('links.active')}</Badge>
                  )}
                </div>
                {linkInfo.expires_at && (
                  <div>
                    <p className="text-sm text-muted-foreground">
                      {t('linkDetail.expiresAt')}
                    </p>
                    <p className="font-medium">
                      {new Date(linkInfo.expires_at).toLocaleDateString()}
                    </p>
                  </div>
                )}
              </div>

              {/* QR Code Button */}
              <div className="flex items-center gap-4 pt-4 border-t">
                <Button variant="outline" size="sm" onClick={handleOpenQrCode}>
                  <Download className="w-4 h-4 mr-1" />
                  {t('links.qr.download')}
                </Button>
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>

      {/* Trend Chart */}
      <TrendChart data={trendChartData} loading={loading} />

      {/* Referrer + Geo */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ReferrerChart data={referrerChartData} loading={loading} />
        <GeoDistribution data={geoStats} loading={loading} />
      </div>

      {/* Device Analytics */}
      <DeviceAnalytics data={deviceData} loading={loading} />

      {/* QR Code Dialog */}
      <QrCodeDialog
        code={qrCodeDialogOpen ? (code ?? null) : null}
        onClose={() => setQrCodeDialogOpen(false)}
      />
    </div>
  )
}
