import axios, { type AxiosError } from 'axios'

/**
 * API 错误响应接口
 */
export interface ApiErrorResponse {
  code?: number
  message?: string
  error?: string // 兼容旧格式
  [key: string]: unknown
}

/**
 * 从 API 错误响应中提取错误消息
 * @param error - 未知类型的错误对象
 * @param defaultMessage - 默认错误消息
 * @returns 提取的错误消息
 */
export function extractErrorMessage(
  error: unknown,
  defaultMessage: string,
): string {
  // 检查是否是 Axios 错误
  if (axios.isAxiosError(error)) {
    const axiosError = error as AxiosError<ApiErrorResponse>

    // 尝试从响应中获取错误消息
    if (axiosError.response?.data) {
      const data = axiosError.response.data

      // 优先使用 message 字段（新格式）
      if (typeof data.message === 'string' && data.message) {
        return data.message
      }

      // 兼容旧格式 error 字段
      if (typeof data.error === 'string' && data.error) {
        return data.error
      }
    }

    // 使用 axios 错误消息
    if (axiosError.message) {
      return axiosError.message
    }
  }

  // 检查是否是标准 Error 对象
  if (error instanceof Error) {
    return error.message
  }

  // 检查是否是旧格式的错误对象（向后兼容）
  if (error && typeof error === 'object' && 'response' in error) {
    const legacyError = error as { response?: { data?: { error?: string } } }
    const errorMsg = legacyError.response?.data?.error
    if (typeof errorMsg === 'string' && errorMsg) {
      return errorMsg
    }
  }

  // 返回默认消息
  return defaultMessage
}

/**
 * 判断错误是否是网络错误
 * @param error - 未知类型的错误对象
 * @returns 是否是网络错误
 */
export function isNetworkError(error: unknown): boolean {
  if (!axios.isAxiosError(error)) {
    return false
  }

  return (
    error.code === 'ECONNREFUSED' ||
    error.code === 'ERR_NETWORK' ||
    error.code === 'ETIMEDOUT' ||
    !error.response
  )
}

/**
 * 判断错误是否是认证错误
 * @param error - 未知类型的错误对象
 * @returns 是否是认证错误
 */
export function isAuthError(error: unknown): boolean {
  if (!axios.isAxiosError(error)) {
    return false
  }

  return error.response?.status === 401
}

/**
 * 判断错误是否是权限错误
 * @param error - 未知类型的错误对象
 * @returns 是否是权限错误
 */
export function isForbiddenError(error: unknown): boolean {
  if (!axios.isAxiosError(error)) {
    return false
  }

  return error.response?.status === 403
}

/**
 * 判断错误是否是资源不存在错误
 * @param error - 未知类型的错误对象
 * @returns 是否是 404 错误
 */
export function isNotFoundError(error: unknown): boolean {
  if (!axios.isAxiosError(error)) {
    return false
  }

  return error.response?.status === 404
}
