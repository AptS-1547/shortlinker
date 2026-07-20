import axios, {
  type AxiosError,
  type AxiosInstance,
  type InternalAxiosRequestConfig,
} from 'axios'
import { LRUCache } from 'lru-cache'
import { appConfig } from '@/config/app'
import { forceLogout, refreshTokenFromHttp } from '@/stores/authStore'
import { httpLogger } from '@/utils/logger'
import { ENDPOINT_PATTERNS, ENDPOINTS } from './endpoints'
import { ErrorCode } from './types.generated'

/** CSRF Cookie 名称（硬编码常量） */
const CSRF_COOKIE_NAME = 'csrf_token'

// ==================== 错误处理 ====================
export class ApiError extends Error {
  status?: number
  code?: number
  errorCode?: ErrorCode

  constructor(message: string, status?: number, errorCode?: ErrorCode) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.errorCode = errorCode
    this.code = errorCode
  }
}

// ==================== 请求缓存管理 ====================

/**
 * 缓存条目（包含标签）
 */
interface CacheEntry<T> {
  data: T
  tags: Set<string>
}

/**
 * 缓存策略配置
 */
interface CachePolicy {
  ttl?: number // 缓存时间（毫秒）
  enabled?: boolean // 是否启用缓存
  tags?: string[] | ((params: Record<string, string>) => string[]) // 标签（静态或动态）
}

/**
 * 请求缓存管理器（基于 lru-cache）
 */
class RequestCache {
  private cache: LRUCache<string, CacheEntry<unknown>>
  private tagIndex: Map<string, Set<string>> // 标签 → URL 映射

  constructor() {
    this.cache = new LRUCache({
      max: 500, // 最大条目数（从 100 提升到 500）
      ttl: 5 * 60 * 1000, // 默认 TTL 5 分钟
      updateAgeOnGet: false, // 不续命，按固定时间过期（防抖场景）
      updateAgeOnHas: false,
    })
    this.tagIndex = new Map()
  }

  /**
   * 获取缓存数据
   */
  get<T>(url: string): T | null {
    const entry = this.cache.get(url) as CacheEntry<T> | undefined
    if (!entry) return null
    return entry.data
  }

  /**
   * 设置缓存数据（带标签）
   */
  set<T>(
    url: string,
    data: T,
    options?: { ttl?: number; tags?: string[] },
  ): void {
    const tags = new Set(options?.tags || [])
    const entry: CacheEntry<T> = { data, tags }

    // 存入缓存
    this.cache.set(url, entry, { ttl: options?.ttl })

    // 更新标签索引
    for (const tag of tags) {
      if (!this.tagIndex.has(tag)) {
        this.tagIndex.set(tag, new Set())
      }
      this.tagIndex.get(tag)?.add(url)
    }
  }

  /**
   * 按标签失效缓存（精细控制）
   */
  invalidateTags(tags: string[]): void {
    for (const tag of tags) {
      const urls = this.tagIndex.get(tag)
      if (urls) {
        for (const url of urls) {
          this.cache.delete(url)
        }
        this.tagIndex.delete(tag)
      }
    }
  }

  /**
   * 清除匹配模式的缓存（兼容旧接口）
   */
  clear(pattern?: string): void {
    if (!pattern) {
      this.cache.clear()
      this.tagIndex.clear()
      return
    }

    const keysToDelete: string[] = []
    for (const key of this.cache.keys()) {
      if (key.includes(pattern)) {
        keysToDelete.push(key)
      }
    }

    for (const key of keysToDelete) {
      const entry = this.cache.get(key) as CacheEntry<unknown> | undefined
      if (entry) {
        // 清理标签索引
        for (const tag of entry.tags) {
          this.tagIndex.get(tag)?.delete(key)
        }
      }
      this.cache.delete(key)
    }
  }

  /**
   * 清除所有缓存
   */
  clearAll(): void {
    this.cache.clear()
    this.tagIndex.clear()
  }

  /**
   * 获取缓存统计信息（调试用）
   */
  getStats() {
    return {
      size: this.cache.size,
      maxSize: this.cache.max,
      tags: this.tagIndex.size,
    }
  }
}

// ==================== 缓存策略配置 ====================

/**
 * 缓存策略映射表
 *
 * 定位：防抖机制，防止快速重复请求
 * TTL 策略：短 TTL，优先数据新鲜度
 */
const CACHE_POLICIES: Record<string, CachePolicy> = {
  // 链接管理（频繁修改，短 TTL）
  [`GET ${ENDPOINTS.LINKS.LIST}`]: {
    ttl: 30 * 1000, // 30 秒（从 2 分钟缩短）
    enabled: true,
    tags: ['links-list'],
  },
  [`GET ${ENDPOINT_PATTERNS.LINKS_SINGLE}`]: {
    ttl: 1 * 60 * 1000, // 1 分钟（从 5 分钟缩短）
    enabled: true,
    tags: (params) => [`link:${params.code}`],
  },
  [`GET ${ENDPOINTS.STATS}`]: {
    ttl: 30 * 1000, // 30 秒（从 1 分钟缩短）
    enabled: true,
    tags: ['stats'],
  },

  // 系统配置（读多写少，适度缓存）
  [`GET ${ENDPOINTS.CONFIG.LIST}`]: {
    ttl: 5 * 60 * 1000, // 5 分钟（从 10 分钟缩短）
    enabled: true,
    tags: ['config-all'],
  },
  [`GET ${ENDPOINT_PATTERNS.CONFIG_SINGLE}`]: {
    ttl: 5 * 60 * 1000, // 5 分钟（从 10 分钟缩短）
    enabled: true,
    tags: (params) => [`config:${params.key}`, 'config-all'],
  },
  [`GET ${ENDPOINT_PATTERNS.CONFIG_HISTORY}`]: {
    ttl: 2 * 60 * 1000, // 2 分钟（从 5 分钟缩短）
    enabled: true,
    tags: (params) => [`config:${params.key}:history`],
  },

  // 健康检查（典型防抖场景）
  'GET /health': {
    ttl: 10 * 1000, // 10 秒（从 30 秒缩短）
    enabled: true,
  },

  // 认证（不缓存）
  [`GET ${ENDPOINTS.AUTH.VERIFY}`]: {
    enabled: false,
  },
}

/**
 * 根据 URL 匹配缓存策略
 */
function matchCachePolicy(method: string, url: string): CachePolicy | null {
  // 提取路径（去掉查询参数）
  const path = url.split('?')[0]

  // 精确匹配
  const exactKey = `${method} ${path}`
  if (CACHE_POLICIES[exactKey]) {
    return CACHE_POLICIES[exactKey]
  }

  // 参数匹配（如 /links/:code）
  for (const [pattern, policy] of Object.entries(CACHE_POLICIES)) {
    const [pMethod, pPath] = pattern.split(' ')
    if (pMethod !== method) continue

    const regex = new RegExp(`^${pPath.replace(/:[^/]+/g, '([^/]+)')}$`)
    if (regex.test(path)) {
      return policy
    }
  }

  return null
}

/**
 * 从 URL 提取路径参数
 */
function extractPathParams(url: string): Record<string, string> {
  // 简化实现：提取最后一个路径段作为通用参数
  const path = url.split('?')[0]
  const segments = path.split('/').filter(Boolean)

  // 提取常见参数
  const params: Record<string, string> = {}

  // /links/:code 或 /config/:key
  if (segments.length >= 2) {
    const lastSegment = segments[segments.length - 1]
    // 假设最后一段是参数（code 或 key）
    if (segments[segments.length - 2] === 'links') {
      params.code = lastSegment
    } else if (segments[segments.length - 2] === 'config') {
      params.key = lastSegment
    }
  }

  return params
}

/**
 * 进行中的请求管理器（防止重复请求）
 */
class PendingRequestManager {
  private pending = new Map<string, Promise<unknown>>()

  /**
   * 生成请求 key
   */
  private generateKey(url: string): string {
    return url
  }

  /**
   * 添加进行中的请求
   */
  add<T>(url: string, promise: Promise<T>): Promise<T> {
    const key = this.generateKey(url)
    this.pending.set(key, promise)

    // 请求完成后自动移除
    promise.finally(() => {
      this.pending.delete(key)
    })

    return promise
  }

  /**
   * 获取进行中的请求
   */
  get<T>(url: string): Promise<T> | undefined {
    const key = this.generateKey(url)
    return this.pending.get(key) as Promise<T> | undefined
  }

  /**
   * 检查是否有进行中的请求
   */
  has(url: string): boolean {
    const key = this.generateKey(url)
    return this.pending.has(key)
  }
}

const handleAuthError = (): void => {
  if (typeof window !== 'undefined') {
    // 同步更新 authStore 状态
    forceLogout()
    // Cookie is automatically managed by browser, no localStorage cleanup needed
    const basePath = appConfig.basePath
    const loginPath =
      basePath && basePath !== '/' ? `${basePath}/login` : '/login'
    window.location.href = loginPath
  }
}

// Flag to prevent multiple refresh attempts
let isRefreshing = false
let refreshPromise: Promise<void> | null = null

// ==================== HTTP 客户端 ====================
export class HttpClient {
  private client: AxiosInstance
  private context: string
  private cache: RequestCache
  private pendingRequests: PendingRequestManager

  constructor(baseURL: string, context: string = '') {
    this.context = context
    this.cache = new RequestCache()
    this.pendingRequests = new PendingRequestManager()
    this.client = axios.create({
      baseURL,
      withCredentials: true, // Important: send cookies automatically
      timeout: 10000,
    })

    this.setupInterceptors()
  }

  private setupInterceptors(): void {
    // Request interceptor - add CSRF token header for mutating requests
    this.client.interceptors.request.use((config) => {
      // Add CSRF token for non-GET requests (POST, PUT, DELETE, PATCH)
      if (
        config.method &&
        !['get', 'head', 'options'].includes(config.method.toLowerCase())
      ) {
        const csrfToken = document.cookie
          .split('; ')
          .find((row) => row.startsWith(`${CSRF_COOKIE_NAME}=`))
          ?.split('=')[1]

        if (csrfToken) {
          config.headers['X-CSRF-Token'] = decodeURIComponent(csrfToken)
        }
      }
      return config
    })

    // Response interceptor with token refresh logic
    this.client.interceptors.response.use(
      (response) => response,
      async (error: AxiosError) => {
        const originalRequest = error.config as InternalAxiosRequestConfig & {
          _retry?: boolean
        }

        // Handle 401 errors
        if (error.response?.status === 401) {
          httpLogger.info('401 intercepted for:', originalRequest?.url)

          // Don't retry for auth endpoints
          if (
            originalRequest?.url?.includes('/auth/login') ||
            originalRequest?.url?.includes('/auth/refresh') ||
            originalRequest?.url?.includes('/auth/verify')
          ) {
            httpLogger.info('401 on auth endpoint, not retrying')
            return this.handleHttpError(error)
          }

          // Already retried, redirect to login
          if (originalRequest?._retry) {
            httpLogger.info('401 already retried, redirecting to login')
            handleAuthError()
            return this.handleHttpError(error)
          }

          // Try to refresh token using authStore's unified refresh logic
          if (!isRefreshing) {
            httpLogger.info('401 triggering token refresh')
            isRefreshing = true
            refreshPromise = refreshTokenFromHttp()
              .then(() => {
                httpLogger.info('Token refresh success, retrying request')
                isRefreshing = false
                refreshPromise = null
              })
              .catch(() => {
                httpLogger.info('Token refresh failed, redirecting to login')
                isRefreshing = false
                refreshPromise = null
                handleAuthError()
                throw error
              })
          } else {
            httpLogger.info('401 waiting for ongoing refresh')
          }

          // Wait for refresh to complete
          try {
            await refreshPromise
            // Mark as retried and retry original request
            originalRequest._retry = true
            httpLogger.info('Retrying original request:', originalRequest?.url)
            return this.client.request(originalRequest)
          } catch {
            return this.handleHttpError(error)
          }
        }

        return this.handleHttpError(error)
      },
    )
  }

  private handleHttpError(error: unknown): never {
    if (axios.isAxiosError(error)) {
      const axiosError = error as AxiosError

      // Canceled request (AbortController)
      if (axiosError.code === 'ERR_CANCELED') {
        const cancelError = new Error('Request canceled')
        cancelError.name = 'AbortError'
        throw cancelError
      }

      // Network error
      if (
        axiosError.code === 'ECONNREFUSED' ||
        axiosError.code === 'ERR_NETWORK'
      ) {
        throw new ApiError(
          'Network Error: Cannot connect to server',
          undefined,
          undefined,
        )
      }

      const status = axiosError.response?.status
      const context = this.context

      // 从新格式 {code, message, data?} 提取后端错误信息
      let backendMessage: string | undefined
      let backendErrorCode: ErrorCode | undefined
      if (axiosError.response?.data) {
        const data = axiosError.response.data as {
          code?: number
          message?: string
          error?: string // 兼容旧格式
          data?: { error?: string } // 兼容旧格式
        }
        backendErrorCode = data.code as ErrorCode | undefined
        backendMessage = data.message || data.error || data.data?.error
      }

      // 如果后端返回了 ErrorCode，直接使用
      if (
        backendErrorCode !== undefined &&
        backendErrorCode !== ErrorCode.Success
      ) {
        throw new ApiError(
          backendMessage || `Error ${status}`,
          status,
          backendErrorCode,
        )
      }

      // 兜底：按 HTTP 状态码构建错误
      const fallbackMessage = (() => {
        switch (status) {
          case 400:
            return `400: Bad Request${context ? ` for ${context}` : ''}`
          case 401:
            return axiosError.config?.url?.includes('/auth/login')
              ? 'Invalid credentials'
              : `401: Unauthorized access${context ? ` to ${context}` : ''}`
          case 403:
            return `403: Forbidden${context ? ` - insufficient permissions for ${context}` : ''}`
          case 404:
            return `404: ${context || 'Resource'} not found`
          case 429:
            return '429: Too many requests - please try again later'
          case 500:
            return `500: ${context ? `${context} failed` : 'Internal server error'}`
          case 503:
            return '503: Service temporarily unavailable'
          default:
            return `${status}: ${axiosError.response?.statusText || 'Unknown error'}`
        }
      })()

      const fallbackCode = (() => {
        switch (status) {
          case 400:
            return ErrorCode.BadRequest
          case 401:
            return axiosError.config?.url?.includes('/auth/login')
              ? ErrorCode.AuthFailed
              : ErrorCode.Unauthorized
          case 403:
            // 403 通常是 CSRF 保护触发
            return ErrorCode.CsrfInvalid
          case 404:
            return ErrorCode.NotFound
          case 409:
            // 409 冲突通常是链接已存在
            return ErrorCode.LinkAlreadyExists
          case 429:
            return ErrorCode.RateLimitExceeded
          case 500:
            return ErrorCode.InternalServerError
          case 503:
            return ErrorCode.ServiceUnavailable
          default:
            return ErrorCode.InternalServerError
        }
      })()

      throw new ApiError(
        backendMessage || fallbackMessage,
        status,
        fallbackCode,
      )
    }

    throw error instanceof Error
      ? error
      : new ApiError('Unknown error occurred')
  }

  async get<T = unknown>(
    url: string,
    options?: {
      signal?: AbortSignal
      ttl?: number
      skipCache?: boolean
      tags?: string[]
    },
  ): Promise<T> {
    const { signal, ttl, skipCache = false, tags } = options || {}

    // 匹配缓存策略
    const policy = matchCachePolicy('GET', url)
    const shouldCache = !skipCache && policy?.enabled !== false

    // 1. 检查缓存
    if (shouldCache) {
      const cached = this.cache.get<T>(url)
      if (cached !== null) {
        httpLogger.info(`Cache hit: ${url}`)
        return cached
      }
    }

    // 2. 检查是否有进行中的相同请求（skipCache 时忽略去重）
    if (!skipCache) {
      const pendingRequest = this.pendingRequests.get<T>(url)
      if (pendingRequest) {
        httpLogger.info('Request deduplication:', url)
        return pendingRequest
      }
    }

    // 3. 发起新请求
    const promise = this.client.get(url, { signal }).then((response) => {
      const data = response.data

      // 缓存响应数据
      if (shouldCache) {
        // 解析标签（支持动态标签）
        let cacheTags = tags || []
        if (policy?.tags) {
          if (typeof policy.tags === 'function') {
            // 提取路径参数
            const params = extractPathParams(url)
            cacheTags = [...cacheTags, ...policy.tags(params)]
          } else {
            cacheTags = [...cacheTags, ...policy.tags]
          }
        }

        this.cache.set(url, data, {
          ttl: ttl || policy?.ttl,
          tags: cacheTags,
        })
      }

      return data
    })

    // 4. 添加到进行中的请求
    return this.pendingRequests.add(url, promise)
  }

  /**
   * 按标签失效缓存（新增）
   */
  invalidateTags(tags: string[]): void {
    this.cache.invalidateTags(tags)
  }

  /**
   * 清除缓存（模式匹配，兼容旧接口）
   */
  clearCache(pattern?: string): void {
    this.cache.clear(pattern)
  }

  /**
   * 清除所有缓存
   */
  clearAllCache(): void {
    this.cache.clearAll()
  }

  async post<T = unknown>(url: string, data?: unknown): Promise<T> {
    const response = await this.client.post(url, data)
    return response.data
  }

  async put<T = unknown>(url: string, data?: unknown): Promise<T> {
    const response = await this.client.put(url, data)
    return response.data
  }

  async delete<T = unknown>(url: string, data?: unknown): Promise<T> {
    const response = await this.client.delete(url, { data })
    return response.data
  }
}

// ==================== 客户端实例 ====================
const getBaseUrl = (): string => {
  if (import.meta.env.PROD) {
    return typeof window !== 'undefined' ? window.location.origin : ''
  }
  return appConfig.apiBaseUrl || 'http://127.0.0.1:8080'
}

const baseUrl = getBaseUrl()

// Export config for compatibility
export const config = {
  baseUrl,
  adminRoutePrefix: appConfig.adminRoutePrefix,
  healthRoutePrefix: appConfig.healthRoutePrefix,
}

// Create client instances
export const adminClient = new HttpClient(
  `${baseUrl}${appConfig.adminRoutePrefix}`,
  'admin',
)
export const healthClient = new HttpClient(baseUrl, 'health')
