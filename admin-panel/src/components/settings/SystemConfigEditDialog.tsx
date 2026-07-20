import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { FiAlertTriangle, FiCode, FiRefreshCw } from 'react-icons/fi'
import type { z } from 'zod'

import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { TagInput } from '@/components/ui/tag-input'
import { Textarea } from '@/components/ui/textarea'
import { useConfigSchemaByKey } from '@/hooks/useConfigSchema'
import {
  type ConfigValueType,
  createConfigFormSchema,
} from '@/schemas/systemConfigSchema'
import type { ConfigItemResponse } from '@/services/api'
import { systemConfigService } from '@/services/systemConfigService'
import type { EnumOption } from '@/services/types.generated'
import { formatJSON, getConfigKeyLabel } from '@/utils/configUtils'

interface SystemConfigEditDialogProps {
  config: ConfigItemResponse | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onSave: (
    key: string,
    value: string,
  ) => Promise<{ requires_restart: boolean; message: string | null }>
  saving: boolean
}

export function SystemConfigEditDialog({
  config,
  open,
  onOpenChange,
  onSave,
  saving,
}: SystemConfigEditDialogProps) {
  const { t } = useTranslation()
  const [showRestartWarning, setShowRestartWarning] = useState(false)
  const [generating, setGenerating] = useState(false)

  // 从 API 获取 schema
  const { schema: configSchema } = useConfigSchemaByKey(config?.key || '')

  // 动态创建 zod schema
  const zodSchema = config
    ? createConfigFormSchema(config.value_type as ConfigValueType)
    : createConfigFormSchema('string')

  type FormValues = z.infer<typeof zodSchema>

  const form = useForm<FormValues>({
    resolver: zodResolver(zodSchema),
    defaultValues: {
      value: config?.value || '',
    },
  })

  // 当 config 变化时重置表单
  useEffect(() => {
    if (config) {
      form.reset({ value: config.value })
      setShowRestartWarning(false)
    }
  }, [config, form])

  const handleSave = async (data: FormValues) => {
    if (!config) return

    try {
      const result = await onSave(config.key, data.value)
      if (result.requires_restart) {
        setShowRestartWarning(true)
      } else {
        onOpenChange(false)
      }
    } catch {
      // Error is handled by the store
    }
  }

  const handleClose = () => {
    setShowRestartWarning(false)
    onOpenChange(false)
  }

  const handleFormatJSON = () => {
    const currentValue = form.getValues('value')
    const formatted = formatJSON(currentValue)
    form.setValue('value', formatted)
  }

  const handleSecureGenerate = async () => {
    if (!config || !configSchema?.action) return

    setGenerating(true)
    try {
      const result = await systemConfigService.executeAndSave(
        config.key,
        configSchema.action,
      )
      if (result.success) {
        if (result.requires_restart) {
          setShowRestartWarning(true)
        } else {
          onOpenChange(false)
        }
      }
    } catch {
      // Error is handled by the store
    } finally {
      setGenerating(false)
    }
  }

  if (!config) return null

  // 根据类型和 schema 渲染不同的输入组件
  const renderInput = (field: {
    value: string
    onChange: (value: string) => void
  }) => {
    // 0. 敏感配置 + Action → 只显示 Generate and Save 按钮
    if (config.is_sensitive && configSchema?.action) {
      return (
        <div className="space-y-3">
          <Alert>
            <AlertDescription>
              {t(
                'config.generateAndSaveHint',
                '点击按钮将自动生成并保存新的安全密钥',
              )}
            </AlertDescription>
          </Alert>
          <Button
            type="button"
            variant="default"
            onClick={handleSecureGenerate}
            disabled={generating}
            className="w-full"
          >
            <FiRefreshCw
              className={`mr-2 h-4 w-4 ${generating ? 'animate-spin' : ''}`}
            />
            {generating
              ? t('config.generating', 'Generating...')
              : t('config.generateAndSave', 'Generate and Save')}
          </Button>
        </div>
      )
    }

    // 1. Bool 类型优先使用 Switch（无论是否有 enum_options）
    if (config.value_type === 'bool') {
      return (
        <div className="flex items-center gap-3">
          <Switch
            id="config-bool-value"
            checked={field.value === 'true'}
            onCheckedChange={(checked: boolean) =>
              field.onChange(checked ? 'true' : 'false')
            }
          />
          <label
            htmlFor="config-bool-value"
            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            {field.value === 'true'
              ? t('common.enabled', 'Enabled')
              : t('common.disabled', 'Disabled')}
          </label>
        </div>
      )
    }

    // 2. StringArray 类型 → TagInput 组件
    if (config.value_type === 'stringarray') {
      let items: string[] = []
      try {
        items = JSON.parse(field.value || '[]')
      } catch {
        items = []
      }

      // 根据配置 key 提供不同的 placeholder
      const getPlaceholder = () => {
        switch (config.key) {
          case 'api.trusted_proxies':
            return t(
              'config.placeholder.trustedProxies',
              'IP or CIDR (e.g., 192.168.1.0/24)',
            )
          case 'cors.allowed_origins':
            return t(
              'config.placeholder.allowedOrigins',
              'Origin URL (e.g., https://example.com or *)',
            )
          case 'cors.allowed_headers':
            return t(
              'config.placeholder.allowedHeaders',
              'Header name (e.g., Content-Type)',
            )
          default:
            return t('common.addItem', 'Add item...')
        }
      }

      return (
        <TagInput
          value={items}
          onChange={(newItems) => field.onChange(JSON.stringify(newItems))}
          placeholder={getPlaceholder()}
        />
      )
    }

    // 3. EnumArray 类型 + enum_options → 多选 Checkbox
    if (config.value_type === 'enumarray' && configSchema?.enum_options) {
      let selected: string[] = []
      try {
        selected = JSON.parse(field.value || '[]')
      } catch {
        selected = []
      }
      return (
        <div className="space-y-2">
          {configSchema.enum_options.map((opt: EnumOption) => {
            const label = opt.label_i18n_key
              ? t(opt.label_i18n_key, opt.label)
              : opt.label

            return (
              <div key={opt.value} className="flex items-center gap-2">
                <Checkbox
                  id={`config-multi-${opt.value}`}
                  checked={selected.includes(opt.value)}
                  onCheckedChange={(checked: boolean) => {
                    const newSelected = checked
                      ? [...selected, opt.value]
                      : selected.filter((v) => v !== opt.value)
                    field.onChange(JSON.stringify(newSelected))
                  }}
                />
                <label
                  htmlFor={`config-multi-${opt.value}`}
                  className="text-sm"
                >
                  {label}
                </label>
              </div>
            )
          })}
        </div>
      )
    }

    // 4. enum_options：根据 value_type 决定单选还是多选
    if (configSchema?.enum_options) {
      // 4a. Json 类型 + enum_options → 多选 Checkbox（向后兼容）
      if (config.value_type === 'json') {
        let selected: string[] = []
        try {
          selected = JSON.parse(field.value || '[]')
        } catch {
          selected = []
        }
        return (
          <div className="space-y-2">
            {configSchema.enum_options.map((opt: EnumOption) => {
              const label = opt.label_i18n_key
                ? t(opt.label_i18n_key, opt.label)
                : opt.label

              return (
                <div key={opt.value} className="flex items-center gap-2">
                  <Checkbox
                    id={`config-multi-${opt.value}`}
                    checked={selected.includes(opt.value)}
                    onCheckedChange={(checked: boolean) => {
                      const newSelected = checked
                        ? [...selected, opt.value]
                        : selected.filter((v) => v !== opt.value)
                      field.onChange(JSON.stringify(newSelected))
                    }}
                  />
                  <label
                    htmlFor={`config-multi-${opt.value}`}
                    className="text-sm"
                  >
                    {label}
                  </label>
                </div>
              )
            })}
          </div>
        )
      }

      // 4b. 其他类型 + enum_options → 单选 Select
      return (
        <Select value={field.value} onValueChange={field.onChange}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {configSchema.enum_options.map((opt: EnumOption) => {
              const label = opt.label_i18n_key
                ? t(opt.label_i18n_key, opt.label)
                : opt.label
              const description = opt.description_i18n_key
                ? t(opt.description_i18n_key, opt.description || '')
                : opt.description

              return (
                <SelectItem key={opt.value} value={opt.value}>
                  <span>{label}</span>
                  {description && (
                    <span className="ml-2 text-muted-foreground">
                      - {description}
                    </span>
                  )}
                </SelectItem>
              )
            })}
          </SelectContent>
        </Select>
      )
    }

    // 5. 回退到基于 value_type 的默认渲染
    switch (config.value_type) {
      case 'int':
        return (
          <Input
            type="text"
            value={field.value}
            onChange={(e) => field.onChange(e.target.value)}
            className="font-mono"
            placeholder="42"
          />
        )

      case 'float':
        return (
          <Input
            type="text"
            value={field.value}
            onChange={(e) => field.onChange(e.target.value)}
            className="font-mono"
            placeholder="3.14"
          />
        )

      case 'json':
        // 没有 enum_options 的 json，使用 textarea
        return (
          <div className="space-y-2">
            <Textarea
              value={field.value}
              onChange={(e) => field.onChange(e.target.value)}
              className="font-mono text-sm"
              rows={10}
              placeholder='{"key": "value"}'
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleFormatJSON}
              className="w-full"
            >
              <FiCode className="mr-2 h-4 w-4" />
              {t('config.formatJSON', 'Format JSON')}
            </Button>
          </div>
        )

      default:
        return (
          <Input
            type="text"
            value={field.value}
            onChange={(e) => field.onChange(e.target.value)}
            className="font-mono"
          />
        )
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>
            {t('config.editConfig', 'Edit Configuration')}
          </DialogTitle>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSave)} className="space-y-4">
            {/* 配置键 */}
            <div className="space-y-2">
              <FormLabel>{t('config.key', 'Key')}</FormLabel>
              <div className="flex items-center gap-2">
                <code className="flex-1 rounded bg-muted px-3 py-2 font-mono text-sm">
                  {getConfigKeyLabel(config.key, t)}
                </code>
                <Badge variant="outline">{config.value_type}</Badge>
              </div>
            </div>

            {/* 配置值 */}
            <FormField
              control={form.control}
              name="value"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>{t('config.value', 'Value')}</FormLabel>
                  <FormControl>{renderInput(field)}</FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* 需要重启提示 */}
            {config.requires_restart && (
              <div className="flex items-center gap-2 rounded-md bg-yellow-50 p-3 text-sm text-yellow-800 dark:bg-yellow-950/50 dark:text-yellow-200">
                <FiAlertTriangle className="h-4 w-4 shrink-0" />
                <span>
                  {t(
                    'config.requiresRestartHint',
                    'Changes require server restart to take effect',
                  )}
                </span>
              </div>
            )}

            {/* 保存成功但需要重启的提示 */}
            {showRestartWarning && (
              <div className="flex items-center gap-2 rounded-md bg-green-50 p-3 text-sm text-green-800 dark:bg-green-950/50 dark:text-green-200">
                <span>
                  {t(
                    'config.savedNeedRestart',
                    'Configuration saved. Please restart the server for changes to take effect.',
                  )}
                </span>
              </div>
            )}

            <DialogFooter>
              <Button type="button" variant="outline" onClick={handleClose}>
                {showRestartWarning
                  ? t('common.close', 'Close')
                  : t('common.cancel', 'Cancel')}
              </Button>
              {!showRestartWarning && (
                <Button type="submit" disabled={saving}>
                  {saving
                    ? t('common.saving', 'Saving...')
                    : t('common.save', 'Save')}
                </Button>
              )}
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
