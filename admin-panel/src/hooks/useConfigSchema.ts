import { useEffect, useMemo, useState } from 'react'
import { systemConfigService } from '@/services/systemConfigService'
import type { ConfigSchema } from '@/services/types.generated'

// 全局缓存，带过期时间
let cachedSchemas: ConfigSchema[] | null = null
let cacheTimestamp: number | null = null
let fetchPromise: Promise<ConfigSchema[]> | null = null

// 缓存 TTL: 5 分钟
const CACHE_TTL = 5 * 60 * 1000

/**
 * 获取所有配置的 schema
 *
 * 使用全局缓存，带过期机制
 * @param forceRefresh - 强制刷新，忽略缓存
 */
export function useConfigSchema(forceRefresh = false) {
  const [schemas, setSchemas] = useState<ConfigSchema[]>(cachedSchemas || [])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    // 检查缓存是否有效
    const now = Date.now()
    const isCacheValid =
      cachedSchemas &&
      cacheTimestamp &&
      now - cacheTimestamp < CACHE_TTL &&
      !forceRefresh

    // 已经有有效缓存，不需要再请求
    if (isCacheValid && cachedSchemas) {
      setSchemas(cachedSchemas)
      setIsLoading(false)
      return
    }

    // 强制刷新时清除旧缓存
    if (forceRefresh) {
      cachedSchemas = null
      cacheTimestamp = null
      fetchPromise = null
    }

    // 已经有正在进行的请求，等待它完成
    if (fetchPromise) {
      setIsLoading(true)
      fetchPromise
        .then((data) => {
          setSchemas(data)
          setIsLoading(false)
        })
        .catch((err) => {
          setError(err)
          setIsLoading(false)
        })
      return
    }

    // 发起新请求
    setIsLoading(true)
    fetchPromise = systemConfigService.fetchSchema()

    fetchPromise
      .then((data) => {
        cachedSchemas = data
        cacheTimestamp = Date.now()
        setSchemas(data)
        setIsLoading(false)
      })
      .catch((err) => {
        setError(err)
        setIsLoading(false)
        fetchPromise = null // 失败了允许重试
      })
  }, [forceRefresh])

  return { data: schemas, isLoading, error }
}

/**
 * 根据 key 获取单个配置的 schema
 */
export function useConfigSchemaByKey(key: string, forceRefresh = false) {
  const { data: schemas, isLoading, error } = useConfigSchema(forceRefresh)

  const schema = useMemo(
    () => schemas?.find((s) => s.key === key),
    [schemas, key],
  )

  return { schema, isLoading, error }
}

/**
 * 清除 schema 缓存
 *
 * 用于手动清除缓存，比如配置 schema 更新后
 */
export function clearConfigSchemaCache() {
  cachedSchemas = null
  cacheTimestamp = null
  fetchPromise = null
}
