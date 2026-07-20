import { useTranslation } from 'react-i18next'
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
import type { UseDialogReturn } from '@/hooks/useDialog'
import type { LinkResponse } from '@/services/types'

interface LinkDeleteDialogProps {
  dialog: UseDialogReturn<LinkResponse>
  onConfirm: () => Promise<void>
  isDeleting: boolean
}

/**
 * 链接删除确认对话框
 */
export function LinkDeleteDialog({
  dialog,
  onConfirm,
  isDeleting,
}: LinkDeleteDialogProps) {
  const { t } = useTranslation()

  return (
    <AlertDialog open={dialog.isOpen} onOpenChange={dialog.setIsOpen}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('links.deleteConfirm.title')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('links.deleteConfirm.description', {
              code: dialog.data?.code,
            })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} disabled={isDeleting}>
            {isDeleting ? t('common.deleting') : t('common.delete')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
