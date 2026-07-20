import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import { FiSettings as Settings2 } from 'react-icons/fi'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { ColumnKey } from './LinksTable'
import { DEFAULT_VISIBLE_COLUMNS } from './LinksTable'

// 列名翻译映射
const COLUMN_LABELS: Record<ColumnKey, string> = {
  code: 'links.table.code',
  target: 'links.table.target',
  clicks: 'links.table.clicks',
  status: 'links.table.status',
  created: 'links.table.created',
  expires: 'links.table.expires',
}

interface ColumnConfigDropdownProps {
  visibleColumns: ColumnKey[]
  onColumnToggle: (col: ColumnKey, checked: boolean) => void
}

export const ColumnConfigDropdown = memo(function ColumnConfigDropdown({
  visibleColumns,
  onColumnToggle,
}: ColumnConfigDropdownProps) {
  const { t } = useTranslation()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="sm" className="gap-1">
          <Settings2 className="w-4 h-4" />
          {t('links.columns.configure')}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        <DropdownMenuLabel>{t('links.columns.title')}</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {DEFAULT_VISIBLE_COLUMNS.map((col) => (
          <div key={col} className="flex items-center space-x-2 px-2 py-1.5">
            <Checkbox
              id={`col-${col}`}
              checked={visibleColumns.includes(col)}
              onCheckedChange={(checked) => onColumnToggle(col, !!checked)}
            />
            <label
              htmlFor={`col-${col}`}
              className="text-sm font-normal cursor-pointer flex-1"
            >
              {t(COLUMN_LABELS[col])}
            </label>
          </div>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
})
