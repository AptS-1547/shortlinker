import { useTranslation } from 'react-i18next'
import { Area, AreaChart, ResponsiveContainer, Tooltip } from 'recharts'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

interface MiniTrendChartProps {
  data: { date: string; clicks: number }[]
  loading: boolean
}

export function MiniTrendChart({ data, loading }: MiniTrendChartProps) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium">
          {t('dashboard.stats.clickTrend')}
        </CardTitle>
        <CardDescription>{t('dashboard.stats.last7Days')}</CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <Skeleton className="h-20 w-full" />
        ) : data.length > 0 ? (
          <ResponsiveContainer width="100%" height={80}>
            <AreaChart data={data}>
              <defs>
                <linearGradient id="clickGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop
                    offset="5%"
                    stopColor="var(--primary)"
                    stopOpacity={0.3}
                  />
                  <stop
                    offset="95%"
                    stopColor="var(--primary)"
                    stopOpacity={0}
                  />
                </linearGradient>
              </defs>
              <Tooltip
                contentStyle={{
                  backgroundColor: 'var(--popover)',
                  border: '1px solid var(--border)',
                  borderRadius: '6px',
                  fontSize: '12px',
                }}
                labelStyle={{ color: 'var(--popover-foreground)' }}
                itemStyle={{ color: 'var(--popover-foreground)' }}
                labelFormatter={(_, payload) => {
                  if (payload?.[0]?.payload?.date) {
                    return payload[0].payload.date
                  }
                  return ''
                }}
              />
              <Area
                type="monotone"
                dataKey="clicks"
                stroke="var(--primary)"
                strokeWidth={2}
                fill="url(#clickGradient)"
              />
            </AreaChart>
          </ResponsiveContainer>
        ) : (
          <div className="h-20 flex items-center justify-center text-muted-foreground text-sm">
            {t('analytics.noData')}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
