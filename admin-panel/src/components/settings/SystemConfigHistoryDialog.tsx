import { format } from 'date-fns'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FiArrowRight } from 'react-icons/fi'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Skeleton } from '@/components/ui/skeleton'
import type { ConfigHistoryResponse, ConfigItemResponse } from '@/services/api'
import { SystemConfigAPI } from '@/services/api'

interface SystemConfigHistoryDialogProps {
  config: ConfigItemResponse | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SystemConfigHistoryDialog({
  config,
  open,
  onOpenChange,
}: SystemConfigHistoryDialogProps) {
  const { t } = useTranslation()
  const [history, setHistory] = useState<ConfigHistoryResponse[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadHistory = useCallback(async (key: string) => {
    setLoading(true)
    setError(null)
    try {
      const data = await SystemConfigAPI.fetchHistory(key, 50)
      setHistory(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load history')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (open && config) {
      loadHistory(config.key)
    }
  }, [open, config, loadHistory])

  const formatDate = (dateStr: string) => {
    try {
      return format(new Date(dateStr), 'yyyy-MM-dd HH:mm:ss')
    } catch {
      return dateStr
    }
  }

  const truncateValue = (value: string | null, maxLength: number = 30) => {
    if (!value)
      return <span className="text-muted-foreground italic">null</span>
    if (value.length <= maxLength) return value
    return `${value.slice(0, maxLength)}...`
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>
            {t('config.historyTitle', 'Configuration History')}
          </DialogTitle>
          <DialogDescription>
            <code className="font-mono">{config?.key}</code>
          </DialogDescription>
        </DialogHeader>

        <div className="max-h-[400px] overflow-y-auto py-4">
          {loading ? (
            <div className="space-y-3">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex items-center gap-2">
                  <Skeleton className="h-4 w-32" />
                  <Skeleton className="h-4 w-20" />
                  <Skeleton className="h-4 w-20" />
                </div>
              ))}
            </div>
          ) : error ? (
            <div className="text-center py-8 text-destructive">{error}</div>
          ) : history.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              {t('config.noHistory', 'No history records')}
            </div>
          ) : (
            <div className="space-y-3">
              {history.map((item) => (
                <div
                  key={item.id}
                  className="flex items-start gap-3 rounded-md border p-3 text-sm"
                >
                  <div className="shrink-0 text-muted-foreground">
                    {formatDate(item.changed_at)}
                  </div>
                  <div className="flex flex-1 items-center gap-2 overflow-hidden">
                    <code className="rounded bg-red-50 px-1.5 py-0.5 text-red-700 dark:bg-red-950 dark:text-red-300">
                      {truncateValue(item.old_value)}
                    </code>
                    <FiArrowRight className="h-4 w-4 shrink-0 text-muted-foreground" />
                    <code className="rounded bg-green-50 px-1.5 py-0.5 text-green-700 dark:bg-green-950 dark:text-green-300">
                      {truncateValue(item.new_value)}
                    </code>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.close', 'Close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
