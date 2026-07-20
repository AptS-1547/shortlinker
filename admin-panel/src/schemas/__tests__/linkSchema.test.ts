import { describe, expect, it } from 'vitest'
import { type LinkFormData, linkSchema } from '../linkSchema'

describe('linkSchema', () => {
  // ==========================================================================
  // code field
  // ==========================================================================

  describe('code field', () => {
    it('should accept undefined code (auto-generated)', () => {
      const data: LinkFormData = {
        code: undefined,
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept empty string code', () => {
      const data: LinkFormData = {
        code: '',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept valid alphanumeric code', () => {
      const data: LinkFormData = {
        code: 'abc123',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept code with hyphen', () => {
      const data: LinkFormData = {
        code: 'my-link',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept code with underscore', () => {
      const data: LinkFormData = {
        code: 'my_link',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept code with path separator', () => {
      const data: LinkFormData = {
        code: 'folder/link',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should reject code with spaces', () => {
      const data: LinkFormData = {
        code: 'my link',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('Invalid short code')
      }
    })

    it('should reject code with special characters', () => {
      const data: LinkFormData = {
        code: 'my@link',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })

    it('should reject reserved code: login', () => {
      const data: LinkFormData = {
        code: 'login',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('reserved')
      }
    })

    it('should reject reserved code: admin', () => {
      const data: LinkFormData = {
        code: 'admin',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })

    it('should reject reserved code case-insensitively', () => {
      const data: LinkFormData = {
        code: 'DASHBOARD',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })
  })

  // ==========================================================================
  // target field
  // ==========================================================================

  describe('target field', () => {
    it('should require target URL', () => {
      const data = {
        code: 'test',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })

    it('should reject empty target', () => {
      const data: LinkFormData = {
        code: 'test',
        target: '',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('required')
      }
    })

    it('should accept valid https URL', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept valid http URL', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'http://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept URL with path and query', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com/path?foo=bar',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should reject javascript: protocol', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'javascript:alert(1)',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })

    it('should reject data: protocol', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'data:text/html,<script>',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })

    it('should reject invalid URL format', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'not-a-url',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
    })
  })

  // ==========================================================================
  // password field
  // ==========================================================================

  describe('password field', () => {
    it('should accept undefined password', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        password: undefined,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept null password', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        password: null,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept password string', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        password: 'secret123',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept empty password string', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        password: '',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })
  })

  // ==========================================================================
  // expires_at field
  // ==========================================================================

  describe('expires_at field', () => {
    it('should accept undefined expires_at', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        expires_at: undefined,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept null expires_at', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        expires_at: null,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept future date', () => {
      const futureDate = new Date()
      futureDate.setFullYear(futureDate.getFullYear() + 1)
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        expires_at: futureDate.toISOString(),
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should reject past date', () => {
      const pastDate = '2020-01-01T00:00:00Z'
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        expires_at: pastDate,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('future')
      }
    })

    it('should accept empty string expires_at', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        expires_at: '',
      }
      const result = linkSchema.safeParse(data)
      // Empty string should be treated as "no expiration"
      expect(result.success).toBe(true)
    })
  })

  // ==========================================================================
  // force field
  // ==========================================================================

  describe('force field', () => {
    it('should accept undefined force', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        force: undefined,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept true force', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        force: true,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should accept false force', () => {
      const data: LinkFormData = {
        code: 'test',
        target: 'https://example.com',
        force: false,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })
  })

  // ==========================================================================
  // Complete form validation
  // ==========================================================================

  describe('complete form validation', () => {
    it('should validate complete valid form', () => {
      const futureDate = new Date()
      futureDate.setMonth(futureDate.getMonth() + 1)
      const data: LinkFormData = {
        code: 'my-link-2024',
        target: 'https://example.com/page?ref=test',
        password: 'secret',
        expires_at: futureDate.toISOString(),
        force: true,
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })

    it('should validate minimal valid form', () => {
      const data: LinkFormData = {
        target: 'https://example.com',
      }
      const result = linkSchema.safeParse(data)
      expect(result.success).toBe(true)
    })
  })
})
