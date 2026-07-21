import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { batchService } from '@/services/batchService'
import type { GetLinksQuery } from '@/services/types'
import { logger } from '@/utils/logger'

interface UseLinksBatchOptions {
  onBatchDeleteSuccess?: () => void
}

export function useLinksBatch(
  linkCodes: string[],
  options: UseLinksBatchOptions = {},
) {
  const { t } = useTranslation()
  const { onBatchDeleteSuccess } = options

  const [selectedCodes, setSelectedCodes] = useState<Set<string>>(new Set())
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false)
  const [batchDeleting, setBatchDeleting] = useState(false)
  const [exporting, setExporting] = useState(false)

  const selectedCount = selectedCodes.size

  const handleSelectChange = useCallback((code: string, checked: boolean) => {
    setSelectedCodes((prev) => {
      const next = new Set(prev)
      if (checked) {
        next.add(code)
      } else {
        next.delete(code)
      }
      return next
    })
  }, [])

  const handleSelectAll = useCallback(
    (checked: boolean) => {
      if (checked) {
        setSelectedCodes(new Set(linkCodes))
      } else {
        setSelectedCodes(new Set())
      }
    },
    [linkCodes],
  )

  const handleClearSelection = useCallback(() => {
    setSelectedCodes(new Set())
  }, [])

  const handleBatchDelete = useCallback(async () => {
    setBatchDeleting(true)
    try {
      const result = await batchService.deleteLinks(Array.from(selectedCodes))
      if (result.failed.length > 0) {
        toast.warning(
          t('links.batchDeletePartial', {
            success: result.success.length,
            failed: result.failed.length,
            defaultValue: `Deleted ${result.success.length} links, ${result.failed.length} failed`,
          }),
        )
      } else {
        toast.success(
          t('links.batchDeleteSuccess', {
            count: result.success.length,
            defaultValue: `Successfully deleted ${result.success.length} links`,
          }),
        )
      }
      setSelectedCodes(new Set())
      setBatchDeleteOpen(false)
      onBatchDeleteSuccess?.()
    } catch {
      toast.error(t('links.batchDeleteError', 'Failed to delete links'))
    } finally {
      setBatchDeleting(false)
    }
  }, [selectedCodes, onBatchDeleteSuccess, t])

  const handleExport = useCallback(
    async (query: Partial<GetLinksQuery>) => {
      setExporting(true)
      try {
        await batchService.exportLinks(query)
        toast.success(t('links.export.success'))
      } catch (error) {
        toast.error(t('links.export.error'))
        logger.error('Failed to export links:', error)
      } finally {
        setExporting(false)
      }
    },
    [t],
  )

  return {
    selectedCodes,
    selectedCount,
    batchDeleteOpen,
    batchDeleting,
    exporting,
    setBatchDeleteOpen,
    handleSelectChange,
    handleSelectAll,
    handleClearSelection,
    handleBatchDelete,
    handleExport,
  }
}
