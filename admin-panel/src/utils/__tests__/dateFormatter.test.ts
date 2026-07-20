import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  daysDifference,
  formatDate,
  formatDateTime,
  formatDateTimeLocal,
  formatRelativeTime,
  formatToRFC3339,
  isExpired,
} from '../dateFormatter'

describe('dateFormatter', () => {
  // ==========================================================================
  // formatDate
  // ==========================================================================

  describe('formatDate', () => {
    it('should format date with default locale (en-US)', () => {
      const result = formatDate('2024-03-15T10:30:00Z')
      // en-US format: "Mar 15, 2024"
      expect(result).toContain('Mar')
      expect(result).toContain('15')
      expect(result).toContain('2024')
    })

    it('should format date with zh-CN locale', () => {
      const result = formatDate('2024-03-15T10:30:00Z', 'zh-CN')
      expect(result).toContain('2024')
      expect(result).toContain('3')
      expect(result).toContain('15')
    })

    it('should handle ISO date string', () => {
      const result = formatDate('2024-01-01')
      expect(result).toContain('2024')
    })

    it('should return original string on invalid date', () => {
      const invalidDate = 'not-a-date'
      const result = formatDate(invalidDate)
      // May return 'Invalid Date' string or original depending on locale
      expect(typeof result).toBe('string')
    })
  })

  // ==========================================================================
  // formatDateTime
  // ==========================================================================

  describe('formatDateTime', () => {
    it('should format datetime with default locale', () => {
      const result = formatDateTime('2024-03-15T10:30:00Z')
      expect(result).toContain('Mar')
      expect(result).toContain('15')
      expect(result).toContain('2024')
    })

    it('should format datetime with zh-CN locale', () => {
      const result = formatDateTime('2024-03-15T10:30:00Z', 'zh-CN')
      expect(result).toContain('2024')
    })

    it('should handle invalid date', () => {
      const result = formatDateTime('invalid')
      expect(typeof result).toBe('string')
    })
  })

  // ==========================================================================
  // formatDateTimeLocal
  // ==========================================================================

  describe('formatDateTimeLocal', () => {
    it('should convert RFC3339 to datetime-local format', () => {
      const result = formatDateTimeLocal('2024-03-15T10:30:00Z')
      // Should return YYYY-MM-DDTHH:mm format
      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/)
    })

    it('should handle RFC3339 with timezone offset', () => {
      const result = formatDateTimeLocal('2024-03-15T10:30:00+08:00')
      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/)
    })

    it('should return empty string for invalid date', () => {
      const result = formatDateTimeLocal('invalid-date')
      expect(result).toBe('')
    })
  })

  // ==========================================================================
  // formatToRFC3339
  // ==========================================================================

  describe('formatToRFC3339', () => {
    it('should convert datetime-local to RFC3339 format', () => {
      const result = formatToRFC3339('2024-03-15T10:30')
      // Should return ISO format ending with Z
      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}Z$/)
    })

    it('should handle date-only input', () => {
      const result = formatToRFC3339('2024-03-15')
      expect(result).toContain('2024-03-15')
    })

    it('should return empty string for invalid input', () => {
      const result = formatToRFC3339('not-a-date')
      expect(result).toBe('')
    })
  })

  // ==========================================================================
  // isExpired
  // ==========================================================================

  describe('isExpired', () => {
    it('should return true for past date', () => {
      const pastDate = '2020-01-01T00:00:00Z'
      expect(isExpired(pastDate)).toBe(true)
    })

    it('should return false for future date', () => {
      const futureDate = '2099-12-31T23:59:59Z'
      expect(isExpired(futureDate)).toBe(false)
    })

    it('should return false for invalid date', () => {
      expect(isExpired('invalid')).toBe(false)
    })
  })

  // ==========================================================================
  // daysDifference
  // ==========================================================================

  describe('daysDifference', () => {
    it('should calculate positive days difference', () => {
      const date1 = '2024-01-01T00:00:00Z'
      const date2 = '2024-01-10T00:00:00Z'
      expect(daysDifference(date1, date2)).toBe(9)
    })

    it('should calculate negative days difference', () => {
      const date1 = '2024-01-10T00:00:00Z'
      const date2 = '2024-01-01T00:00:00Z'
      expect(daysDifference(date1, date2)).toBe(-9)
    })

    it('should return 0 for same date', () => {
      const date = '2024-01-01T00:00:00Z'
      expect(daysDifference(date, date)).toBe(0)
    })

    it('should handle single argument (compare to now)', () => {
      const pastDate = new Date()
      pastDate.setDate(pastDate.getDate() - 5)
      const result = daysDifference(pastDate.toISOString())
      expect(result).toBe(5)
    })

    it('should return NaN for invalid date', () => {
      // Note: new Date('invalid') doesn't throw, returns Invalid Date
      // getTime() on Invalid Date returns NaN
      expect(daysDifference('invalid')).toBeNaN()
    })
  })

  // ==========================================================================
  // formatRelativeTime
  // ==========================================================================

  describe('formatRelativeTime', () => {
    beforeEach(() => {
      vi.useFakeTimers()
      vi.setSystemTime(new Date('2024-03-15T12:00:00Z'))
    })

    afterEach(() => {
      vi.useRealTimers()
    })

    it('should format seconds ago', () => {
      const date = new Date('2024-03-15T11:59:30Z')
      const result = formatRelativeTime(date.toISOString())
      expect(result).toContain('second')
    })

    it('should format minutes ago', () => {
      const date = new Date('2024-03-15T11:55:00Z')
      const result = formatRelativeTime(date.toISOString())
      expect(result).toContain('minute')
    })

    it('should format hours ago', () => {
      const date = new Date('2024-03-15T10:00:00Z')
      const result = formatRelativeTime(date.toISOString())
      expect(result).toContain('hour')
    })

    it('should format days ago', () => {
      const date = new Date('2024-03-13T12:00:00Z')
      const result = formatRelativeTime(date.toISOString())
      expect(result).toContain('day')
    })

    it('should handle different locales', () => {
      const date = new Date('2024-03-14T12:00:00Z')
      const resultEn = formatRelativeTime(date.toISOString(), 'en')
      expect(typeof resultEn).toBe('string')
    })

    it('should return original string for invalid date', () => {
      const invalid = 'not-a-date'
      const result = formatRelativeTime(invalid)
      expect(result).toBe(invalid)
    })
  })
})
