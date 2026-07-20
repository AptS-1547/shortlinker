import { zodResolver } from '@hookform/resolvers/zod'
import { format } from 'date-fns'
import { useEffect } from 'react'
import { Controller, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { FiCalendar as CalendarIcon, FiX as X } from 'react-icons/fi'
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import type { UseDialogReturn } from '@/hooks/useDialog'
import { cn } from '@/lib/utils'
import type { LinkFormData } from '@/schemas/linkSchema'
import { linkSchema } from '@/schemas/linkSchema'
import type { LinkResponse, PostNewLink } from '@/services/types'

// 生成小时选项 (00-23)
const HOURS = Array.from({ length: 24 }, (_, i) =>
  i.toString().padStart(2, '0'),
)
// 生成分钟选项 (00-59)
const MINUTES = Array.from({ length: 60 }, (_, i) =>
  i.toString().padStart(2, '0'),
)

interface LinkFormDialogProps {
  dialog: UseDialogReturn<LinkResponse>
  onSave: (data: PostNewLink) => Promise<void>
  isSaving: boolean
}

/**
 * 链接创建/编辑表单对话框
 */
export function LinkFormDialog({
  dialog,
  onSave,
  isSaving,
}: LinkFormDialogProps) {
  const { t } = useTranslation()

  const {
    control,
    handleSubmit: createSubmitHandler,
    formState: { errors, isSubmitting },
    reset,
    watch,
    setValue,
  } = useForm<LinkFormData>({
    resolver: zodResolver(linkSchema),
    mode: 'onBlur',
    defaultValues: {
      code: dialog.data?.code || '',
      target: dialog.data?.target || '',
      password: null,
      expires_at: dialog.data?.expires_at || null,
    },
  })

  // 对话框打开时重置表单
  useEffect(() => {
    if (dialog.isOpen) {
      reset({
        code: dialog.data?.code || '',
        target: dialog.data?.target || '',
        password: null,
        expires_at: dialog.data?.expires_at || null,
      })
    }
  }, [dialog.isOpen, dialog.data, reset])

  // 监听 expires_at 变化
  const expiresAt = watch('expires_at')

  const handleSubmit = createSubmitHandler(async (data) => {
    // 直接将验证后的数据传递给 onSave
    await onSave(data as PostNewLink)
  })

  return (
    <Dialog open={dialog.isOpen} onOpenChange={dialog.setIsOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {dialog.isEditMode ? t('links.editLink') : t('links.createLink')}
          </DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            {/* Code 字段 */}
            <div className="space-y-2">
              <Label htmlFor="code">{t('links.form.code')}</Label>
              <Controller
                name="code"
                control={control}
                render={({ field }) => (
                  <Input
                    {...field}
                    id="code"
                    value={field.value || ''}
                    placeholder={t('links.form.codePlaceholder')}
                    disabled={dialog.isEditMode}
                  />
                )}
              />
              {errors.code && (
                <p className="text-sm text-destructive">
                  {errors.code.message}
                </p>
              )}
            </div>

            {/* Target 字段 */}
            <div className="space-y-2">
              <Label htmlFor="target">{t('links.form.target')}</Label>
              <Controller
                name="target"
                control={control}
                render={({ field }) => (
                  <Input
                    {...field}
                    id="target"
                    placeholder={t('links.form.targetPlaceholder')}
                    required
                  />
                )}
              />
              {errors.target && (
                <p className="text-sm text-destructive">
                  {errors.target.message}
                </p>
              )}
            </div>

            {/* Password 字段 */}
            <div className="space-y-2">
              <Label htmlFor="password">{t('links.form.password')}</Label>
              <Controller
                name="password"
                control={control}
                render={({ field }) => (
                  <Input
                    {...field}
                    id="password"
                    type="password"
                    value={field.value || ''}
                    onChange={(e) => field.onChange(e.target.value || null)}
                    placeholder={t('links.form.passwordPlaceholder')}
                  />
                )}
              />
            </div>

            {/* Expires At 字段 */}
            <div className="space-y-2">
              <Label>{t('links.expiresAtOptional')}</Label>
              <div className="flex items-center gap-2">
                {/* 日期选择 */}
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      type="button"
                      variant="outline"
                      className={cn(
                        'flex-1 justify-start text-left font-normal',
                        !expiresAt && 'text-muted-foreground',
                      )}
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {expiresAt
                        ? format(new Date(expiresAt), 'yyyy-MM-dd')
                        : t('links.permanent')}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0" align="start">
                    <Calendar
                      mode="single"
                      selected={expiresAt ? new Date(expiresAt) : undefined}
                      onSelect={(date) => {
                        if (date) {
                          // 保留现有时间，如果没有则默认 23:59
                          const existingDate = expiresAt
                            ? new Date(expiresAt)
                            : null
                          const hours = existingDate?.getHours() ?? 23
                          const minutes = existingDate?.getMinutes() ?? 59
                          date.setHours(hours, minutes, 0, 0)
                          setValue('expires_at', date.toISOString())
                        }
                      }}
                      disabled={(date) => {
                        const today = new Date()
                        today.setHours(0, 0, 0, 0)
                        return date < today
                      }}
                    />
                  </PopoverContent>
                </Popover>

                {/* 时间选择 */}
                {expiresAt && (
                  <>
                    <Select
                      value={format(new Date(expiresAt), 'HH')}
                      onValueChange={(hour) => {
                        const date = new Date(expiresAt)
                        date.setHours(parseInt(hour, 10))
                        setValue('expires_at', date.toISOString())
                      }}
                    >
                      <SelectTrigger className="w-[70px]">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {HOURS.map((h) => (
                          <SelectItem key={h} value={h}>
                            {h}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <span className="text-muted-foreground">:</span>
                    <Select
                      value={format(new Date(expiresAt), 'mm')}
                      onValueChange={(minute) => {
                        const date = new Date(expiresAt)
                        date.setMinutes(parseInt(minute, 10))
                        setValue('expires_at', date.toISOString())
                      }}
                    >
                      <SelectTrigger className="w-[70px]">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {MINUTES.map((m) => (
                          <SelectItem key={m} value={m}>
                            {m}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </>
                )}

                {expiresAt && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    onClick={() => setValue('expires_at', null)}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                )}
              </div>
              <p className="text-xs text-muted-foreground">
                {t('links.expiresAtHelp')}
              </p>
              {errors.expires_at && (
                <p className="text-sm text-destructive">
                  {errors.expires_at.message}
                </p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={dialog.close}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={isSubmitting || isSaving}>
              {isSubmitting || isSaving ? t('common.saving') : t('common.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
