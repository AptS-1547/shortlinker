import { useTranslation } from 'react-i18next'
import { FiMonitor as Monitor } from 'react-icons/fi'
import { Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import type { DeviceAnalyticsResponse } from '@/services/types.generated'
import { COLORS } from './utils'

interface DeviceAnalyticsProps {
  data: DeviceAnalyticsResponse | null
  loading: boolean
}

export function DeviceAnalytics({ data, loading }: DeviceAnalyticsProps) {
  const { t } = useTranslation()

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Monitor className="w-5 h-5" />
            {t('analytics.devices')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-62.5 w-full" />
        </CardContent>
      </Card>
    )
  }

  if (!data) return null

  const sections = [
    {
      title: t('analytics.browsers'),
      data: data.browsers.map((b, i) => ({
        name: b.name,
        value: Number(b.count),
        percentage: b.percentage,
        fill: COLORS[i % COLORS.length],
      })),
    },
    {
      title: t('analytics.operatingSystems'),
      data: data.operating_systems.map((os, i) => ({
        name: os.name,
        value: Number(os.count),
        percentage: os.percentage,
        fill: COLORS[i % COLORS.length],
      })),
    },
    {
      title: t('analytics.deviceCategories'),
      data: data.devices.map((d, i) => ({
        name: d.name,
        value: Number(d.count),
        percentage: d.percentage,
        fill: COLORS[i % COLORS.length],
      })),
    },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Monitor className="w-5 h-5" />
          {t('analytics.devices')}
          {data.bot_percentage > 0 && (
            <Badge variant="secondary" className="ml-2">
              {t('analytics.botTraffic')}: {data.bot_percentage.toFixed(1)}%
            </Badge>
          )}
        </CardTitle>
        <CardDescription>{t('analytics.devicesDesc')}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {sections.map((section) => (
            <div key={section.title}>
              <h4 className="text-sm font-medium mb-3">{section.title}</h4>
              {section.data.length > 0 ? (
                <>
                  <ResponsiveContainer width="100%" height={180}>
                    <PieChart>
                      <Pie
                        data={section.data}
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
                            entry.payload as (typeof section.data)[number]
                          return [
                            `${value} (${payload.percentage.toFixed(1)}%)`,
                            payload.name,
                          ]
                        }}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                  <div className="space-y-1.5 mt-2">
                    {section.data.slice(0, 5).map((item, index) => (
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
                <div className="h-45 flex items-center justify-center text-muted-foreground text-sm">
                  {t('analytics.noData')}
                </div>
              )}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
