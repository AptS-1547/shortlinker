import { format } from 'date-fns'
import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiCalendar as CalendarIcon,
  FiFilter as Filter,
  FiSearch as Search,
  FiX as X,
} from 'react-icons/fi'
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import { Input } from '@/components/ui/input'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { StatusFilter } from '@/hooks/useLinksFilters'
import { cn } from '@/lib/utils'

interface LinksFilterBarProps {
  searchQuery: string
  statusFilter: StatusFilter
  createdAfter: Date | undefined
  createdBefore: Date | undefined
  onSearchChange: (value: string) => void
  onStatusChange: (value: StatusFilter) => void
  onCreatedAfterChange: (date: Date | undefined) => void
  onCreatedBeforeChange: (date: Date | undefined) => void
}

export const LinksFilterBar = memo(function LinksFilterBar({
  searchQuery,
  statusFilter,
  createdAfter,
  createdBefore,
  onSearchChange,
  onStatusChange,
  onCreatedAfterChange,
  onCreatedBeforeChange,
}: LinksFilterBarProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-wrap items-center gap-4">
      {/* 搜索框 */}
      <div className="relative w-full md:w-[240px]">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder={t('links.search.placeholder')}
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-9"
        />
      </div>

      {/* 创建时间起始 */}
      <Popover>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            className={cn(
              'w-full md:w-[180px] justify-start text-left font-normal',
              !createdAfter && 'text-muted-foreground',
            )}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            {createdAfter
              ? format(createdAfter, 'yyyy-MM-dd')
              : t('links.createdAfter')}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="single"
            selected={createdAfter}
            onSelect={onCreatedAfterChange}
            initialFocus
          />
        </PopoverContent>
      </Popover>
      {createdAfter && (
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() => onCreatedAfterChange(undefined)}
        >
          <X className="h-4 w-4" />
        </Button>
      )}

      {/* 创建时间结束 */}
      <Popover>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            className={cn(
              'w-full md:w-[180px] justify-start text-left font-normal',
              !createdBefore && 'text-muted-foreground',
            )}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            {createdBefore
              ? format(createdBefore, 'yyyy-MM-dd')
              : t('links.createdBefore')}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="single"
            selected={createdBefore}
            onSelect={onCreatedBeforeChange}
            initialFocus
          />
        </PopoverContent>
      </Popover>
      {createdBefore && (
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() => onCreatedBeforeChange(undefined)}
        >
          <X className="h-4 w-4" />
        </Button>
      )}

      {/* 状态筛选 */}
      <Select value={statusFilter} onValueChange={onStatusChange}>
        <SelectTrigger className="w-full md:w-[160px]">
          <Filter className="w-4 h-4 mr-2" />
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">
            {t('links.filterOptions.allLinks')}
          </SelectItem>
          <SelectItem value="active">
            {t('links.filterOptions.activeOnly')}
          </SelectItem>
          <SelectItem value="expired">
            {t('links.filterOptions.expiredOnly')}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
})
