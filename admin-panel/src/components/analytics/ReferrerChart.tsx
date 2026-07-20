import { useTranslation } from 'react-i18next'
import { FiBarChart2 as BarChart3 } from 'react-icons/fi'
import { Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { COLORS } from './utils'

interface ReferrerChartDataItem {
  name: string
  value: number
  percentage: number
  fill: string
}

interface ReferrerChartProps {
  data: ReferrerChartDataItem[]
  loading: boolean
}

export function ReferrerChart({ data, loading }: ReferrerChartProps) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <BarChart3 className="w-5 h-5" />
          {t('analytics.referrers')}
        </CardTitle>
        <CardDescription>{t('analytics.referrersDesc')}</CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <Skeleton className="h-62.5 w-full" />
        ) : data.length > 0 ? (
          <ResponsiveContainer width="100%" height={250}>
            <PieChart>
              <Pie
                data={data}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={80}
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
                  const payload = entry.payload as ReferrerChartDataItem
                  return [
                    `${value} (${payload.percentage.toFixed(1)}%)`,
                    payload.name,
                  ]
                }}
              />
            </PieChart>
          </ResponsiveContainer>
        ) : (
          <div className="h-62.5 flex items-center justify-center text-muted-foreground">
            {t('analytics.noData')}
          </div>
        )}
        {/* Legend */}
        {data.length > 0 && (
          <div className="mt-4 space-y-2">
            {data.slice(0, 5).map((item, index) => (
              <div key={item.name} className="flex items-center gap-2">
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: COLORS[index % COLORS.length] }}
                />
                <span className="text-sm text-muted-foreground truncate flex-1">
                  {item.name}
                </span>
                <span className="text-sm font-medium">
                  {item.percentage.toFixed(1)}%
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
