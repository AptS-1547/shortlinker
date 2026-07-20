import { describe, expect, it } from 'vitest'
import {
  getReservedShortCodes,
  isEmpty,
  isLengthInRange,
  isReservedShortCode,
  isValidDate,
  isValidEmail,
  isValidHttpUrl,
  isValidShortCode,
  isValidUrl,
  validatePasswordStrength,
  validateSafeUrl,
  validateShortCode,
} from '../validators'

describe('validators', () => {
  // ==========================================================================
  // URL Validation
  // ==========================================================================

  describe('isValidUrl', () => {
    it('should accept valid http URL', () => {
      expect(isValidUrl('http://example.com')).toBe(true)
    })

    it('should accept valid https URL', () => {
      expect(isValidUrl('https://example.com')).toBe(true)
    })

    it('should accept URL with path', () => {
      expect(isValidUrl('https://example.com/path/to/page')).toBe(true)
    })

    it('should accept URL with query params', () => {
      expect(isValidUrl('https://example.com?foo=bar&baz=qux')).toBe(true)
    })

    it('should accept URL with port', () => {
      expect(isValidUrl('https://example.com:8080')).toBe(true)
    })

    it('should reject invalid URL', () => {
      expect(isValidUrl('not-a-url')).toBe(false)
    })

    it('should reject empty string', () => {
      expect(isValidUrl('')).toBe(false)
    })

    it('should reject null/undefined', () => {
      expect(isValidUrl(null as unknown as string)).toBe(false)
      expect(isValidUrl(undefined as unknown as string)).toBe(false)
    })
  })

  describe('validateSafeUrl', () => {
    it('should accept valid https URL', () => {
      const result = validateSafeUrl('https://example.com')
      expect(result.isValid).toBe(true)
      expect(result.error).toBeUndefined()
    })

    it('should accept valid http URL', () => {
      const result = validateSafeUrl('http://example.com')
      expect(result.isValid).toBe(true)
    })

    it('should reject javascript: protocol', () => {
      const result = validateSafeUrl('javascript:alert(1)')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('Dangerous protocol')
    })

    it('should reject data: protocol', () => {
      const result = validateSafeUrl('data:text/html,<script>')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('Dangerous protocol')
    })

    it('should reject file: protocol', () => {
      const result = validateSafeUrl('file:///etc/passwd')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('Dangerous protocol')
    })

    it('should reject ftp: protocol', () => {
      const result = validateSafeUrl('ftp://files.example.com')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('Invalid protocol')
    })

    it('should reject invalid URL format', () => {
      const result = validateSafeUrl('not-a-url')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('Invalid URL format')
    })

    it('should reject empty string', () => {
      const result = validateSafeUrl('')
      expect(result.isValid).toBe(false)
      expect(result.error).toBe('URL is required')
    })
  })

  describe('isValidHttpUrl', () => {
    it('should return true for valid http URL', () => {
      expect(isValidHttpUrl('http://example.com')).toBe(true)
    })

    it('should return false for dangerous URL', () => {
      expect(isValidHttpUrl('javascript:alert(1)')).toBe(false)
    })
  })

  // ==========================================================================
  // Short Code Validation
  // ==========================================================================

  describe('isValidShortCode', () => {
    it('should accept alphanumeric code', () => {
      expect(isValidShortCode('abc123')).toBe(true)
    })

    it('should accept code with underscore', () => {
      expect(isValidShortCode('my_link')).toBe(true)
    })

    it('should accept code with hyphen', () => {
      expect(isValidShortCode('my-link')).toBe(true)
    })

    it('should accept code with dot', () => {
      expect(isValidShortCode('my.link')).toBe(true)
    })

    it('should accept code with path separator', () => {
      expect(isValidShortCode('folder/link')).toBe(true)
    })

    it('should accept nested path', () => {
      expect(isValidShortCode('a/b/c')).toBe(true)
    })

    it('should reject code starting with /', () => {
      expect(isValidShortCode('/link')).toBe(false)
    })

    it('should reject code ending with /', () => {
      expect(isValidShortCode('link/')).toBe(false)
    })

    it('should reject code with consecutive slashes', () => {
      expect(isValidShortCode('a//b')).toBe(false)
    })

    it('should reject code with spaces', () => {
      expect(isValidShortCode('my link')).toBe(false)
    })

    it('should reject code with special characters', () => {
      expect(isValidShortCode('my@link')).toBe(false)
      expect(isValidShortCode('my#link')).toBe(false)
    })

    it('should reject empty string', () => {
      expect(isValidShortCode('')).toBe(false)
    })

    it('should respect min length', () => {
      expect(isValidShortCode('ab', 3)).toBe(false)
      expect(isValidShortCode('abc', 3)).toBe(true)
    })

    it('should respect max length', () => {
      expect(isValidShortCode('abcdef', 1, 5)).toBe(false)
      expect(isValidShortCode('abcde', 1, 5)).toBe(true)
    })
  })

  describe('isReservedShortCode', () => {
    it('should detect static reserved code: login', () => {
      expect(isReservedShortCode('login')).toBe(true)
    })

    it('should detect static reserved code: dashboard', () => {
      expect(isReservedShortCode('dashboard')).toBe(true)
    })

    it('should detect default reserved code: admin', () => {
      expect(isReservedShortCode('admin')).toBe(true)
    })

    it('should detect default reserved code: health', () => {
      expect(isReservedShortCode('health')).toBe(true)
    })

    it('should be case insensitive', () => {
      expect(isReservedShortCode('LOGIN')).toBe(true)
      expect(isReservedShortCode('Admin')).toBe(true)
    })

    it('should detect reserved code in path', () => {
      expect(isReservedShortCode('admin/test')).toBe(true)
      expect(isReservedShortCode('login/redirect')).toBe(true)
    })

    it('should not flag non-reserved code', () => {
      expect(isReservedShortCode('mylink')).toBe(false)
      expect(isReservedShortCode('test123')).toBe(false)
    })

    it('should return false for empty string', () => {
      expect(isReservedShortCode('')).toBe(false)
    })
  })

  describe('validateShortCode', () => {
    it('should accept valid code', () => {
      const result = validateShortCode('my-link-123')
      expect(result.isValid).toBe(true)
      expect(result.error).toBeUndefined()
    })

    it('should reject invalid format', () => {
      const result = validateShortCode('my link')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('Invalid short code format')
    })

    it('should reject reserved code', () => {
      const result = validateShortCode('admin')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('reserved keyword')
    })

    it('should reject reserved code in path', () => {
      const result = validateShortCode('login/test')
      expect(result.isValid).toBe(false)
      expect(result.error).toContain('reserved keyword')
    })
  })

  describe('getReservedShortCodes', () => {
    it('should include static reserved codes', () => {
      const reserved = getReservedShortCodes()
      expect(reserved).toContain('login')
      expect(reserved).toContain('dashboard')
      expect(reserved).toContain('links')
      expect(reserved).toContain('settings')
    })

    it('should include default admin and health', () => {
      const reserved = getReservedShortCodes()
      expect(reserved).toContain('admin')
      expect(reserved).toContain('health')
    })
  })

  // ==========================================================================
  // Password Validation
  // ==========================================================================

  describe('validatePasswordStrength', () => {
    it('should reject empty password', () => {
      const result = validatePasswordStrength('')
      expect(result.isValid).toBe(false)
      expect(result.score).toBe(0)
      expect(result.feedback).toContain('Password is required')
    })

    it('should reject short password', () => {
      const result = validatePasswordStrength('Short1')
      expect(result.isValid).toBe(false)
      expect(result.feedback).toContain(
        'Password should be at least 8 characters',
      )
    })

    it('should provide feedback for missing lowercase', () => {
      const result = validatePasswordStrength('UPPERCASE123')
      expect(result.feedback).toContain(
        'Password should contain lowercase letters',
      )
    })

    it('should provide feedback for missing uppercase', () => {
      const result = validatePasswordStrength('lowercase123')
      expect(result.feedback).toContain(
        'Password should contain uppercase letters',
      )
    })

    it('should provide feedback for missing numbers', () => {
      const result = validatePasswordStrength('PasswordOnly')
      expect(result.feedback).toContain('Password should contain numbers')
    })

    it('should provide feedback for missing special chars', () => {
      const result = validatePasswordStrength('Password123')
      expect(result.feedback).toContain(
        'Password should contain special characters',
      )
    })

    it('should accept strong password', () => {
      const result = validatePasswordStrength('MyP@ssword123!')
      expect(result.isValid).toBe(true)
      expect(result.score).toBeGreaterThanOrEqual(3)
    })

    it('should have max score of 4', () => {
      const result = validatePasswordStrength('SuperStrong#123Password!')
      expect(result.score).toBeLessThanOrEqual(4)
    })
  })

  // ==========================================================================
  // Email Validation
  // ==========================================================================

  describe('isValidEmail', () => {
    it('should accept valid email', () => {
      expect(isValidEmail('user@example.com')).toBe(true)
    })

    it('should accept email with subdomain', () => {
      expect(isValidEmail('user@mail.example.com')).toBe(true)
    })

    it('should accept email with plus sign', () => {
      expect(isValidEmail('user+tag@example.com')).toBe(true)
    })

    it('should reject email without @', () => {
      expect(isValidEmail('userexample.com')).toBe(false)
    })

    it('should reject email without domain', () => {
      expect(isValidEmail('user@')).toBe(false)
    })

    it('should reject email without TLD', () => {
      expect(isValidEmail('user@example')).toBe(false)
    })

    it('should reject empty string', () => {
      expect(isValidEmail('')).toBe(false)
    })
  })

  // ==========================================================================
  // Date Validation
  // ==========================================================================

  describe('isValidDate', () => {
    it('should accept valid ISO date', () => {
      expect(isValidDate('2024-01-15')).toBe(true)
    })

    it('should accept valid ISO datetime', () => {
      expect(isValidDate('2024-01-15T12:00:00Z')).toBe(true)
    })

    it('should reject invalid date string', () => {
      expect(isValidDate('not-a-date')).toBe(false)
    })

    it('should reject empty string', () => {
      expect(isValidDate('')).toBe(false)
    })

    it('should respect minDate', () => {
      expect(isValidDate('2024-01-01', '2024-01-15')).toBe(false)
      expect(isValidDate('2024-01-20', '2024-01-15')).toBe(true)
    })

    it('should respect maxDate', () => {
      expect(isValidDate('2024-01-20', undefined, '2024-01-15')).toBe(false)
      expect(isValidDate('2024-01-10', undefined, '2024-01-15')).toBe(true)
    })

    it('should respect both min and max', () => {
      expect(isValidDate('2024-01-15', '2024-01-10', '2024-01-20')).toBe(true)
      expect(isValidDate('2024-01-05', '2024-01-10', '2024-01-20')).toBe(false)
      expect(isValidDate('2024-01-25', '2024-01-10', '2024-01-20')).toBe(false)
    })
  })

  // ==========================================================================
  // String Utilities
  // ==========================================================================

  describe('isEmpty', () => {
    it('should return true for empty string', () => {
      expect(isEmpty('')).toBe(true)
    })

    it('should return true for whitespace only', () => {
      expect(isEmpty('   ')).toBe(true)
      expect(isEmpty('\t\n')).toBe(true)
    })

    it('should return true for null', () => {
      expect(isEmpty(null)).toBe(true)
    })

    it('should return true for undefined', () => {
      expect(isEmpty(undefined)).toBe(true)
    })

    it('should return false for non-empty string', () => {
      expect(isEmpty('hello')).toBe(false)
    })

    it('should return false for string with leading/trailing whitespace', () => {
      expect(isEmpty('  hello  ')).toBe(false)
    })
  })

  describe('isLengthInRange', () => {
    it('should return true when length is in range', () => {
      expect(isLengthInRange('hello', 1, 10)).toBe(true)
    })

    it('should return true when length equals min', () => {
      expect(isLengthInRange('abc', 3, 10)).toBe(true)
    })

    it('should return true when length equals max', () => {
      expect(isLengthInRange('abcde', 1, 5)).toBe(true)
    })

    it('should return false when length is less than min', () => {
      expect(isLengthInRange('ab', 3, 10)).toBe(false)
    })

    it('should return false when length is greater than max', () => {
      expect(isLengthInRange('abcdef', 1, 5)).toBe(false)
    })

    it('should return false for empty string when min > 0', () => {
      expect(isLengthInRange('', 1, 10)).toBe(false)
    })

    it('should return false for null/undefined', () => {
      expect(isLengthInRange(null as unknown as string, 1, 10)).toBe(false)
      expect(isLengthInRange(undefined as unknown as string, 1, 10)).toBe(false)
    })
  })
})
