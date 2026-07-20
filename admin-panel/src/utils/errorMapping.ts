import type { ApiError } from '@/services/http'
import { ErrorCode } from '@/services/types.generated'

/**
 * 类型守卫：判断错误是否为 ApiError
 */
export function isApiError(error: unknown): error is ApiError {
  return (
    error instanceof Error &&
    'errorCode' in error &&
    typeof (error as ApiError).errorCode === 'number'
  )
}

/**
 * ErrorCode → i18n 翻译键映射表
 */
const ERROR_CODE_TO_I18N_KEY: Partial<Record<ErrorCode, string>> = {
  // 通用错误
  [ErrorCode.BadRequest]: 'errors.badRequest',
  [ErrorCode.Unauthorized]: 'errors.unauthorized',
  [ErrorCode.NotFound]: 'errors.notFound',
  [ErrorCode.RateLimitExceeded]: 'errors.tooManyRequests',
  [ErrorCode.InternalServerError]: 'errors.serverError',
  [ErrorCode.BatchSizeTooLarge]: 'errors.batchSizeTooLarge',
  [ErrorCode.FileTooLarge]: 'errors.fileTooLarge',
  [ErrorCode.InvalidDateFormat]: 'errors.invalidDateFormat',
  [ErrorCode.ServiceUnavailable]: 'errors.serviceUnavailable',

  // 认证错误
  [ErrorCode.AuthFailed]: 'errors.authFailed',
  [ErrorCode.TokenExpired]: 'errors.tokenExpired',
  [ErrorCode.TokenInvalid]: 'errors.tokenInvalid',
  [ErrorCode.CsrfInvalid]: 'errors.csrfInvalid',

  // 链接错误
  [ErrorCode.LinkNotFound]: 'errors.linkNotFound',
  [ErrorCode.LinkAlreadyExists]: 'errors.linkAlreadyExists',
  [ErrorCode.LinkInvalidUrl]: 'errors.linkInvalidUrl',
  [ErrorCode.LinkInvalidExpireTime]: 'errors.linkInvalidExpireTime',
  [ErrorCode.LinkPasswordHashError]: 'errors.linkPasswordHashError',
  [ErrorCode.LinkDatabaseError]: 'errors.linkDatabaseError',
  [ErrorCode.LinkEmptyCode]: 'errors.linkEmptyCode',
  [ErrorCode.LinkInvalidCode]: 'errors.linkInvalidCode',
  [ErrorCode.LinkReservedCode]: 'errors.linkReservedCode',

  // 导入导出错误
  [ErrorCode.ImportFailed]: 'errors.importFailed',
  [ErrorCode.ExportFailed]: 'errors.exportFailed',
  [ErrorCode.InvalidMultipartData]: 'errors.invalidMultipartData',
  [ErrorCode.FileReadError]: 'errors.fileReadError',
  [ErrorCode.CsvFileMissing]: 'errors.csvFileMissing',
  [ErrorCode.CsvParseError]: 'errors.csvParseError',
  [ErrorCode.CsvGenerationError]: 'errors.csvGenerationError',

  // 配置错误
  [ErrorCode.ConfigNotFound]: 'errors.configNotFound',
  [ErrorCode.ConfigUpdateFailed]: 'errors.configUpdateFailed',
  [ErrorCode.ConfigReloadFailed]: 'errors.configReloadFailed',

  // Analytics 错误
  [ErrorCode.AnalyticsQueryFailed]: 'errors.analyticsQueryFailed',
  [ErrorCode.AnalyticsLinkNotFound]: 'errors.analyticsLinkNotFound',
  [ErrorCode.AnalyticsInvalidDateRange]: 'errors.analyticsInvalidDateRange',
}

/**
 * 将错误代码映射到 i18n 翻译键
 */
export function getErrorI18nKey(error: unknown): string {
  // 网络错误优先检测（可能没有 code 属性）
  if (error instanceof Error && error.message?.includes('Network Error')) {
    return 'errors.network'
  }

  if (error instanceof Error && 'code' in error) {
    const apiError = error as ApiError
    return (
      ERROR_CODE_TO_I18N_KEY[apiError.code as ErrorCode] ?? 'errors.unknown'
    )
  }

  return 'errors.unknown'
}

/**
 * 获取用户友好的错误消息
 * @param error 错误对象
 * @param t i18n 翻译函数
 * @param fallback 后备消息
 */
export function getUserFriendlyErrorMessage(
  error: unknown,
  t: (key: string) => string,
  fallback?: string,
): string {
  const i18nKey = getErrorI18nKey(error)
  const message = t(i18nKey)

  // 如果翻译失败（返回的是 key 本身），使用后备消息或错误消息
  if (message === i18nKey) {
    if (fallback) return fallback
    if (error instanceof Error) return error.message
    return String(error)
  }

  return message
}

/**
 * 判断错误是否为网络错误
 */
export function isNetworkError(error: unknown): boolean {
  if (error instanceof Error) {
    // 网络错误的 code 为 undefined，通过 message 判断
    return error.message?.includes('Network Error') === true
  }
  return false
}

/**
 * 判断错误是否为认证错误
 */
export function isAuthenticationError(error: unknown): boolean {
  if (error instanceof Error && 'code' in error) {
    const code = (error as ApiError).code
    return (
      code === ErrorCode.Unauthorized ||
      code === ErrorCode.AuthFailed ||
      code === ErrorCode.TokenExpired ||
      code === ErrorCode.TokenInvalid
    )
  }
  return false
}

/**
 * 判断错误是否为服务器错误
 */
export function isServerError(error: unknown): boolean {
  if (error instanceof Error && 'code' in error) {
    const code = (error as ApiError).code
    return (
      code === ErrorCode.InternalServerError ||
      code === ErrorCode.ServiceUnavailable
    )
  }
  return false
}

/**
 * 判断错误是否可以重试
 */
export function isRetryableError(error: unknown): boolean {
  return isNetworkError(error) || isServerError(error)
}
