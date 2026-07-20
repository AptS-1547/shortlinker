import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
  formatDateTime,
  formatDate as formatDateUtil,
  formatRelativeTime,
} from '@/utils/dateFormatter'

export function useDateFormat() {
  const { i18n } = useTranslation()

  const formatDate = useCallback(
    (dateString: string): string => {
      return formatDateUtil(dateString, i18n.language)
    },
    [i18n.language],
  )

  const formatDateTimeStr = useCallback(
    (dateString: string): string => {
      return formatDateTime(dateString, i18n.language)
    },
    [i18n.language],
  )

  const formatRelative = useCallback(
    (dateString: string): string => {
      return formatRelativeTime(dateString, i18n.language)
    },
    [i18n.language],
  )

  return {
    formatDate,
    formatDateTime: formatDateTimeStr,
    formatRelative,
  }
}
