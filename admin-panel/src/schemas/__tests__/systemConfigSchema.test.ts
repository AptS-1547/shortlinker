import { describe, expect, it } from 'vitest'
import {
  createConfigFormSchema,
  createConfigValueSchema,
} from '../systemConfigSchema'

describe('systemConfigSchema', () => {
  // ==========================================================================
  // createConfigValueSchema - bool type
  // ==========================================================================

  describe('createConfigValueSchema - bool', () => {
    const schema = createConfigValueSchema('bool')

    it('should accept "true"', () => {
      const result = schema.safeParse('true')
      expect(result.success).toBe(true)
    })

    it('should accept "false"', () => {
      const result = schema.safeParse('false')
      expect(result.success).toBe(true)
    })

    it('should reject "True" (case sensitive)', () => {
      const result = schema.safeParse('True')
      expect(result.success).toBe(false)
    })

    it('should reject "1"', () => {
      const result = schema.safeParse('1')
      expect(result.success).toBe(false)
    })

    it('should reject "yes"', () => {
      const result = schema.safeParse('yes')
      expect(result.success).toBe(false)
    })

    it('should reject empty string', () => {
      const result = schema.safeParse('')
      expect(result.success).toBe(false)
    })

    it('should have correct error message', () => {
      const result = schema.safeParse('invalid')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('true')
        expect(result.error.issues[0].message).toContain('false')
      }
    })
  })

  // ==========================================================================
  // createConfigValueSchema - int type
  // ==========================================================================

  describe('createConfigValueSchema - int', () => {
    const schema = createConfigValueSchema('int')

    it('should accept positive integer', () => {
      const result = schema.safeParse('42')
      expect(result.success).toBe(true)
    })

    it('should accept zero', () => {
      const result = schema.safeParse('0')
      expect(result.success).toBe(true)
    })

    it('should accept negative integer', () => {
      const result = schema.safeParse('-10')
      expect(result.success).toBe(true)
    })

    it('should accept large integer', () => {
      const result = schema.safeParse('9999999999')
      expect(result.success).toBe(true)
    })

    it('should reject float', () => {
      const result = schema.safeParse('3.14')
      expect(result.success).toBe(false)
    })

    it('should reject non-numeric string', () => {
      const result = schema.safeParse('abc')
      expect(result.success).toBe(false)
    })

    it('should reject empty string', () => {
      const result = schema.safeParse('')
      expect(result.success).toBe(false)
    })

    it('should reject string with spaces', () => {
      const result = schema.safeParse(' 42 ')
      expect(result.success).toBe(false)
    })

    it('should reject hex format', () => {
      const result = schema.safeParse('0xFF')
      expect(result.success).toBe(false)
    })

    it('should have correct error message', () => {
      const result = schema.safeParse('not-a-number')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('integer')
      }
    })
  })

  // ==========================================================================
  // createConfigValueSchema - json type
  // ==========================================================================

  describe('createConfigValueSchema - json', () => {
    const schema = createConfigValueSchema('json')

    it('should accept valid JSON object', () => {
      const result = schema.safeParse('{"key": "value"}')
      expect(result.success).toBe(true)
    })

    it('should accept valid JSON array', () => {
      const result = schema.safeParse('[1, 2, 3]')
      expect(result.success).toBe(true)
    })

    it('should accept JSON string', () => {
      const result = schema.safeParse('"hello"')
      expect(result.success).toBe(true)
    })

    it('should accept JSON number', () => {
      const result = schema.safeParse('42')
      expect(result.success).toBe(true)
    })

    it('should accept JSON boolean', () => {
      const result = schema.safeParse('true')
      expect(result.success).toBe(true)
    })

    it('should accept JSON null', () => {
      const result = schema.safeParse('null')
      expect(result.success).toBe(true)
    })

    it('should accept nested JSON', () => {
      const result = schema.safeParse('{"a": {"b": [1, 2, {"c": 3}]}}')
      expect(result.success).toBe(true)
    })

    it('should reject invalid JSON', () => {
      const result = schema.safeParse('{invalid}')
      expect(result.success).toBe(false)
    })

    it('should reject empty string', () => {
      const result = schema.safeParse('')
      expect(result.success).toBe(false)
    })

    it('should reject whitespace only', () => {
      const result = schema.safeParse('   ')
      expect(result.success).toBe(false)
    })

    it('should reject unclosed brace', () => {
      const result = schema.safeParse('{"key": "value"')
      expect(result.success).toBe(false)
    })

    it('should reject trailing comma', () => {
      const result = schema.safeParse('{"key": "value",}')
      expect(result.success).toBe(false)
    })

    it('should have correct error message', () => {
      const result = schema.safeParse('not json')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.issues[0].message).toContain('JSON')
      }
    })
  })

  // ==========================================================================
  // createConfigValueSchema - string type (default)
  // ==========================================================================

  describe('createConfigValueSchema - string', () => {
    const schema = createConfigValueSchema('string')

    it('should accept any string', () => {
      const result = schema.safeParse('hello world')
      expect(result.success).toBe(true)
    })

    it('should accept empty string', () => {
      const result = schema.safeParse('')
      expect(result.success).toBe(true)
    })

    it('should accept special characters', () => {
      const result = schema.safeParse('!@#$%^&*()')
      expect(result.success).toBe(true)
    })

    it('should accept unicode', () => {
      const result = schema.safeParse('你好世界')
      expect(result.success).toBe(true)
    })

    it('should accept multiline string', () => {
      const result = schema.safeParse('line1\nline2\nline3')
      expect(result.success).toBe(true)
    })
  })

  // ==========================================================================
  // createConfigFormSchema
  // ==========================================================================

  describe('createConfigFormSchema', () => {
    it('should create form schema for bool type', () => {
      const schema = createConfigFormSchema('bool')
      const result = schema.safeParse({ value: 'true' })
      expect(result.success).toBe(true)
    })

    it('should create form schema for int type', () => {
      const schema = createConfigFormSchema('int')
      const result = schema.safeParse({ value: '42' })
      expect(result.success).toBe(true)
    })

    it('should create form schema for json type', () => {
      const schema = createConfigFormSchema('json')
      const result = schema.safeParse({ value: '{"key": "value"}' })
      expect(result.success).toBe(true)
    })

    it('should create form schema for string type', () => {
      const schema = createConfigFormSchema('string')
      const result = schema.safeParse({ value: 'any string' })
      expect(result.success).toBe(true)
    })

    it('should reject invalid form structure', () => {
      const schema = createConfigFormSchema('bool')
      const result = schema.safeParse({ notValue: 'true' })
      expect(result.success).toBe(false)
    })

    it('should reject missing value field', () => {
      const schema = createConfigFormSchema('bool')
      const result = schema.safeParse({})
      expect(result.success).toBe(false)
    })
  })
})
