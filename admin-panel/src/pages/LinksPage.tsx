import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiDownload as Download,
  FiPlus as Plus,
  FiUpload as Upload,
} from 'react-icons/fi'
import { toast } from 'sonner'
import PageHeader from '@/components/layout/PageHeader'
import {
  ColumnConfigDropdown,
  LinkDeleteDialog,
  LinkFormDialog,
  LinkImportDialog,
  LinksBatchActions,
  LinksFilterBar,
  LinksTable,
  QrCodeDialog,
} from '@/components/links'
import { LinkAnalyticsSheet } from '@/components/links/LinkAnalyticsSheet'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@/components/ui/pagination'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { useColumnVisibility } from '@/hooks/useColumnVisibility'
import { useCopyToClipboard } from '@/hooks/useCopyToClipboard'
import { useDialog } from '@/hooks/useDialog'
import { useLinksBatch } from '@/hooks/useLinksBatch'
import { useLinksFilters } from '@/hooks/useLinksFilters'
import { useLinksSort } from '@/hooks/useLinksSort'
import { batchService } from '@/services/batchService'
import type { LinkResponse, PostNewLink } from '@/services/types'
import { useLinksStore } from '@/stores/linksStore'
import { logger } from '@/utils/logger'
import { buildShortUrl } from '@/utils/urlBuilder'

export default function LinksPage() {
  const { t } = useTranslation()
  const { copy } = useCopyToClipboard()

  // Store
  const {
    links,
    fetching,
    creating,
    updating,
    deleting,
    pagination,
    fetchLinks,
    createLink,
    updateLink,
    deleteLink,
    goToPage,
    setPageSize,
  } = useLinksStore()

  // Hooks
  const filters = useLinksFilters({ onFilterChange: fetchLinks })
  const { sortField, sortDirection, sortedLinks, handleSort } =
    useLinksSort(links)
  const { visibleColumns, handleColumnToggle } = useColumnVisibility()
  const batch = useLinksBatch(
    links.map((l) => l.code),
    { onBatchDeleteSuccess: fetchLinks },
  )

  // 过滤条件变化时清除批量选择
  const prevFilters = useRef({
    searchQuery: filters.searchQuery,
    statusFilter: filters.statusFilter,
    createdAfter: filters.createdAfter,
    createdBefore: filters.createdBefore,
  })

  useEffect(() => {
    const filtersChanged =
      prevFilters.current.searchQuery !== filters.searchQuery ||
      prevFilters.current.statusFilter !== filters.statusFilter ||
      prevFilters.current.createdAfter !== filters.createdAfter ||
      prevFilters.current.createdBefore !== filters.createdBefore

    if (filtersChanged) {
      batch.handleClearSelection()
      prevFilters.current = {
        searchQuery: filters.searchQuery,
        statusFilter: filters.statusFilter,
        createdAfter: filters.createdAfter,
        createdBefore: filters.createdBefore,
      }
    }
  }, [
    filters.searchQuery,
    filters.statusFilter,
    filters.createdAfter,
    filters.createdBefore,
    batch,
  ])

  // 分页/页面大小变更时清除批量选择的包装函数
  const handlePageChange = useCallback(
    (page: number) => {
      batch.handleClearSelection()
      goToPage(page)
    },
    [batch, goToPage],
  )

  const handlePageSizeChange = useCallback(
    (size: number) => {
      batch.handleClearSelection()
      setPageSize(size)
    },
    [batch, setPageSize],
  )

  // 导入成功时清除批量选择并刷新
  const handleImportSuccess = useCallback(() => {
    batch.handleClearSelection()
    fetchLinks()
  }, [batch, fetchLinks])

  // Dialog state
  const formDialog = useDialog<LinkResponse>()
  const deleteDialog = useDialog<LinkResponse>()
  const [copiedCode, setCopiedCode] = useState<string | null>(null)
  const [importOpen, setImportOpen] = useState(false)
  const [exporting, setExporting] = useState(false)
  const [analyticsCode, setAnalyticsCode] = useState<string | null>(null)
  const [qrCodeDialogCode, setQrCodeDialogCode] = useState<string | null>(null)

  // Handlers
  const handleOpenCreate = useCallback(() => {
    formDialog.open()
  }, [formDialog])

  const handleOpenEdit = useCallback(
    (link: LinkResponse) => {
      formDialog.open(link)
    },
    [formDialog],
  )

  const handleSave = useCallback(
    async (data: PostNewLink) => {
      try {
        if (formDialog.isEditMode && formDialog.data) {
          await updateLink(formDialog.data.code, data)
          toast.success(t('links.updateSuccess', 'Link updated successfully'))
        } else {
          await createLink(data)
          toast.success(t('links.createSuccess', 'Link created successfully'))
        }
        formDialog.close()
      } catch {
        toast.error(
          formDialog.isEditMode
            ? t('links.updateError', 'Failed to update link')
            : t('links.createError', 'Failed to create link'),
        )
      }
    },
    [formDialog, updateLink, createLink, t],
  )

  const handleDelete = useCallback(async () => {
    if (deleteDialog.data) {
      try {
        await deleteLink(deleteDialog.data.code)
        toast.success(t('links.deleteSuccess', 'Link deleted successfully'))
        batch.handleClearSelection()
        deleteDialog.close()
      } catch {
        toast.error(t('links.deleteError', 'Failed to delete link'))
      }
    }
  }, [deleteDialog, deleteLink, t, batch])

  const handleCopy = useCallback(
    async (code: string) => {
      const url = buildShortUrl(code)
      const success = await copy(url)
      if (success) {
        setCopiedCode(code)
        setTimeout(() => setCopiedCode(null), 2000)
        toast.success(t('links.copySuccess', 'Link copied to clipboard'))
      } else {
        toast.error(t('links.copyError', 'Failed to copy link'))
      }
    },
    [copy, t],
  )

  const handleOpenQrCode = useCallback((code: string) => {
    setQrCodeDialogCode(code)
  }, [])

  const handleExport = useCallback(async () => {
    setExporting(true)
    try {
      // 构建当前过滤条件
      const query = {
        search: filters.searchQuery || undefined,
        created_after: filters.createdAfter?.toISOString(),
        created_before: filters.createdBefore?.toISOString(),
        only_active: filters.statusFilter === 'active' || undefined,
        only_expired: filters.statusFilter === 'expired' || undefined,
      }

      // 触发下载（预检 + 浏览器原生下载）
      await batchService.exportLinks(query)

      // 立即成功提示（浏览器已开始下载）
      toast.success(t('links.export.success'))
    } catch (error) {
      // 预检失败时的错误处理（401/500等）
      toast.error(t('links.export.error'))
      logger.error('Failed to export links:', error)
    } finally {
      setExporting(false)
    }
  }, [filters, t])

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('links.title')}
        description={t('links.description')}
        actions={
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              onClick={handleExport}
              disabled={exporting}
              className="gap-2"
            >
              <Download className="w-4 h-4" />
              {exporting
                ? t('links.export.exporting')
                : t('links.export.button')}
            </Button>
            <Button
              variant="outline"
              onClick={() => setImportOpen(true)}
              className="gap-2"
            >
              <Upload className="w-4 h-4" />
              {t('links.import.button')}
            </Button>
            <Button onClick={handleOpenCreate} className="gap-2">
              <Plus className="w-4 h-4" />
              {t('links.createNew')}
            </Button>
          </div>
        }
      />

      {/* Filters */}
      <LinksFilterBar
        searchQuery={filters.searchQuery}
        statusFilter={filters.statusFilter}
        createdAfter={filters.createdAfter}
        createdBefore={filters.createdBefore}
        onSearchChange={filters.setSearchQuery}
        onStatusChange={filters.setStatusFilter}
        onCreatedAfterChange={filters.setCreatedAfter}
        onCreatedBeforeChange={filters.setCreatedBefore}
      />

      {/* Links Table Card */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-base font-medium">
              {t('links.tableTitle')} ({pagination.total})
            </CardTitle>
            <div className="flex items-center gap-2">
              {batch.selectedCount > 0 ? (
                <LinksBatchActions
                  selectedCount={batch.selectedCount}
                  onDelete={() => batch.setBatchDeleteOpen(true)}
                  onClear={batch.handleClearSelection}
                />
              ) : (
                <ColumnConfigDropdown
                  visibleColumns={visibleColumns}
                  onColumnToggle={handleColumnToggle}
                />
              )}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {fetching ? (
            <TableSkeleton />
          ) : links.length > 0 ? (
            <>
              <LinksTable
                links={sortedLinks}
                copiedCode={copiedCode}
                selectedCodes={batch.selectedCodes}
                sortField={sortField}
                sortDirection={sortDirection}
                visibleColumns={visibleColumns}
                onCopy={handleCopy}
                onEdit={handleOpenEdit}
                onDelete={deleteDialog.open}
                onSelectChange={batch.handleSelectChange}
                onSelectAll={batch.handleSelectAll}
                onSort={handleSort}
                onViewAnalytics={setAnalyticsCode}
                onDownloadQr={handleOpenQrCode}
              />
              <PaginationControls
                pagination={pagination}
                onPageChange={handlePageChange}
                onPageSizeChange={handlePageSizeChange}
              />
            </>
          ) : (
            <EmptyState onCreateClick={handleOpenCreate} />
          )}
        </CardContent>
      </Card>

      {/* Dialogs */}
      <LinkFormDialog
        dialog={formDialog}
        onSave={handleSave}
        isSaving={creating || updating}
      />

      <LinkDeleteDialog
        dialog={deleteDialog}
        onConfirm={handleDelete}
        isDeleting={deleting}
      />

      {/* 批量删除确认对话框 */}
      <AlertDialog
        open={batch.batchDeleteOpen}
        onOpenChange={batch.setBatchDeleteOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('links.batch.deleteTitle')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('links.batch.deleteDescription', {
                count: batch.selectedCount,
              })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={batch.batchDeleting}>
              {t('common.cancel')}
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={batch.handleBatchDelete}
              disabled={batch.batchDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {batch.batchDeleting ? t('common.deleting') : t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 导入对话框 */}
      <LinkImportDialog
        open={importOpen}
        onOpenChange={setImportOpen}
        onSuccess={handleImportSuccess}
      />

      {/* 链接分析侧边栏 */}
      <LinkAnalyticsSheet
        code={analyticsCode}
        onClose={() => setAnalyticsCode(null)}
      />

      {/* QR Code 预览弹窗 */}
      <QrCodeDialog
        code={qrCodeDialogCode}
        onClose={() => setQrCodeDialogCode(null)}
      />
    </div>
  )
}

// 子组件
function TableSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: static skeleton list, order never changes
        <div key={`skeleton-${i}`} className="flex items-center gap-4">
          <Skeleton className="h-5 w-5" />
          <Skeleton className="h-5 w-24" />
          <Skeleton className="h-5 flex-1" />
          <Skeleton className="h-5 w-32" />
          <Skeleton className="h-5 w-20" />
          <Skeleton className="h-8 w-24" />
        </div>
      ))}
    </div>
  )
}

function EmptyState({ onCreateClick }: { onCreateClick: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col items-center justify-center py-12 text-center">
      <div className="rounded-full bg-muted p-3 mb-4">
        <Plus className="h-8 w-8 text-muted-foreground" />
      </div>
      <h3 className="text-lg font-medium mb-2">
        {t('links.noLinksTitle', 'No links yet')}
      </h3>
      <p className="text-sm text-muted-foreground mb-4 max-w-sm">
        {t(
          'links.noLinksDescription',
          'Get started by creating your first short link',
        )}
      </p>
      <Button onClick={onCreateClick}>
        <Plus className="mr-2 h-4 w-4" />
        {t('links.createLink', 'Create Link')}
      </Button>
    </div>
  )
}

interface PaginationControlsProps {
  pagination: {
    page: number
    pageSize: number
    total: number
    totalPages: number
    hasPrev: boolean
    hasNext: boolean
  }
  onPageChange: (page: number) => void
  onPageSizeChange: (size: number) => void
}

function PaginationControls({
  pagination,
  onPageChange,
  onPageSizeChange,
}: PaginationControlsProps) {
  const { t } = useTranslation()

  return (
    <div className="mt-4 flex items-center justify-between">
      {/* Page Size Selector */}
      <div className="flex items-center gap-2 shrink-0">
        <span className="text-sm text-muted-foreground whitespace-nowrap">
          {t('links.pageSize')}
        </span>
        <Select
          value={String(pagination.pageSize)}
          onValueChange={(value) => onPageSizeChange(Number(value))}
        >
          <SelectTrigger className="w-[70px] h-8">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="10">10</SelectItem>
            <SelectItem value="20">20</SelectItem>
            <SelectItem value="50">50</SelectItem>
            <SelectItem value="100">100</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Page Navigation */}
      {pagination.totalPages > 1 && (
        <Pagination>
          <PaginationContent>
            <PaginationItem>
              <PaginationPrevious
                onClick={() =>
                  pagination.hasPrev && onPageChange(pagination.page - 1)
                }
                className={
                  !pagination.hasPrev
                    ? 'pointer-events-none opacity-50'
                    : 'cursor-pointer'
                }
              />
            </PaginationItem>
            {Array.from({ length: pagination.totalPages }, (_, i) => i + 1)
              .filter((page) => Math.abs(page - pagination.page) <= 2)
              .map((page) => (
                <PaginationItem key={page}>
                  <PaginationLink
                    onClick={() => onPageChange(page)}
                    isActive={page === pagination.page}
                    className="cursor-pointer"
                  >
                    {page}
                  </PaginationLink>
                </PaginationItem>
              ))}
            <PaginationItem>
              <PaginationNext
                onClick={() =>
                  pagination.hasNext && onPageChange(pagination.page + 1)
                }
                className={
                  !pagination.hasNext
                    ? 'pointer-events-none opacity-50'
                    : 'cursor-pointer'
                }
              />
            </PaginationItem>
          </PaginationContent>
        </Pagination>
      )}
    </div>
  )
}
