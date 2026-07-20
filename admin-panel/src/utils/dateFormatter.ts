/**
 * 日期格式化工具函数
 */

import { dateLogger } from './logger'

/**
 * 格式化日期为友好的显示格式
 * @param dateString - ISO 8601 日期字符串
 * @param locale - 语言代码（默认为 'en-US'）
 * @returns 格式化后的日期字符串
 */
export function formatDate(
  dateString: string,
  locale: string = 'en-US',
): string {
  try {
    return new Date(dateString).toLocaleDateString(locale, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  } catch (error) {
    dateLogger.error('Failed to format date:', error)
    return dateString
  }
}

/**
 * 格式化日期为完整的日期时间格式
 * @param dateString - ISO 8601 日期字符串
 * @param locale - 语言代码（默认为 'en-US'）
 * @returns 格式化后的日期时间字符串
 */
export function formatDateTime(
  dateString: string,
  locale: string = 'en-US',
): string {
  try {
    return new Date(dateString).toLocaleString(locale, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch (error) {
    dateLogger.error('Failed to format datetime:', error)
    return dateString
  }
}

/**
 * 将 RFC3339 时间格式转换为 datetime-local 输入框格式
 * @param rfc3339Time - RFC3339 格式的时间字符串
 * @returns datetime-local 格式的时间字符串（YYYY-MM-DDTHH:mm）
 */
export function formatDateTimeLocal(rfc3339Time: string): string {
  try {
    const date = new Date(rfc3339Time)
    // 获取本地时间偏移
    const offset = date.getTimezoneOffset()
    const localDate = new Date(date.getTime() - offset * 60 * 1000)
    // 转换为 YYYY-MM-DDTHH:mm 格式
    return localDate.toISOString().slice(0, 16)
  } catch (error) {
    dateLogger.error('Failed to format datetime-local:', error)
    return ''
  }
}

/**
 * 将 datetime-local 格式转换为 RFC3339 时间格式（ISO 8601）
 * @param datetimeLocal - datetime-local 格式的时间字符串（YYYY-MM-DDTHH:mm）
 * @returns RFC3339 格式的时间字符串
 */
export function formatToRFC3339(datetimeLocal: string): string {
  try {
    const date = new Date(datetimeLocal)
    return date.toISOString()
  } catch (error) {
    dateLogger.error('Failed to format to RFC3339:', error)
    return ''
  }
}

/**
 * 检查日期是否已过期
 * @param expiresAt - ISO 8601 日期字符串
 * @returns 是否已过期
 */
export function isExpired(expiresAt: string): boolean {
  try {
    return new Date(expiresAt) < new Date()
  } catch (error) {
    dateLogger.error('Failed to check expiration:', error)
    return false
  }
}

/**
 * 计算两个日期之间的天数差
 * @param date1 - 第一个日期字符串
 * @param date2 - 第二个日期字符串（默认为当前时间）
 * @returns 天数差（可能为负数）
 */
export function daysDifference(
  date1: string,
  date2: string = new Date().toISOString(),
): number {
  try {
    const d1 = new Date(date1)
    const d2 = new Date(date2)
    const diffTime = d2.getTime() - d1.getTime()
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24))
  } catch (error) {
    dateLogger.error('Failed to calculate days difference:', error)
    return 0
  }
}

/**
 * 格式化相对时间（如"2 hours ago"）
 * @param dateString - ISO 8601 日期字符串
 * @param locale - 语言代码（默认为 'en'）
 * @returns 相对时间字符串
 */
export function formatRelativeTime(
  dateString: string,
  locale: string = 'en',
): string {
  try {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffSec = Math.floor(diffMs / 1000)
    const diffMin = Math.floor(diffSec / 60)
    const diffHour = Math.floor(diffMin / 60)
    const diffDay = Math.floor(diffHour / 24)

    const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })

    if (diffDay > 0) {
      return rtf.format(-diffDay, 'day')
    } else if (diffHour > 0) {
      return rtf.format(-diffHour, 'hour')
    } else if (diffMin > 0) {
      return rtf.format(-diffMin, 'minute')
    } else {
      return rtf.format(-diffSec, 'second')
    }
  } catch (error) {
    dateLogger.error('Failed to format relative time:', error)
    return dateString
  }
}
