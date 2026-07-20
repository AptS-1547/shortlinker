import { beforeEach, describe, expect, it } from 'vitest'
import {
  buildShortUrl,
  buildUrl,
  buildUrlParams,
  isValidUrl,
  normalizeUrl,
  parseUrlParams,
} from '../urlBuilder'

describe('urlBuilder', () => {
  // ==========================================================================
  // buildUrlParams
  // ==========================================================================

  describe('buildUrlParams', () => {
    it('should create URLSearchParams from object', () => {
      const params = buildUrlParams({ foo: 'bar', baz: 'qux' })
      expect(params.get('foo')).toBe('bar')
      expect(params.get('baz')).toBe('qux')
    })

    it('should skip undefined values', () => {
      const params = buildUrlParams({ foo: 'bar', baz: undefined })
      expect(params.get('foo')).toBe('bar')
      expect(params.has('baz')).toBe(false)
    })

    it('should skip null values', () => {
      const params = buildUrlParams({ foo: 'bar', baz: null })
      expect(params.get('foo')).toBe('bar')
      expect(params.has('baz')).toBe(false)
    })

    it('should skip empty string values', () => {
      const params = buildUrlParams({ foo: 'bar', baz: '' })
      expect(params.get('foo')).toBe('bar')
      expect(params.has('baz')).toBe(false)
    })

    it('should handle boolean values', () => {
      const params = buildUrlParams({ active: true, disabled: false })
      expect(params.get('active')).toBe('true')
      expect(params.get('disabled')).toBe('false')
    })

    it('should handle number values', () => {
      const params = buildUrlParams({ page: 1, size: 20, price: 99.99 })
      expect(params.get('page')).toBe('1')
      expect(params.get('size')).toBe('20')
      expect(params.get('price')).toBe('99.99')
    })

    it('should handle array values', () => {
      const params = buildUrlParams({ tags: ['a', 'b', 'c'] })
      expect(params.getAll('tags')).toEqual(['a', 'b', 'c'])
    })

    it('should skip empty values in array', () => {
      const params = buildUrlParams({ tags: ['a', '', null, 'b'] })
      expect(params.getAll('tags')).toEqual(['a', 'b'])
    })

    it('should JSON stringify objects', () => {
      const params = buildUrlParams({ filter: { status: 'active' } })
      expect(params.get('filter')).toBe('{"status":"active"}')
    })

    it('should handle empty object', () => {
      const params = buildUrlParams({})
      expect(params.toString()).toBe('')
    })
  })

  // ==========================================================================
  // buildUrl
  // ==========================================================================

  describe('buildUrl', () => {
    it('should build URL with base only', () => {
      expect(buildUrl('https://example.com')).toBe('https://example.com')
    })

    it('should build URL with path', () => {
      expect(buildUrl('https://example.com', '/api/v1')).toBe(
        'https://example.com/api/v1',
      )
    })

    it('should build URL with path without leading slash', () => {
      expect(buildUrl('https://example.com', 'api/v1')).toBe(
        'https://example.com/api/v1',
      )
    })

    it('should handle base URL with trailing slash', () => {
      expect(buildUrl('https://example.com/', '/api/v1')).toBe(
        'https://example.com/api/v1',
      )
    })

    it('should build URL with params', () => {
      const url = buildUrl('https://example.com', '/search', { q: 'test' })
      expect(url).toBe('https://example.com/search?q=test')
    })

    it('should build URL with multiple params', () => {
      const url = buildUrl('https://example.com', '/search', {
        q: 'test',
        page: 1,
      })
      expect(url).toContain('q=test')
      expect(url).toContain('page=1')
    })

    it('should skip empty params', () => {
      const url = buildUrl('https://example.com', '/search', {
        q: 'test',
        empty: null,
      })
      expect(url).toBe('https://example.com/search?q=test')
    })

    it('should handle empty path', () => {
      const url = buildUrl('https://example.com', '', { q: 'test' })
      expect(url).toBe('https://example.com?q=test')
    })

    it('should handle no params', () => {
      const url = buildUrl('https://example.com', '/api')
      expect(url).toBe('https://example.com/api')
    })
  })

  // ==========================================================================
  // parseUrlParams
  // ==========================================================================

  describe('parseUrlParams', () => {
    it('should parse single param', () => {
      const params = parseUrlParams('https://example.com?foo=bar')
      expect(params.foo).toBe('bar')
    })

    it('should parse multiple params', () => {
      const params = parseUrlParams('https://example.com?foo=bar&baz=qux')
      expect(params.foo).toBe('bar')
      expect(params.baz).toBe('qux')
    })

    it('should handle duplicate keys as array', () => {
      const params = parseUrlParams('https://example.com?tag=a&tag=b&tag=c')
      expect(params.tag).toEqual(['a', 'b', 'c'])
    })

    it('should handle URL without params', () => {
      const params = parseUrlParams('https://example.com')
      expect(Object.keys(params)).toHaveLength(0)
    })

    it('should handle invalid URL', () => {
      const params = parseUrlParams('not-a-url')
      expect(Object.keys(params)).toHaveLength(0)
    })

    it('should decode URL encoded values', () => {
      const params = parseUrlParams('https://example.com?q=hello%20world')
      expect(params.q).toBe('hello world')
    })
  })

  // ==========================================================================
  // isValidUrl
  // ==========================================================================

  describe('isValidUrl', () => {
    it('should return true for valid https URL', () => {
      expect(isValidUrl('https://example.com')).toBe(true)
    })

    it('should return true for valid http URL', () => {
      expect(isValidUrl('http://example.com')).toBe(true)
    })

    it('should return true for URL with path', () => {
      expect(isValidUrl('https://example.com/path/to/page')).toBe(true)
    })

    it('should return true for URL with query params', () => {
      expect(isValidUrl('https://example.com?foo=bar')).toBe(true)
    })

    it('should return true for URL with port', () => {
      expect(isValidUrl('https://example.com:8080')).toBe(true)
    })

    it('should return false for invalid URL', () => {
      expect(isValidUrl('not-a-url')).toBe(false)
    })

    it('should return false for empty string', () => {
      expect(isValidUrl('')).toBe(false)
    })

    it('should return true for data URL', () => {
      // Note: this checks URL validity, not safety
      expect(isValidUrl('data:text/html,test')).toBe(true)
    })
  })

  // ==========================================================================
  // normalizeUrl
  // ==========================================================================

  describe('normalizeUrl', () => {
    it('should add https:// to URL without protocol', () => {
      expect(normalizeUrl('example.com')).toBe('https://example.com')
    })

    it('should not modify URL with https://', () => {
      expect(normalizeUrl('https://example.com')).toBe('https://example.com')
    })

    it('should not modify URL with http://', () => {
      expect(normalizeUrl('http://example.com')).toBe('http://example.com')
    })

    it('should not modify URL with other protocols', () => {
      expect(normalizeUrl('ftp://files.example.com')).toBe(
        'ftp://files.example.com',
      )
    })

    it('should use custom default protocol', () => {
      expect(normalizeUrl('example.com', 'http')).toBe('http://example.com')
    })

    it('should return empty string for empty input', () => {
      expect(normalizeUrl('')).toBe('')
    })

    it('should handle URL with path', () => {
      expect(normalizeUrl('example.com/path')).toBe('https://example.com/path')
    })
  })

  // ==========================================================================
  // buildShortUrl
  // ==========================================================================

  describe('buildShortUrl', () => {
    beforeEach(() => {
      // Mock window.location.origin
      Object.defineProperty(window, 'location', {
        value: { origin: 'https://short.link' },
        writable: true,
      })
    })

    it('should build short URL with code', () => {
      expect(buildShortUrl('abc123')).toBe('https://short.link/abc123')
    })

    it('should handle code with path', () => {
      expect(buildShortUrl('folder/link')).toBe(
        'https://short.link/folder/link',
      )
    })

    it('should handle empty code', () => {
      expect(buildShortUrl('')).toBe('https://short.link/')
    })
  })
})
