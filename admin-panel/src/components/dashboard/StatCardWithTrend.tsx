import { FiArrowDown as ArrowDown, FiArrowUp as ArrowUp } from 'react-icons/fi'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

interface StatCardWithTrendProps {
  title: string
  value: number | string
  change?: number | null // percentage change, e.g. 12.5 means +12.5%
  changeLabel?: string
  icon: React.ReactNode
  loading?: boolean
}

export function StatCardWithTrend({
  title,
  value,
  change,
  changeLabel,
  icon,
  loading,
}: StatCardWithTrendProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        {icon}
      </CardHeader>
      <CardContent>
        {loading ? (
          <Skeleton className="h-8 w-24" />
        ) : (
          <>
            <div className="text-2xl font-bold">
              {typeof value === 'number' ? value.toLocaleString() : value}
            </div>
            {change != null && (
              <p className="text-xs text-muted-foreground flex items-center gap-1 mt-1">
                {change > 0 ? (
                  <ArrowUp className="w-3 h-3 text-green-500" />
                ) : change < 0 ? (
                  <ArrowDown className="w-3 h-3 text-red-500" />
                ) : null}
                <span
                  className={
                    change > 0
                      ? 'text-green-500'
                      : change < 0
                        ? 'text-red-500'
                        : ''
                  }
                >
                  {change > 0 ? '+' : ''}
                  {change.toFixed(1)}%
                </span>
                {changeLabel && <span>{changeLabel}</span>}
              </p>
            )}
          </>
        )}
      </CardContent>
    </Card>
  )
}
