import { describe, expect, it } from 'vitest'
import type { ApiError } from '@/services/http'
import { ErrorCode } from '@/services/types'
import {
  getErrorI18nKey,
  getUserFriendlyErrorMessage,
  isAuthenticationError,
  isNetworkError,
  isRetryableError,
  isServerError,
} from '../errorMapping'

// Helper to create mock ApiError with numeric ErrorCode
function createApiError(
  code: number | undefined,
  message = 'Test error',
): ApiError {
  const error = new Error(message) as ApiError
  error.code = code
  error.errorCode = code as ErrorCode | undefined
  return error
}

// Helper for network errors (no code, message-based detection)
function createNetworkError(
  message = 'Network Error: Cannot connect to server',
): ApiError {
  const error = new Error(message) as ApiError
  return error
}

describe('errorMapping', () => {
  // ==========================================================================
  // getErrorI18nKey
  // ==========================================================================

  describe('getErrorI18nKey', () => {
    it('should return errors.network for network error', () => {
      const error = createNetworkError()
      expect(getErrorI18nKey(error)).toBe('errors.network')
    })

    it('should return errors.badRequest for BadRequest', () => {
      const error = createApiError(ErrorCode.BadRequest)
      expect(getErrorI18nKey(error)).toBe('errors.badRequest')
    })

    it('should return errors.authFailed for AuthFailed', () => {
      const error = createApiError(ErrorCode.AuthFailed)
      expect(getErrorI18nKey(error)).toBe('errors.authFailed')
    })

    it('should return errors.unauthorized for Unauthorized', () => {
      const error = createApiError(ErrorCode.Unauthorized)
      expect(getErrorI18nKey(error)).toBe('errors.unauthorized')
    })

    it('should return errors.notFound for NotFound', () => {
      const error = createApiError(ErrorCode.NotFound)
      expect(getErrorI18nKey(error)).toBe('errors.notFound')
    })

    it('should return errors.tooManyRequests for RateLimitExceeded', () => {
      const error = createApiError(ErrorCode.RateLimitExceeded)
      expect(getErrorI18nKey(error)).toBe('errors.tooManyRequests')
    })

    it('should return errors.serverError for InternalServerError', () => {
      const error = createApiError(ErrorCode.InternalServerError)
      expect(getErrorI18nKey(error)).toBe('errors.serverError')
    })

    it('should return errors.serviceUnavailable for ServiceUnavailable', () => {
      const error = createApiError(ErrorCode.ServiceUnavailable)
      expect(getErrorI18nKey(error)).toBe('errors.serviceUnavailable')
    })

    it('should return errors.unknown for unknown code', () => {
      const error = createApiError(99999)
      expect(getErrorI18nKey(error)).toBe('errors.unknown')
    })

    it('should return errors.unknown for non-Error object', () => {
      expect(getErrorI18nKey('string error')).toBe('errors.unknown')
    })

    it('should return errors.unknown for null', () => {
      expect(getErrorI18nKey(null)).toBe('errors.unknown')
    })

    it('should return errors.unknown for Error without code', () => {
      const error = new Error('No code')
      expect(getErrorI18nKey(error)).toBe('errors.unknown')
    })

    it('should return errors.linkNotFound for LinkNotFound', () => {
      const error = createApiError(ErrorCode.LinkNotFound)
      expect(getErrorI18nKey(error)).toBe('errors.linkNotFound')
    })

    it('should return errors.csrfInvalid for CsrfInvalid', () => {
      const error = createApiError(ErrorCode.CsrfInvalid)
      expect(getErrorI18nKey(error)).toBe('errors.csrfInvalid')
    })
  })

  // ==========================================================================
  // getUserFriendlyErrorMessage
  // ==========================================================================

  describe('getUserFriendlyErrorMessage', () => {
    const mockTranslate = (key: string): string => {
      const translations: Record<string, string> = {
        'errors.network': 'Network connection failed',
        'errors.unauthorized': 'Please log in again',
        'errors.unknown': 'errors.unknown', // Simulate untranslated key
      }
      return translations[key] || key
    }

    it('should return translated message for known error', () => {
      const error = createNetworkError()
      const result = getUserFriendlyErrorMessage(error, mockTranslate)
      expect(result).toBe('Network connection failed')
    })

    it('should return fallback when translation fails', () => {
      const error = createApiError(99999)
      const result = getUserFriendlyErrorMessage(
        error,
        mockTranslate,
        'Fallback message',
      )
      expect(result).toBe('Fallback message')
    })

    it('should return error message when no translation and no fallback', () => {
      const error = createApiError(99999, 'Original error message')
      const result = getUserFriendlyErrorMessage(error, mockTranslate)
      expect(result).toBe('Original error message')
    })

    it('should return stringified error for non-Error objects', () => {
      const result = getUserFriendlyErrorMessage('string error', mockTranslate)
      expect(result).toBe('string error')
    })
  })

  // ==========================================================================
  // isNetworkError
  // ==========================================================================

  describe('isNetworkError', () => {
    it('should return true for network error', () => {
      const error = createNetworkError()
      expect(isNetworkError(error)).toBe(true)
    })

    it('should return false for other error codes', () => {
      const error = createApiError(ErrorCode.InternalServerError)
      expect(isNetworkError(error)).toBe(false)
    })

    it('should return false for non-ApiError', () => {
      expect(isNetworkError(new Error('test'))).toBe(false)
    })

    it('should return false for null', () => {
      expect(isNetworkError(null)).toBe(false)
    })
  })

  // ==========================================================================
  // isAuthenticationError
  // ==========================================================================

  describe('isAuthenticationError', () => {
    it('should return true for Unauthorized code', () => {
      const error = createApiError(ErrorCode.Unauthorized)
      expect(isAuthenticationError(error)).toBe(true)
    })

    it('should return true for AuthFailed code', () => {
      const error = createApiError(ErrorCode.AuthFailed)
      expect(isAuthenticationError(error)).toBe(true)
    })

    it('should return true for TokenExpired code', () => {
      const error = createApiError(ErrorCode.TokenExpired)
      expect(isAuthenticationError(error)).toBe(true)
    })

    it('should return false for other error codes', () => {
      const error = createApiError(ErrorCode.BadRequest)
      expect(isAuthenticationError(error)).toBe(false)
    })

    it('should return false for non-ApiError', () => {
      expect(isAuthenticationError('string')).toBe(false)
    })
  })

  // ==========================================================================
  // isServerError
  // ==========================================================================

  describe('isServerError', () => {
    it('should return true for InternalServerError code', () => {
      const error = createApiError(ErrorCode.InternalServerError)
      expect(isServerError(error)).toBe(true)
    })

    it('should return true for ServiceUnavailable code', () => {
      const error = createApiError(ErrorCode.ServiceUnavailable)
      expect(isServerError(error)).toBe(true)
    })

    it('should return false for other error codes', () => {
      const error = createApiError(ErrorCode.BadRequest)
      expect(isServerError(error)).toBe(false)
    })

    it('should return false for non-ApiError', () => {
      expect(isServerError(undefined)).toBe(false)
    })
  })

  // ==========================================================================
  // isRetryableError
  // ==========================================================================

  describe('isRetryableError', () => {
    it('should return true for network errors', () => {
      const error = createNetworkError()
      expect(isRetryableError(error)).toBe(true)
    })

    it('should return true for server errors', () => {
      const error = createApiError(ErrorCode.InternalServerError)
      expect(isRetryableError(error)).toBe(true)
    })

    it('should return true for service unavailable', () => {
      const error = createApiError(ErrorCode.ServiceUnavailable)
      expect(isRetryableError(error)).toBe(true)
    })

    it('should return false for client errors', () => {
      const error = createApiError(ErrorCode.BadRequest)
      expect(isRetryableError(error)).toBe(false)
    })

    it('should return false for auth errors', () => {
      const error = createApiError(ErrorCode.Unauthorized)
      expect(isRetryableError(error)).toBe(false)
    })
  })
})
