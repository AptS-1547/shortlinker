import { useTranslation } from 'react-i18next'
import { FiGlobe as Globe } from 'react-icons/fi'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import type { GeoStats } from '@/services/types.generated'
import { COLORS } from './utils'

interface GeoDistributionProps {
  data: GeoStats[]
  loading: boolean
  maxItems?: number
}

export function GeoDistribution({
  data,
  loading,
  maxItems = 9,
}: GeoDistributionProps) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Globe className="w-5 h-5" />
          {t('analytics.geoDistribution')}
        </CardTitle>
        <CardDescription>{t('analytics.geoDistributionDesc')}</CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <Skeleton className="h-50 w-full" />
        ) : data.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {data.slice(0, maxItems).map((geo, index) => (
              <div
                key={`${geo.country}-${geo.city ?? 'all'}`}
                className="flex items-center gap-3 p-3 rounded-lg border border-border/50"
              >
                <div
                  className="w-2 h-8 rounded-full"
                  style={{ backgroundColor: COLORS[index % COLORS.length] }}
                />
                <div className="flex-1 min-w-0">
                  <div className="font-medium truncate">
                    {geo.country}
                    {geo.city && (
                      <span className="text-muted-foreground">
                        {' '}
                        / {geo.city}
                      </span>
                    )}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {Number(geo.count).toLocaleString()} {t('analytics.clicks')}
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="h-50 flex items-center justify-center text-muted-foreground">
            {t('analytics.noGeoData')}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
