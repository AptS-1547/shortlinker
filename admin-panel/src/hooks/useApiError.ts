import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { getErrorI18nKey, isApiError } from '@/utils/errorMapping'

export interface UseApiErrorOptions {
  /** 是否静默处理（不显示通知） */
  silent?: boolean
  /** 自定义错误标题 */
  title?: string
}

/**
 * 统一 API 错误处理 Hook
 *
 * 自动处理 ApiError、普通 Error 和未知错误，
 * 根据 ErrorCode 显示对应的国际化错误消息。
 *
 * @example
 * ```tsx
 * const { handleError } = useApiError()
 *
 * try {
 *   await createLink(data)
 * } catch (error) {
 *   handleError(error) // 自动显示精细错误消息
 * }
 * ```
 */
export function useApiError() {
  const { t } = useTranslation()

  const handleError = useCallback(
    (error: unknown, options: UseApiErrorOptions = {}) => {
      const { silent = false, title } = options

      let message: string

      if (isApiError(error)) {
        // ApiError: 优先使用 i18n 翻译
        const i18nKey = getErrorI18nKey(error)
        const translated = t(i18nKey)
        // 如果翻译失败（返回 key 本身），使用后端消息
        message = translated !== i18nKey ? translated : error.message
      } else if (error instanceof Error) {
        // 普通 Error
        message = error.message
      } else if (typeof error === 'string') {
        // 字符串错误
        message = error
      } else {
        // 未知错误
        message = t('errors.unknown')
      }

      // 显示通知（除非静默）
      if (!silent) {
        toast.error(title || message)
      }

      return message
    },
    [t],
  )

  /**
   * 包装异步函数，自动处理错误
   *
   * @example
   * ```tsx
   * const { withErrorHandling } = useApiError()
   *
   * const result = await withErrorHandling(
   *   () => api.createLink(data),
   *   { title: '创建链接失败' }
   * )
   * ```
   */
  const withErrorHandling = useCallback(
    <T>(
      asyncFn: () => Promise<T>,
      options: UseApiErrorOptions = {},
    ): Promise<T | undefined> => {
      return asyncFn().catch((error) => {
        handleError(error, options)
        return undefined
      })
    },
    [handleError],
  )

  return { handleError, withErrorHandling }
}

export default useApiError
