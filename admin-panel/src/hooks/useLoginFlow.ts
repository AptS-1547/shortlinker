import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { healthService } from '@/services/healthService'
import { useAuthStore } from '@/stores/authStore'
import { authLogger } from '@/utils/logger'

function authenticationErrorMessage(
  error: unknown,
  t: (key: string) => string,
) {
  if (!(error instanceof Error)) return t('auth.errors.authFailed')
  if (
    error.message.includes('Network Error') ||
    error.message.includes('ECONNREFUSED')
  ) {
    return t('auth.errors.networkError')
  }
  if (
    error.message.includes('401') ||
    error.message.includes('INVALID_CREDENTIALS')
  ) {
    return t('auth.errors.unauthorized')
  }
  if (error.message.includes('404')) return t('auth.errors.notFound')
  if (error.message.includes('500')) return t('auth.errors.serverError')
  return t('auth.errors.authFailed')
}

export function useLoginFlow() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const login = useAuthStore((state) => state.login)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState('')

  const authenticate = useCallback(
    async (password: string) => {
      if (!password) {
        setError(t('auth.errors.passwordRequired'))
        return
      }

      setIsSubmitting(true)
      setError('')
      try {
        setError(t('auth.authenticating'))
        await login(password)
        setError(t('auth.verifying'))
        await healthService.check()
        navigate('/dashboard')
      } catch (requestError) {
        authLogger.error('Authentication failed:', requestError)
        setError(authenticationErrorMessage(requestError, t))
      } finally {
        setIsSubmitting(false)
      }
    },
    [login, navigate, t],
  )

  return { authenticate, error, isSubmitting }
}
