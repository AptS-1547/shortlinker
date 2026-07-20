import type { TFunction } from 'i18next'

import type { ConfigItemResponse } from '@/services/api'
import type { ConfigSchema } from '@/services/types.generated'

/**
 * 获取配置键的翻译标签
 * 如果没有翻译，返回原始 key
 */
export function getConfigKeyLabel(key: string, t: TFunction): string {
  const translationKey = `config.keys.${key}`
  const translated = t(translationKey)
  // 如果翻译结果和 key 相同，说明没有翻译
  return translated === translationKey ? key : translated
}

/**
 * 格式化 JSON 字符串
 * 如果输入不是有效的 JSON，返回原字符串
 */
export function formatJSON(jsonString: string): string {
  try {
    const parsed = JSON.parse(jsonString)
    return JSON.stringify(parsed, null, 2)
  } catch {
    return jsonString
  }
}

/**
 * 验证 JSON 字符串是否有效
 */
export function isValidJSON(jsonString: string): boolean {
  if (jsonString.trim() === '') {
    return false
  }
  try {
    JSON.parse(jsonString)
    return true
  } catch {
    return false
  }
}

/**
 * 配置分组的显示信息
 */
export const CONFIG_CATEGORY_INFO: Record<
  string,
  { label: string; i18nKey: string }
> = {
  auth: { label: 'Authentication', i18nKey: 'config.category.auth' },
  cookie: { label: 'Cookie Settings', i18nKey: 'config.category.cookie' },
  features: { label: 'Feature Flags', i18nKey: 'config.category.features' },
  routes: { label: 'Route Configuration', i18nKey: 'config.category.routes' },
  cors: { label: 'CORS Settings', i18nKey: 'config.category.cors' },
  tracking: { label: 'Click Tracking', i18nKey: 'config.category.tracking' },
  analytics: { label: 'Analytics', i18nKey: 'config.category.analytics' },
  cache: { label: 'Cache Settings', i18nKey: 'config.category.cache' },
  other: { label: 'Other', i18nKey: 'config.category.other' },
}

/**
 * 分组配置项
 */
export function groupConfigsByCategory(
  configs: ConfigItemResponse[],
  schemas: ConfigSchema[],
): Record<string, ConfigItemResponse[]> {
  const groups: Record<string, ConfigItemResponse[]> = {}

  // 创建 key -> schema 的映射，便于快速查找
  const schemaMap = new Map(schemas.map((s) => [s.key, s]))

  for (const config of configs) {
    const schema = schemaMap.get(config.key)
    const category = schema?.category || 'other'

    if (!groups[category]) {
      groups[category] = []
    }
    groups[category].push(config)
  }

  // 按 schema.order 对每个分组内的配置项排序
  for (const category of Object.keys(groups)) {
    groups[category].sort((a, b) => {
      const orderA = schemaMap.get(a.key)?.order ?? Number.MAX_SAFE_INTEGER
      const orderB = schemaMap.get(b.key)?.order ?? Number.MAX_SAFE_INTEGER
      return orderA - orderB
    })
  }

  return groups
}
