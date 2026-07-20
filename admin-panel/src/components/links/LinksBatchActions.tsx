import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import { FiTrash2 as Trash2 } from 'react-icons/fi'
import { Button } from '@/components/ui/button'

interface LinksBatchActionsProps {
  selectedCount: number
  onDelete: () => void
  onClear: () => void
}

export const LinksBatchActions = memo(function LinksBatchActions({
  selectedCount,
  onDelete,
  onClear,
}: LinksBatchActionsProps) {
  const { t } = useTranslation()

  if (selectedCount === 0) return null

  return (
    <>
      <span className="text-sm text-muted-foreground">
        {t('links.batch.selected', { count: selectedCount })}
      </span>
      <Button
        variant="destructive"
        size="sm"
        onClick={onDelete}
        className="gap-1"
      >
        <Trash2 className="w-4 h-4" />
        {t('links.batch.delete')}
      </Button>
      <Button variant="ghost" size="sm" onClick={onClear}>
        {t('links.batch.clear')}
      </Button>
    </>
  )
})
