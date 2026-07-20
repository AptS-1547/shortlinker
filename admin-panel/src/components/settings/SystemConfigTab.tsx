import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FiRefreshCw } from 'react-icons/fi'
import { toast } from 'sonner'

import {
  SystemConfigEditDialog,
  SystemConfigHistoryDialog,
  SystemConfigTable,
} from '@/components/settings'
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useConfigSchema } from '@/hooks/useConfigSchema'
import { cn } from '@/lib/utils'
import type { ConfigItemResponse } from '@/services/api'
import { useSystemConfigStore } from '@/stores/systemConfigStore'
import {
  CONFIG_CATEGORY_INFO,
  groupConfigsByCategory,
} from '@/utils/configUtils'

// 组件 Props
interface SystemConfigTabProps {
  isActive?: boolean
}

export function SystemConfigTab({ isActive = true }: SystemConfigTabProps) {
  const { t } = useTranslation()
  const {
    configs,
    fetching,
    updating,
    reloading,
    fetchConfigs,
    updateConfig,
    reloadConfigs,
  } = useSystemConfigStore()

  // 获取 schema 数据
  const { data: schemas } = useConfigSchema()

  // 对话框状态
  const [editingConfig, setEditingConfig] = useState<ConfigItemResponse | null>(
    null,
  )
  const [historyConfig, setHistoryConfig] = useState<ConfigItemResponse | null>(
    null,
  )

  useEffect(() => {
    if (!isActive) return // 不激活时不请求
    const controller = new AbortController()
    fetchConfigs(controller.signal).catch(() => {
      // Error is handled by the store
    })
    return () => controller.abort()
  }, [isActive, fetchConfigs])

  // 按分组整理配置
  const groupedConfigs = useMemo(() => {
    return groupConfigsByCategory(configs, schemas || [])
  }, [configs, schemas])

  // 分组排序（按预定义顺序）
  const categoryOrder = [
    'auth',
    'cookie',
    'features',
    'routes',
    'cors',
    'tracking',
    'analytics',
    'cache',
    'other',
  ]
  const sortedCategories = useMemo(() => {
    return Object.keys(groupedConfigs).sort(
      (a, b) => categoryOrder.indexOf(a) - categoryOrder.indexOf(b),
    )
  }, [groupedConfigs])

  const handleReload = async () => {
    try {
      await reloadConfigs()
      toast.success(
        t('config.reloadSuccess', 'Configuration reloaded successfully'),
      )
    } catch {
      toast.error(t('config.reloadError', 'Failed to reload configuration'))
    }
  }

  const handleSave = async (key: string, value: string) => {
    const result = await updateConfig(key, value)
    if (result.requires_restart) {
      toast.warning(
        result.message ||
          t(
            'config.savedNeedRestart',
            'Configuration saved. Please restart the server for changes to take effect.',
          ),
      )
    } else {
      toast.success(
        t('config.updateSuccess', 'Configuration updated successfully'),
      )
    }
    return result
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">
            {t('config.title', 'System Configuration')}
          </h3>
          <p className="text-sm text-muted-foreground">
            {t('config.description', 'Manage runtime configuration settings')}
          </p>
        </div>
        <Button variant="outline" onClick={handleReload} disabled={reloading}>
          <FiRefreshCw
            className={`mr-2 h-4 w-4 ${reloading ? 'animate-spin' : ''}`}
          />
          {t('config.reload', 'Reload')}
        </Button>
      </div>

      {/* 配置列表 - 按分组展示 */}
      {fetching ? (
        <div className="space-y-2">
          {[1, 2, 3, 4, 5].map((i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      ) : (
        <Accordion
          type="multiple"
          defaultValue={sortedCategories}
          className="w-full space-y-2 pb-2"
        >
          {sortedCategories.map((category, index) => {
            const info = CONFIG_CATEGORY_INFO[category] || {
              label: category,
              i18nKey: `config.category.${category}`,
            }
            const categoryConfigs = groupedConfigs[category]
            const isLast = index === sortedCategories.length - 1

            return (
              <AccordionItem
                key={category}
                value={category}
                className={cn(
                  'rounded-lg border bg-card',
                  isLast && '!border-b',
                )}
              >
                <AccordionTrigger className="px-4 hover:no-underline">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold">
                      {t(info.i18nKey, info.label)}
                    </span>
                    <Badge variant="secondary">{categoryConfigs.length}</Badge>
                  </div>
                </AccordionTrigger>
                <AccordionContent className="px-0 pb-0">
                  <SystemConfigTable
                    configs={categoryConfigs}
                    onEdit={setEditingConfig}
                    onHistory={setHistoryConfig}
                  />
                </AccordionContent>
              </AccordionItem>
            )
          })}
        </Accordion>
      )}

      {/* 编辑对话框 */}
      <SystemConfigEditDialog
        config={editingConfig}
        open={!!editingConfig}
        onOpenChange={(open) => !open && setEditingConfig(null)}
        onSave={handleSave}
        saving={updating}
      />

      {/* 历史对话框 */}
      <SystemConfigHistoryDialog
        config={historyConfig}
        open={!!historyConfig}
        onOpenChange={(open) => !open && setHistoryConfig(null)}
      />
    </div>
  )
}
