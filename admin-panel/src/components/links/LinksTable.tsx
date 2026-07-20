import { memo, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiArrowDown as ArrowDown,
  FiArrowUp as ArrowUp,
  FiBarChart2 as BarChart3Icon,
  FiCheck as Check,
  FiChevronDown as ChevronDown,
  FiChevronRight as ChevronRight,
  FiCopy as Copy,
  FiDownload as Download,
  FiEdit as Edit,
  FiExternalLink as ExternalLink,
  FiLock as Lock,
  FiMoreHorizontal as MoreHorizontal,
  FiTrash2 as Trash2,
} from 'react-icons/fi'
import { HiSelector as ArrowUpDown } from 'react-icons/hi'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useDateFormat } from '@/hooks/useDateFormat'
import type { LinkResponse } from '@/services/types'
import { buildShortUrl } from '@/utils/urlBuilder'

export type SortField = 'clicks' | 'created_at' | null
export type SortDirection = 'asc' | 'desc'
export type ColumnKey =
  | 'code'
  | 'target'
  | 'clicks'
  | 'status'
  | 'created'
  | 'expires'

// 默认显示的列
export const DEFAULT_VISIBLE_COLUMNS: ColumnKey[] = [
  'code',
  'target',
  'clicks',
  'status',
  'created',
  'expires',
]

interface LinksTableProps {
  links: LinkResponse[]
  copiedCode: string | null
  selectedCodes: Set<string>
  sortField: SortField
  sortDirection: SortDirection
  visibleColumns: ColumnKey[]
  onCopy: (code: string) => void
  onEdit: (link: LinkResponse) => void
  onDelete: (link: LinkResponse) => void
  onSelectChange: (code: string, checked: boolean) => void
  onSelectAll: (checked: boolean) => void
  onSort: (field: SortField) => void
  onViewAnalytics?: (code: string) => void
  onDownloadQr?: (code: string) => void
}

/**
 * 判断链接是否已过期
 */
function isExpired(expiresAt: string | null): boolean {
  if (!expiresAt) return false
  return new Date(expiresAt) < new Date()
}

/**
 * 单个链接行组件（包含展开功能）
 */
const LinkRow = memo(function LinkRow({
  link,
  copiedCode,
  isSelected,
  visibleColumns,
  columnCount,
  onCopy,
  onEdit,
  onDelete,
  onSelectChange,
  onViewAnalytics,
  onDownloadQr,
}: {
  link: LinkResponse
  copiedCode: string | null
  isSelected: boolean
  visibleColumns: ColumnKey[]
  columnCount: number
  onCopy: (code: string) => void
  onEdit: (link: LinkResponse) => void
  onDelete: (link: LinkResponse) => void
  onSelectChange: (code: string, checked: boolean) => void
  onViewAnalytics?: (code: string) => void
  onDownloadQr?: (code: string) => void
}) {
  const { t } = useTranslation()
  const { formatDateTime } = useDateFormat()
  const [isExpanded, setIsExpanded] = useState(false)

  const isVisible = (col: ColumnKey) => visibleColumns.includes(col)

  return (
    <>
      <TableRow data-state={isSelected ? 'selected' : undefined}>
        {/* 展开按钮 + Checkbox 列 */}
        <TableCell className="w-[70px]">
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={() => setIsExpanded(!isExpanded)}
              aria-label={
                isExpanded ? t('common.collapse') : t('common.expand')
              }
              aria-expanded={isExpanded}
            >
              {isExpanded ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
            </Button>
            <Checkbox
              checked={isSelected}
              onCheckedChange={(checked) =>
                onSelectChange(link.code, !!checked)
              }
              aria-label={t('links.batch.selectRow')}
            />
          </div>
        </TableCell>

        {/* Code 列 - 可点击复制 */}
        {isVisible('code') && (
          <TableCell>
            <button
              type="button"
              onClick={() => onCopy(link.code)}
              className="font-mono font-medium hover:text-primary cursor-pointer inline-flex items-center gap-1 group/code"
            >
              /{link.code}
              {copiedCode === link.code ? (
                <Check className="w-3 h-3 text-green-500" />
              ) : (
                <Copy className="w-3 h-3 opacity-50 md:opacity-0 md:group-hover/code:opacity-50 transition-opacity" />
              )}
            </button>
          </TableCell>
        )}

        {/* Target URL 列 - tooltip 显示完整 URL */}
        {isVisible('target') && (
          <TableCell className="max-w-xs">
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="truncate block cursor-help text-muted-foreground">
                  {link.target}
                </span>
              </TooltipTrigger>
              <TooltipContent className="max-w-md break-all">
                {link.target}
              </TooltipContent>
            </Tooltip>
          </TableCell>
        )}

        {/* Clicks 列 */}
        {isVisible('clicks') && (
          <TableCell className="hidden md:table-cell">
            {link.click_count || 0}
          </TableCell>
        )}

        {/* Status 列 */}
        {isVisible('status') && (
          <TableCell>
            {isExpired(link.expires_at) ? (
              <Badge variant="destructive">{t('links.status.expired')}</Badge>
            ) : link.password ? (
              <Badge variant="secondary">{t('links.status.protected')}</Badge>
            ) : (
              <Badge variant="default">{t('links.status.active')}</Badge>
            )}
          </TableCell>
        )}

        {/* Created 列 - 小屏幕隐藏 */}
        {isVisible('created') && (
          <TableCell className="hidden lg:table-cell text-muted-foreground">
            {formatDateTime(link.created_at)}
          </TableCell>
        )}

        {/* Expires 列 - 小屏幕隐藏 */}
        {isVisible('expires') && (
          <TableCell className="hidden lg:table-cell text-muted-foreground">
            {link.expires_at
              ? formatDateTime(link.expires_at)
              : t('links.permanent')}
          </TableCell>
        )}

        {/* Actions 列 - 编辑按钮 + 更多下拉菜单 */}
        <TableCell className="text-right">
          <div className="flex items-center justify-end gap-1">
            {/* 编辑按钮 - 始终显示 */}
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8"
                  onClick={() => onEdit(link)}
                  aria-label={t('common.edit')}
                >
                  <Edit className="w-4 h-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>{t('common.edit')}</TooltipContent>
            </Tooltip>

            {/* 更多操作下拉菜单 */}
            <DropdownMenu>
              <Tooltip>
                <TooltipTrigger asChild>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      aria-label={t('common.more')}
                    >
                      <MoreHorizontal className="w-4 h-4" />
                    </Button>
                  </DropdownMenuTrigger>
                </TooltipTrigger>
                <TooltipContent>{t('common.more')}</TooltipContent>
              </Tooltip>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => onCopy(link.code)}>
                  <Copy className="w-4 h-4 mr-2" />
                  {t('common.copy')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() =>
                    window.open(buildShortUrl(link.code), '_blank')
                  }
                >
                  <ExternalLink className="w-4 h-4 mr-2" />
                  {t('links.openInNewTab')}
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => onViewAnalytics?.(link.code)}>
                  <BarChart3Icon className="w-4 h-4 mr-2" />
                  {t('analytics.viewAnalytics')}
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => onDownloadQr?.(link.code)}>
                  <Download className="w-4 h-4 mr-2" />
                  {t('links.qr.download')}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  className="text-destructive focus:text-destructive"
                  onClick={() => onDelete(link)}
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  {t('common.delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </TableCell>
      </TableRow>

      {/* 展开的详情行 */}
      {isExpanded && (
        <TableRow className="bg-muted/50 hover:bg-muted/50">
          <TableCell colSpan={columnCount} className="py-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              {/* 完整 URL */}
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  {t('links.table.targetUrl')}
                </div>
                <a
                  href={link.target}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline break-all"
                >
                  {link.target}
                </a>
              </div>

              {/* 短链接 */}
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  {t('links.expand.shortUrl')}
                </div>
                <a
                  href={buildShortUrl(link.code)}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline font-mono"
                >
                  {buildShortUrl(link.code)}
                </a>
              </div>

              {/* 创建时间 */}
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  {t('links.table.created')}
                </div>
                <div>{formatDateTime(link.created_at)}</div>
              </div>

              {/* 过期时间 */}
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  {t('links.table.expires')}
                </div>
                <div>
                  {link.expires_at ? (
                    <span
                      className={
                        isExpired(link.expires_at) ? 'text-destructive' : ''
                      }
                    >
                      {formatDateTime(link.expires_at)}
                    </span>
                  ) : (
                    t('links.permanent')
                  )}
                </div>
              </div>

              {/* 点击次数 */}
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  {t('links.table.clicks')}
                </div>
                <div>{link.click_count || 0}</div>
              </div>

              {/* 密码保护 */}
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  {t('links.expand.passwordProtection')}
                </div>
                <div className="flex items-center gap-1">
                  {link.password ? (
                    <>
                      <Lock className="w-4 h-4 text-amber-500" />
                      <span>{t('common.yes')}</span>
                    </>
                  ) : (
                    <span className="text-muted-foreground">
                      {t('common.no')}
                    </span>
                  )}
                </div>
              </div>
            </div>
          </TableCell>
        </TableRow>
      )}
    </>
  )
})

/**
 * 可排序表头组件
 */
const SortableHeader = memo(function SortableHeader({
  field,
  currentField,
  direction,
  onSort,
  children,
  className,
}: {
  field: SortField
  currentField: SortField
  direction: SortDirection
  onSort: (field: SortField) => void
  children: React.ReactNode
  className?: string
}) {
  const isActive = currentField === field

  return (
    <TableHead className={className}>
      <button
        type="button"
        className="flex items-center gap-1 hover:text-foreground transition-colors -ml-2 px-2 py-1 rounded"
        onClick={() => onSort(field)}
      >
        {children}
        {isActive ? (
          direction === 'asc' ? (
            <ArrowUp className="w-4 h-4" />
          ) : (
            <ArrowDown className="w-4 h-4" />
          )
        ) : (
          <ArrowUpDown className="w-4 h-4 opacity-50" />
        )}
      </button>
    </TableHead>
  )
})

/**
 * 链接表格组件
 */
export const LinksTable = memo(function LinksTable({
  links,
  copiedCode,
  selectedCodes,
  sortField,
  sortDirection,
  visibleColumns,
  onCopy,
  onEdit,
  onDelete,
  onSelectChange,
  onSelectAll,
  onSort,
  onViewAnalytics,
  onDownloadQr,
}: LinksTableProps) {
  const { t } = useTranslation()

  // 计算全选状态
  const isAllSelected = useMemo(
    () =>
      links.length > 0 && links.every((link) => selectedCodes.has(link.code)),
    [links, selectedCodes],
  )
  const isSomeSelected = useMemo(
    () => links.some((link) => selectedCodes.has(link.code)) && !isAllSelected,
    [links, selectedCodes, isAllSelected],
  )

  // Radix Checkbox 的 checked 值：true | false | 'indeterminate'
  const headerCheckState = isAllSelected
    ? true
    : isSomeSelected
      ? 'indeterminate'
      : false

  const isVisible = (col: ColumnKey) => visibleColumns.includes(col)

  // 计算总列数（用于展开行的 colSpan）
  const columnCount = useMemo(() => {
    // 1 (展开+checkbox) + visible columns + 1 (actions)
    return 1 + visibleColumns.length + 1
  }, [visibleColumns])

  return (
    <TooltipProvider>
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              {/* 展开按钮 + 全选 Checkbox */}
              <TableHead className="w-[70px]">
                <div className="flex items-center gap-1 pl-7">
                  <Checkbox
                    checked={headerCheckState}
                    onCheckedChange={(checked) => onSelectAll(!!checked)}
                    aria-label={t('links.batch.selectAll')}
                  />
                </div>
              </TableHead>
              {isVisible('code') && (
                <TableHead>{t('links.table.code')}</TableHead>
              )}
              {isVisible('target') && (
                <TableHead>{t('links.table.target')}</TableHead>
              )}
              {isVisible('clicks') && (
                <SortableHeader
                  field="clicks"
                  currentField={sortField}
                  direction={sortDirection}
                  onSort={onSort}
                  className="hidden md:table-cell"
                >
                  {t('links.table.clicks')}
                </SortableHeader>
              )}
              {isVisible('status') && (
                <TableHead>{t('links.table.status')}</TableHead>
              )}
              {isVisible('created') && (
                <SortableHeader
                  field="created_at"
                  currentField={sortField}
                  direction={sortDirection}
                  onSort={onSort}
                  className="hidden lg:table-cell"
                >
                  {t('links.table.created')}
                </SortableHeader>
              )}
              {isVisible('expires') && (
                <TableHead className="hidden lg:table-cell">
                  {t('links.table.expires')}
                </TableHead>
              )}
              <TableHead className="text-right">
                {t('links.table.actions')}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {links.map((link) => (
              <LinkRow
                key={link.code}
                link={link}
                copiedCode={copiedCode}
                isSelected={selectedCodes.has(link.code)}
                visibleColumns={visibleColumns}
                columnCount={columnCount}
                onCopy={onCopy}
                onEdit={onEdit}
                onDelete={onDelete}
                onSelectChange={onSelectChange}
                onViewAnalytics={onViewAnalytics}
                onDownloadQr={onDownloadQr}
              />
            ))}
          </TableBody>
        </Table>
      </div>
    </TooltipProvider>
  )
})
