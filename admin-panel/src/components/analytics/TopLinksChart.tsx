import { useTranslation } from 'react-i18next'
import { FiLink as Link2 } from 'react-icons/fi'
import {
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

interface TopLinksChartProps {
  data: { code: string; clicks: number }[]
  loading: boolean
}

export function TopLinksChart({ data, loading }: TopLinksChartProps) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Link2 className="w-5 h-5" />
          {t('analytics.topLinks')}
        </CardTitle>
        <CardDescription>{t('analytics.topLinksDesc')}</CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <Skeleton className="h-62.5 w-full" />
        ) : data.length > 0 ? (
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={data} layout="vertical">
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis
                type="number"
                className="text-xs"
                tick={{ fill: 'var(--muted-foreground)' }}
              />
              <YAxis
                type="category"
                dataKey="code"
                width={80}
                className="text-xs"
                tick={{ fill: 'var(--muted-foreground)' }}
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
              <Bar dataKey="clicks" fill="var(--primary)" />
            </BarChart>
          </ResponsiveContainer>
        ) : (
          <div className="h-62.5 flex items-center justify-center text-muted-foreground">
            {t('analytics.noData')}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
