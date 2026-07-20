import type { AxiosError, InternalAxiosRequestConfig } from 'axios'
import { describe, expect, it } from 'vitest'
import {
  extractErrorMessage,
  isAuthError,
  isForbiddenError,
  isNetworkError,
  isNotFoundError,
} from '../errorHandler'

// Helper to create mock AxiosError
function createAxiosError(
  message: string,
  code?: string,
  response?: {
    status?: number
    data?: { error?: string; message?: string }
  },
): AxiosError {
  const error = new Error(message) as AxiosError
  error.isAxiosError = true
  error.code = code
  error.name = 'AxiosError'
  error.message = message
  error.config = {} as InternalAxiosRequestConfig
  error.toJSON = () => ({})
  if (response) {
    error.response = {
      status: response.status || 500,
      data: response.data || {},
      statusText: '',
      headers: {},
      config: {} as InternalAxiosRequestConfig,
    }
  }
  return error
}

describe('errorHandler', () => {
  // ==========================================================================
  // extractErrorMessage
  // ==========================================================================

  describe('extractErrorMessage', () => {
    it('should extract error from axios response.data.error', () => {
      const error = createAxiosError('Request failed', undefined, {
        status: 400,
        data: { error: 'Bad request error' },
      })

      const result = extractErrorMessage(error, 'Default message')
      expect(result).toBe('Bad request error')
    })

    it('should extract message from axios response.data.message', () => {
      const error = createAxiosError('Request failed', undefined, {
        status: 400,
        data: { message: 'Validation failed' },
      })

      const result = extractErrorMessage(error, 'Default message')
      expect(result).toBe('Validation failed')
    })

    it('should prefer message field over error field', () => {
      const error = createAxiosError('Request failed', undefined, {
        status: 400,
        data: { error: 'Error field', message: 'Message field' },
      })

      const result = extractErrorMessage(error, 'Default message')
      expect(result).toBe('Message field')
    })

    it('should use axios error message if no response data', () => {
      const error = createAxiosError('Network Error')

      const result = extractErrorMessage(error, 'Default message')
      expect(result).toBe('Network Error')
    })

    it('should use default message for non-axios Error', () => {
      const error = new Error('Some error')

      const result = extractErrorMessage(error, 'Default message')
      expect(result).toBe('Some error')
    })

    it('should use default message for non-Error object', () => {
      const result = extractErrorMessage('string error', 'Default message')
      expect(result).toBe('Default message')
    })

    it('should use default message for null', () => {
      const result = extractErrorMessage(null, 'Default message')
      expect(result).toBe('Default message')
    })

    it('should use default message for undefined', () => {
      const result = extractErrorMessage(undefined, 'Default message')
      expect(result).toBe('Default message')
    })

    it('should handle legacy error format', () => {
      const legacyError = {
        response: {
          data: {
            error: 'Legacy error message',
          },
        },
      }

      const result = extractErrorMessage(legacyError, 'Default message')
      expect(result).toBe('Legacy error message')
    })

    it('should skip empty error strings in response', () => {
      const error = createAxiosError('Request failed', undefined, {
        status: 400,
        data: { error: '', message: 'Non-empty message' },
      })

      const result = extractErrorMessage(error, 'Default message')
      expect(result).toBe('Non-empty message')
    })
  })

  // ==========================================================================
  // isNetworkError
  // ==========================================================================

  describe('isNetworkError', () => {
    it('should return true for ECONNREFUSED', () => {
      const error = createAxiosError('Connection refused', 'ECONNREFUSED')
      expect(isNetworkError(error)).toBe(true)
    })

    it('should return true for ERR_NETWORK', () => {
      const error = createAxiosError('Network error', 'ERR_NETWORK')
      expect(isNetworkError(error)).toBe(true)
    })

    it('should return true for ETIMEDOUT', () => {
      const error = createAxiosError('Timeout', 'ETIMEDOUT')
      expect(isNetworkError(error)).toBe(true)
    })

    it('should return true when no response', () => {
      const error = createAxiosError('No response')
      expect(isNetworkError(error)).toBe(true)
    })

    it('should return false for axios error with response', () => {
      const error = createAxiosError('Bad request', undefined, {
        status: 400,
        data: {},
      })
      expect(isNetworkError(error)).toBe(false)
    })

    it('should return false for non-axios error', () => {
      const error = new Error('Some error')
      expect(isNetworkError(error)).toBe(false)
    })

    it('should return false for null', () => {
      expect(isNetworkError(null)).toBe(false)
    })
  })

  // ==========================================================================
  // isAuthError
  // ==========================================================================

  describe('isAuthError', () => {
    it('should return true for 401 status', () => {
      const error = createAxiosError('Unauthorized', undefined, {
        status: 401,
        data: {},
      })
      expect(isAuthError(error)).toBe(true)
    })

    it('should return false for other status codes', () => {
      const error = createAxiosError('Forbidden', undefined, {
        status: 403,
        data: {},
      })
      expect(isAuthError(error)).toBe(false)
    })

    it('should return false for non-axios error', () => {
      const error = new Error('Some error')
      expect(isAuthError(error)).toBe(false)
    })

    it('should return false for axios error without response', () => {
      const error = createAxiosError('Network error')
      expect(isAuthError(error)).toBe(false)
    })
  })

  // ==========================================================================
  // isForbiddenError
  // ==========================================================================

  describe('isForbiddenError', () => {
    it('should return true for 403 status', () => {
      const error = createAxiosError('Forbidden', undefined, {
        status: 403,
        data: {},
      })
      expect(isForbiddenError(error)).toBe(true)
    })

    it('should return false for other status codes', () => {
      const error = createAxiosError('Unauthorized', undefined, {
        status: 401,
        data: {},
      })
      expect(isForbiddenError(error)).toBe(false)
    })

    it('should return false for non-axios error', () => {
      const error = new Error('Some error')
      expect(isForbiddenError(error)).toBe(false)
    })
  })

  // ==========================================================================
  // isNotFoundError
  // ==========================================================================

  describe('isNotFoundError', () => {
    it('should return true for 404 status', () => {
      const error = createAxiosError('Not found', undefined, {
        status: 404,
        data: {},
      })
      expect(isNotFoundError(error)).toBe(true)
    })

    it('should return false for other status codes', () => {
      const error = createAxiosError('Bad request', undefined, {
        status: 400,
        data: {},
      })
      expect(isNotFoundError(error)).toBe(false)
    })

    it('should return false for non-axios error', () => {
      const error = new Error('Some error')
      expect(isNotFoundError(error)).toBe(false)
    })
  })
})
