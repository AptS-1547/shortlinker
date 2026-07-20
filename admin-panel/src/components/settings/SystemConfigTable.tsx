import { useTranslation } from 'react-i18next'
import { FiAlertTriangle, FiClock, FiEdit2 } from 'react-icons/fi'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
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
import type { ConfigItemResponse } from '@/services/api'
import { getConfigKeyLabel } from '@/utils/configUtils'

interface SystemConfigTableProps {
  configs: ConfigItemResponse[]
  onEdit: (config: ConfigItemResponse) => void
  onHistory: (config: ConfigItemResponse) => void
}

/** 类型 Badge 颜色 */
function getTypeBadgeVariant(type: string): {
  variant: 'default' | 'secondary' | 'outline'
  className?: string
} {
  switch (type) {
    case 'bool':
      return { variant: 'secondary' }
    case 'int':
      return { variant: 'outline' }
    case 'enum':
      return {
        variant: 'default',
        className: 'bg-violet-500 dark:bg-violet-600',
      }
    default:
      return { variant: 'default' }
  }
}

export function SystemConfigTable({
  configs,
  onEdit,
  onHistory,
}: SystemConfigTableProps) {
  const { t } = useTranslation()

  if (configs.length === 0) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground">
        {t('config.noConfigs', 'No configurations found')}
      </div>
    )
  }

  return (
    <TooltipProvider>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[280px]">
              {t('config.key', 'Key')}
            </TableHead>
            <TableHead>{t('config.value', 'Value')}</TableHead>
            <TableHead className="w-[80px]">
              {t('config.type', 'Type')}
            </TableHead>
            <TableHead className="w-[100px]">
              {t('config.requiresRestart', 'Restart')}
            </TableHead>
            <TableHead className="w-[120px] text-right">
              {t('common.actions', 'Actions')}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {configs.map((config) => (
            <TableRow
              key={config.key}
              className={
                config.requires_restart
                  ? 'bg-yellow-50/50 dark:bg-yellow-950/20'
                  : ''
              }
            >
              <TableCell className="font-mono text-sm">
                <span className="font-medium">
                  {getConfigKeyLabel(config.key, t)}
                </span>
              </TableCell>
              <TableCell className="font-mono text-sm max-w-[300px] truncate">
                {config.value.length > 0 ? (
                  config.value
                ) : (
                  <span className="italic text-muted-foreground">
                    {t('config.emptyValue', '(not set)')}
                  </span>
                )}
              </TableCell>
              <TableCell>
                {(() => {
                  const { variant, className } = getTypeBadgeVariant(
                    config.value_type,
                  )
                  return (
                    <Badge variant={variant} className={className}>
                      {config.value_type}
                    </Badge>
                  )
                })()}
              </TableCell>
              <TableCell>
                {config.requires_restart ? (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="inline-flex items-center gap-1 text-yellow-600 dark:text-yellow-500">
                        <FiAlertTriangle className="h-4 w-4" />
                        {t('common.yes', 'Yes')}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>
                        {t(
                          'config.requiresRestartHint',
                          'Changes require server restart to take effect',
                        )}
                      </p>
                    </TooltipContent>
                  </Tooltip>
                ) : (
                  <span className="text-muted-foreground">
                    {t('common.no', 'No')}
                  </span>
                )}
              </TableCell>
              <TableCell className="text-right">
                <div className="flex items-center justify-end gap-1">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onEdit(config)}
                      >
                        <FiEdit2 className="h-4 w-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{t('config.edit', 'Edit')}</p>
                    </TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onHistory(config)}
                      >
                        <FiClock className="h-4 w-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{t('config.history', 'History')}</p>
                    </TooltipContent>
                  </Tooltip>
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TooltipProvider>
  )
}
