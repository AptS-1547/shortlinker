import { memo, useCallback, useEffect, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiActivity as Activity,
  FiCheckCircle as CheckCircle2,
  FiClock as Clock,
  FiDatabase as Database,
  FiHardDrive as HardDrive,
  FiLink as Link,
  FiLoader as Loader2,
  FiRefreshCw as RefreshCw,
  FiXCircle as XCircle,
} from 'react-icons/fi'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import type { HealthResponse } from '@/services/types'

interface HealthModalProps {
  open: boolean
  onClose: () => void
  healthData: HealthResponse | null
  onRefresh: () => void
  loading: boolean
}

type HealthStatus = 'healthy' | 'unhealthy' | 'degraded' | 'unknown'

/**
 * 状态徽章组件
 */
const StatusBadge = memo(function StatusBadge({
  status,
  size = 'default',
}: {
  status: string
  size?: 'default' | 'sm'
}) {
  const { t } = useTranslation()
  const normalizedStatus = status.toLowerCase() as HealthStatus

  const variants: Record<
    HealthStatus,
    'default' | 'destructive' | 'secondary' | 'outline'
  > = {
    healthy: 'default',
    unhealthy: 'destructive',
    degraded: 'secondary',
    unknown: 'outline',
  }

  const icons: Record<HealthStatus, React.ReactNode> = {
    healthy: <CheckCircle2 className={size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'} />,
    unhealthy: <XCircle className={size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'} />,
    degraded: <Activity className={size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'} />,
    unknown: <Activity className={size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'} />,
  }

  return (
    <Badge variant={variants[normalizedStatus] || 'outline'} className="gap-1">
      {icons[normalizedStatus]}
      {t(`health.${normalizedStatus}`)}
    </Badge>
  )
})

/**
 * 信息项组件
 */
const InfoItem = memo(function InfoItem({
  label,
  value,
  mono = false,
}: {
  label: string
  value: React.ReactNode
  mono?: boolean
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className={mono ? 'font-mono text-sm' : 'text-sm font-medium'}>
        {value}
      </span>
    </div>
  )
})

/**
 * 统计卡片组件
 */
const StatCard = memo(function StatCard({
  icon: Icon,
  iconColor,
  title,
  value,
  description,
}: {
  icon: React.ElementType
  iconColor: string
  title: string
  value: string | number
  description: string
}) {
  return (
    <Card>
      <CardContent className="pt-4">
        <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground mb-2">
          <Icon className={`h-4 w-4 ${iconColor}`} />
          {title}
        </div>
        <p className="text-2xl font-bold">{value}</p>
        <p className="text-xs text-muted-foreground mt-1">{description}</p>
      </CardContent>
    </Card>
  )
})

/**
 * 加载骨架屏
 */
function LoadingSkeleton() {
  return (
    <div className="space-y-4">
      <Skeleton className="h-20 w-full" />
      <div className="grid grid-cols-2 gap-4">
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-24 w-full" />
      </div>
      <Skeleton className="h-32 w-full" />
    </div>
  )
}

/**
 * 健康状态模态框
 */
export default function HealthModal({
  open,
  onClose,
  healthData,
  onRefresh,
  loading,
}: HealthModalProps) {
  const { t } = useTranslation()

  // Modal 打开时自动刷新
  useEffect(() => {
    if (open) {
      onRefresh()
    }
  }, [open, onRefresh])

  const formatUptime = useCallback((seconds: number) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60

    if (days > 0) return `${days}d ${hours}h ${minutes}m`
    if (hours > 0) return `${hours}h ${minutes}m ${secs}s`
    if (minutes > 0) return `${minutes}m ${secs}s`
    return `${secs}s`
  }, [])

  const formatTimestamp = useCallback((timestamp: string) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      hour12: false,
      timeZoneName: 'short',
    })
  }, [])

  const formattedUptime = useMemo(
    () => formatUptime(healthData?.uptime || 0),
    [healthData?.uptime, formatUptime],
  )

  const responseTime = healthData?.response_time_ms || 0
  const linksCount = healthData?.checks?.storage?.links_count || 0
  const storageType =
    healthData?.checks?.storage?.backend?.storage_type || 'Unknown'
  const storageStatus = healthData?.checks?.storage?.status || 'unknown'
  const supportClick =
    healthData?.checks?.storage?.backend?.support_click || false

  // Cache health check data
  const cacheData = healthData?.checks?.cache
  const cacheStatus = cacheData?.status || 'unknown'
  const cacheType = cacheData?.cache_type || 'Unknown'
  const bloomFilterEnabled = cacheData?.bloom_filter_enabled ?? false
  const negativeCacheEnabled = cacheData?.negative_cache_enabled ?? false

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-[calc(100vw-2rem)] sm:max-w-lg max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            {t('health.title')}
          </DialogTitle>
          <DialogDescription>{t('health.description')}</DialogDescription>
        </DialogHeader>

        {healthData ? (
          <div className="space-y-4 overflow-y-auto flex-1 pr-1">
            {/* Overall Status */}
            <Card className="bg-muted/50">
              <CardContent className="pt-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-1">
                    <p className="text-sm font-medium">
                      {t('health.overallStatus')}
                    </p>
                    <StatusBadge status={healthData.status} />
                  </div>
                  <div className="text-right">
                    <p className="text-xs text-muted-foreground">
                      {t('health.responseTime')}
                    </p>
                    <p className="text-xl font-bold text-primary">
                      {responseTime}
                      <span className="text-sm font-normal text-muted-foreground ml-0.5">
                        ms
                      </span>
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Stats Grid */}
            <div className="grid grid-cols-2 gap-3">
              <StatCard
                icon={Clock}
                iconColor="text-blue-500"
                title={t('health.uptime')}
                value={formattedUptime}
                description={t('health.sinceLast')}
              />
              <StatCard
                icon={Link}
                iconColor="text-green-500"
                title={t('health.totalLinks')}
                value={linksCount}
                description={t('health.activeLinks')}
              />
            </div>

            {/* Storage Backend */}
            <Card>
              <CardContent className="pt-4">
                <div className="flex items-center gap-2 text-sm font-medium mb-3">
                  <Database className="h-4 w-4 text-purple-500" />
                  {t('health.storageBackend')}
                </div>
                <div className="space-y-3">
                  <InfoItem
                    label={t('health.type')}
                    value={
                      <Badge variant="secondary" className="font-mono">
                        {storageType}
                      </Badge>
                    }
                  />
                  <Separator />
                  <InfoItem
                    label={t('health.status')}
                    value={<StatusBadge status={storageStatus} size="sm" />}
                  />
                  <Separator />
                  <InfoItem
                    label={t('health.clickTracking')}
                    value={
                      <Badge variant={supportClick ? 'default' : 'outline'}>
                        {supportClick ? (
                          <CheckCircle2 className="h-3 w-3 mr-1" />
                        ) : (
                          <XCircle className="h-3 w-3 mr-1" />
                        )}
                        {t(supportClick ? 'health.enabled' : 'health.disabled')}
                      </Badge>
                    }
                  />
                </div>
              </CardContent>
            </Card>

            {/* Cache System */}
            {cacheData && (
              <Card>
                <CardContent className="pt-4">
                  <div className="flex items-center gap-2 text-sm font-medium mb-3">
                    <HardDrive className="h-4 w-4 text-orange-500" />
                    {t('health.cacheSystem')}
                  </div>
                  <div className="space-y-3">
                    <InfoItem
                      label={t('health.cacheType')}
                      value={
                        <Badge variant="secondary" className="font-mono">
                          {cacheType}
                        </Badge>
                      }
                    />
                    <Separator />
                    <InfoItem
                      label={t('health.status')}
                      value={<StatusBadge status={cacheStatus} size="sm" />}
                    />
                    <Separator />
                    <InfoItem
                      label={t('health.bloomFilter')}
                      value={
                        <Badge
                          variant={bloomFilterEnabled ? 'default' : 'outline'}
                        >
                          {bloomFilterEnabled ? (
                            <CheckCircle2 className="h-3 w-3 mr-1" />
                          ) : (
                            <XCircle className="h-3 w-3 mr-1" />
                          )}
                          {t(
                            bloomFilterEnabled
                              ? 'health.enabled'
                              : 'health.disabled',
                          )}
                        </Badge>
                      }
                    />
                    <Separator />
                    <InfoItem
                      label={t('health.negativeCache')}
                      value={
                        <Badge
                          variant={negativeCacheEnabled ? 'default' : 'outline'}
                        >
                          {negativeCacheEnabled ? (
                            <CheckCircle2 className="h-3 w-3 mr-1" />
                          ) : (
                            <XCircle className="h-3 w-3 mr-1" />
                          )}
                          {t(
                            negativeCacheEnabled
                              ? 'health.enabled'
                              : 'health.disabled',
                          )}
                        </Badge>
                      }
                    />
                  </div>
                </CardContent>
              </Card>
            )}

            {/* Timestamp */}
            <div className="flex items-center justify-between text-sm px-1">
              <span className="text-muted-foreground">
                {t('health.lastUpdated')}
              </span>
              <span className="font-mono text-xs">
                {formatTimestamp(healthData.timestamp as string)}
              </span>
            </div>
          </div>
        ) : (
          <LoadingSkeleton />
        )}

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={onRefresh}
            disabled={loading}
            className="gap-2"
          >
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
            {t('health.refresh')}
          </Button>
          <Button variant="outline" onClick={onClose}>
            {t('health.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
