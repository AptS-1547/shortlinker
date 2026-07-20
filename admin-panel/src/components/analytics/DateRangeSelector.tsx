import { useTranslation } from 'react-i18next'
import { FiDownload as Download } from 'react-icons/fi'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { GroupBy } from '@/services/types.generated'
import type { DateRange } from './utils'

interface DateRangeSelectorProps {
  dateRange: DateRange
  setDateRange: (value: DateRange) => void
  groupBy: GroupBy
  setGroupBy: (value: GroupBy) => void
  onExport?: () => void
}

export function DateRangeSelector({
  dateRange,
  setDateRange,
  groupBy,
  setGroupBy,
  onExport,
}: DateRangeSelectorProps) {
  const { t } = useTranslation()

  return (
    <div className="flex items-center gap-2">
      {/* Date Range Selector */}
      <Select
        value={dateRange}
        onValueChange={(v) => setDateRange(v as DateRange)}
      >
        <SelectTrigger className="w-30">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="7d">{t('analytics.dateRange.7d')}</SelectItem>
          <SelectItem value="30d">{t('analytics.dateRange.30d')}</SelectItem>
          <SelectItem value="90d">{t('analytics.dateRange.90d')}</SelectItem>
          <SelectItem value="1y">{t('analytics.dateRange.1y')}</SelectItem>
        </SelectContent>
      </Select>

      {/* Group By Selector */}
      <Select value={groupBy} onValueChange={(v) => setGroupBy(v as GroupBy)}>
        <SelectTrigger className="w-25">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="hour">{t('analytics.groupBy.hour')}</SelectItem>
          <SelectItem value="day">{t('analytics.groupBy.day')}</SelectItem>
          <SelectItem value="week">{t('analytics.groupBy.week')}</SelectItem>
          <SelectItem value="month">{t('analytics.groupBy.month')}</SelectItem>
        </SelectContent>
      </Select>

      {/* Export Button */}
      {onExport && (
        <Button variant="outline" size="sm" onClick={onExport}>
          <Download className="w-4 h-4 mr-2" />
          {t('analytics.export')}
        </Button>
      )}
    </div>
  )
}
