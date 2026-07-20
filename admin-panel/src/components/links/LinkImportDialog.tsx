import { useCallback, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiAlertCircle as AlertCircle,
  FiCheckCircle as CheckCircle,
  FiUpload as Upload,
  FiX as X,
} from 'react-icons/fi'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { cn } from '@/lib/utils'
import { batchService } from '@/services/batchService'
import type { ImportMode, ImportResponse } from '@/services/types'

// 与后端 MAX_IMPORT_FILE_SIZE 保持一致 (src/api/services/admin/export_import.rs)
const MAX_FILE_SIZE = 10 * 1024 * 1024 // 10MB

interface LinkImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}

type ImportState = 'idle' | 'uploading' | 'success' | 'error'

export function LinkImportDialog({
  open,
  onOpenChange,
  onSuccess,
}: LinkImportDialogProps) {
  const { t } = useTranslation()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const [file, setFile] = useState<File | null>(null)
  const [mode, setMode] = useState<ImportMode>('skip')
  const [state, setState] = useState<ImportState>('idle')
  const [result, setResult] = useState<ImportResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isDragging, setIsDragging] = useState(false)
  const [uploadProgress, setUploadProgress] = useState(0)

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const selectedFile = e.target.files?.[0]
      if (selectedFile) {
        if (selectedFile.size > MAX_FILE_SIZE) {
          setError(
            t('links.import.fileTooLarge', {
              maxSize: `${(MAX_FILE_SIZE / 1024 / 1024).toFixed(1)}MB`,
            }),
          )
          return
        }
        setFile(selectedFile)
        setState('idle')
        setResult(null)
        setError(null)
      }
    },
    [t],
  )

  const handleImport = useCallback(async () => {
    if (!file) return

    setState('uploading')
    setUploadProgress(0)
    setError(null)

    try {
      const response = await batchService.importLinks(file, mode, (percent) => {
        setUploadProgress(percent)
      })
      setResult(response)
      setState('success')
      onSuccess()
    } catch (err) {
      setError(err instanceof Error ? err.message : t('links.import.error'))
      setState('error')
    }
  }, [file, mode, onSuccess, t])

  const handleClose = useCallback(() => {
    setFile(null)
    setMode('skip')
    setState('idle')
    setResult(null)
    setError(null)
    setUploadProgress(0)
    onOpenChange(false)
  }, [onOpenChange])

  const handleSelectFile = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
  }, [])

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault()
      e.stopPropagation()
      setIsDragging(false)

      const droppedFile = e.dataTransfer.files?.[0]
      if (droppedFile?.name.endsWith('.csv')) {
        if (droppedFile.size > MAX_FILE_SIZE) {
          setError(
            t('links.import.fileTooLarge', {
              maxSize: `${(MAX_FILE_SIZE / 1024 / 1024).toFixed(1)}MB`,
            }),
          )
          return
        }
        setFile(droppedFile)
        setState('idle')
        setResult(null)
        setError(null)
      }
    },
    [t],
  )

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('links.import.title')}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* 文件选择 */}
          <div className="space-y-2">
            <Label>{t('links.import.file')}</Label>
            <input
              ref={fileInputRef}
              type="file"
              accept=".csv"
              onChange={handleFileChange}
              className="hidden"
            />
            <button
              type="button"
              onClick={handleSelectFile}
              onDragOver={handleDragOver}
              onDragEnter={handleDragEnter}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
              className={cn(
                'flex items-center justify-center w-full h-24 border-2 border-dashed rounded-lg cursor-pointer transition-colors',
                isDragging
                  ? 'border-primary bg-primary/10'
                  : 'hover:bg-muted/50',
              )}
            >
              {file ? (
                <div className="flex items-center gap-2 text-sm">
                  <CheckCircle className="h-4 w-4 text-green-500" />
                  <span className="truncate max-w-[200px]">{file.name}</span>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6"
                    onClick={(e) => {
                      e.stopPropagation()
                      setFile(null)
                      setState('idle')
                      setResult(null)
                    }}
                  >
                    <X className="h-3 w-3" />
                  </Button>
                </div>
              ) : (
                <div className="flex flex-col items-center gap-1 text-muted-foreground">
                  <Upload className="h-6 w-6" />
                  <span className="text-sm">
                    {t('links.import.dragOrClick')}
                  </span>
                  <span className="text-xs text-muted-foreground/70">
                    {t('links.import.maxFileSize', { size: '10MB' })}
                  </span>
                </div>
              )}
            </button>
          </div>

          {/* 导入模式 */}
          <div className="space-y-2">
            <Label>{t('links.import.mode')}</Label>
            <RadioGroup
              value={mode}
              onValueChange={(value: string) => setMode(value as ImportMode)}
              className="space-y-2"
            >
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="skip" id="mode-skip" />
                <Label
                  htmlFor="mode-skip"
                  className="font-normal cursor-pointer"
                >
                  {t('links.import.modeSkip')}
                </Label>
              </div>
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="overwrite" id="mode-overwrite" />
                <Label
                  htmlFor="mode-overwrite"
                  className="font-normal cursor-pointer"
                >
                  {t('links.import.modeOverwrite')}
                </Label>
              </div>
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="error" id="mode-error" />
                <Label
                  htmlFor="mode-error"
                  className="font-normal cursor-pointer"
                >
                  {t('links.import.modeError')}
                </Label>
              </div>
            </RadioGroup>
          </div>

          {/* 导入结果 */}
          {state === 'success' && result && (
            <div className="p-3 rounded-lg bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800">
              <div className="flex items-center gap-2 text-green-700 dark:text-green-300 mb-2">
                <CheckCircle className="h-4 w-4" />
                <span className="font-medium">{t('links.import.success')}</span>
              </div>
              <div className="text-sm text-green-600 dark:text-green-400 space-y-1">
                <p>
                  {t('links.import.totalRows', { count: result.total_rows })}
                </p>
                <p>
                  {t('links.import.successCount', {
                    count: result.success_count,
                  })}
                </p>
                {result.skipped_count > 0 && (
                  <p>
                    {t('links.import.skippedCount', {
                      count: result.skipped_count,
                    })}
                  </p>
                )}
                {result.failed_count > 0 && (
                  <p className="text-amber-600 dark:text-amber-400">
                    {t('links.import.failedCount', {
                      count: result.failed_count,
                    })}
                  </p>
                )}
              </div>
              {result.failed_items.length > 0 && (
                <div className="mt-2 max-h-32 overflow-auto">
                  <p className="text-xs text-muted-foreground mb-1">
                    {t('links.import.failedDetails')}
                  </p>
                  {result.failed_items.slice(0, 5).map((item, index) => (
                    <p
                      key={item.row ?? `unknown-${index}`}
                      className="text-xs text-red-500"
                    >
                      {t('links.import.failedItem', {
                        row: item.row ?? '?',
                        code: item.code,
                        error: item.error,
                      })}
                    </p>
                  ))}
                  {result.failed_items.length > 5 && (
                    <p className="text-xs text-muted-foreground">
                      {t('links.import.andMore', {
                        count: result.failed_items.length - 5,
                      })}
                    </p>
                  )}
                </div>
              )}
            </div>
          )}

          {/* 错误信息 */}
          {error && (
            <div className="p-3 rounded-lg bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800">
              <div className="flex items-center gap-2 text-red-700 dark:text-red-300">
                <AlertCircle className="h-4 w-4" />
                <span>{error}</span>
              </div>
            </div>
          )}

          {/* 上传进度 */}
          {state === 'uploading' && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm text-muted-foreground">
                <span>{t('links.import.uploading')}</span>
                <span>{uploadProgress}%</span>
              </div>
              <div className="h-2 bg-muted rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary transition-all duration-200"
                  style={{ width: `${uploadProgress}%` }}
                />
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={handleClose}>
            {state === 'success' ? t('common.close') : t('common.cancel')}
          </Button>
          {state !== 'success' && (
            <Button
              type="button"
              onClick={handleImport}
              disabled={!file || state === 'uploading'}
            >
              {state === 'uploading'
                ? t('links.import.importing')
                : t('links.import.import')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
