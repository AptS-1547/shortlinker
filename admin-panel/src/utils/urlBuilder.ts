/**
 * URL 参数构建工具函数
 */

import { urlLogger } from './logger'

/**
 * 将对象转换为 URLSearchParams
 * 自动过滤掉 undefined、null 和空字符串的值
 * @param params - 参数对象
 * @returns URLSearchParams 实例
 */
export function buildUrlParams(
  params: Record<string, unknown>,
): URLSearchParams {
  const searchParams = new URLSearchParams()

  for (const [key, value] of Object.entries(params)) {
    // 跳过 undefined、null 和空字符串
    if (value === undefined || value === null || value === '') {
      continue
    }

    // 处理布尔值
    if (typeof value === 'boolean') {
      searchParams.append(key, value.toString())
      continue
    }

    // 处理数字
    if (typeof value === 'number') {
      searchParams.append(key, value.toString())
      continue
    }

    // 处理字符串
    if (typeof value === 'string') {
      searchParams.append(key, value)
      continue
    }

    // 处理数组
    if (Array.isArray(value)) {
      value.forEach((item) => {
        if (item !== undefined && item !== null && item !== '') {
          searchParams.append(key, String(item))
        }
      })
      continue
    }

    // 处理其他类型：尝试 JSON 序列化
    try {
      searchParams.append(key, JSON.stringify(value))
    } catch (error) {
      urlLogger.warn(`Failed to serialize parameter "${key}":`, error)
    }
  }

  return searchParams
}

/**
 * 构建完整的 URL，包含查询参数
 * @param baseUrl - 基础 URL
 * @param path - 路径
 * @param params - 查询参数对象
 * @returns 完整的 URL 字符串
 */
export function buildUrl(
  baseUrl: string,
  path: string = '',
  params?: Record<string, unknown>,
): string {
  let url = baseUrl

  // 添加路径
  if (path) {
    // 确保 baseUrl 以 / 结尾，path 不以 / 开头
    url = url.endsWith('/') ? url.slice(0, -1) : url
    const cleanPath = path.startsWith('/') ? path : `/${path}`
    url += cleanPath
  }

  // 添加查询参数
  if (params && Object.keys(params).length > 0) {
    const searchParams = buildUrlParams(params)
    const queryString = searchParams.toString()
    if (queryString) {
      url += `?${queryString}`
    }
  }

  return url
}

/**
 * 从 URL 中解析查询参数
 * @param url - URL 字符串
 * @returns 参数对象
 */
export function parseUrlParams(url: string): Record<string, string | string[]> {
  const params: Record<string, string | string[]> = {}

  try {
    const searchParams = new URL(url).searchParams

    for (const [key, value] of searchParams.entries()) {
      // 如果已存在该 key，转换为数组
      if (key in params) {
        const existing = params[key]
        if (Array.isArray(existing)) {
          existing.push(value)
        } else {
          params[key] = [existing, value]
        }
      } else {
        params[key] = value
      }
    }
  } catch (error) {
    urlLogger.error('Failed to parse URL params:', error)
  }

  return params
}

/**
 * 验证 URL 是否有效
 * @param url - URL 字符串
 * @returns 是否有效
 */
export function isValidUrl(url: string): boolean {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

/**
 * 标准化 URL（确保包含协议）
 * @param url - URL 字符串
 * @param defaultProtocol - 默认协议（默认为 'https'）
 * @returns 标准化后的 URL
 */
export function normalizeUrl(
  url: string,
  defaultProtocol: string = 'https',
): string {
  if (!url) {
    return ''
  }

  // 如果已经有协议，直接返回
  if (url.match(/^[a-z]+:\/\//i)) {
    return url
  }

  // 添加默认协议
  return `${defaultProtocol}://${url}`
}

/**
 * 构建短链接完整 URL
 * @param code - 短链接代码
 * @returns 完整的短链接 URL
 */
export function buildShortUrl(code: string): string {
  // 获取当前域名作为基础 URL
  const baseUrl = typeof window !== 'undefined' ? window.location.origin : ''
  return `${baseUrl}/${code}`
}
