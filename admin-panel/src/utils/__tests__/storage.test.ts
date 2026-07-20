import { beforeEach, describe, expect, it, vi } from 'vitest'
import { STORAGE_KEYS, Storage } from '../storage'

describe('Storage', () => {
  beforeEach(() => {
    // 清理 mock
    vi.clearAllMocks()
    // 重置 localStorage mock
    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem: vi.fn(),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn(),
        length: 0,
        key: vi.fn(),
      },
      writable: true,
    })
  })

  describe('STORAGE_KEYS', () => {
    it('should have expected keys', () => {
      expect(STORAGE_KEYS.LANGUAGE).toBe('preferred-language')
      expect(STORAGE_KEYS.THEME).toBe('theme')
      expect(STORAGE_KEYS.LINKS_PAGE_SIZE).toBe('links-page-size')
      expect(STORAGE_KEYS.LINKS_VISIBLE_COLUMNS).toBe('links-visible-columns')
    })
  })

  describe('get', () => {
    it('should return value from localStorage', () => {
      const mockValue = 'test-value'
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue(mockValue)

      const result = Storage.get('test-key')
      expect(result).toBe(mockValue)
      expect(window.localStorage.getItem).toHaveBeenCalledWith('test-key')
    })

    it('should return null when key does not exist', () => {
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue(null)

      const result = Storage.get('nonexistent')
      expect(result).toBeNull()
    })

    it('should return null and log error on exception', () => {
      vi.spyOn(window.localStorage, 'getItem').mockImplementation(() => {
        throw new Error('Storage error')
      })

      const result = Storage.get('test-key')
      expect(result).toBeNull()
    })
  })

  describe('set', () => {
    it('should set value in localStorage', () => {
      const setItemSpy = vi.spyOn(window.localStorage, 'setItem')

      const result = Storage.set('test-key', 'test-value')
      expect(result).toBe(true)
      expect(setItemSpy).toHaveBeenCalledWith('test-key', 'test-value')
    })

    it('should return false on exception', () => {
      vi.spyOn(window.localStorage, 'setItem').mockImplementation(() => {
        throw new Error('Storage full')
      })

      const result = Storage.set('test-key', 'test-value')
      expect(result).toBe(false)
    })
  })

  describe('remove', () => {
    it('should remove value from localStorage', () => {
      const removeItemSpy = vi.spyOn(window.localStorage, 'removeItem')

      const result = Storage.remove('test-key')
      expect(result).toBe(true)
      expect(removeItemSpy).toHaveBeenCalledWith('test-key')
    })

    it('should return false on exception', () => {
      vi.spyOn(window.localStorage, 'removeItem').mockImplementation(() => {
        throw new Error('Storage error')
      })

      const result = Storage.remove('test-key')
      expect(result).toBe(false)
    })
  })

  describe('getJSON', () => {
    it('should parse and return JSON value', () => {
      const testObj = { foo: 'bar', count: 42 }
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue(
        JSON.stringify(testObj),
      )

      const result = Storage.getJSON<typeof testObj>('test-key')
      expect(result).toEqual(testObj)
    })

    it('should return null for non-existent key', () => {
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue(null)

      const result = Storage.getJSON('nonexistent')
      expect(result).toBeNull()
    })

    it('should return null for invalid JSON', () => {
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue('invalid json {')

      const result = Storage.getJSON('test-key')
      expect(result).toBeNull()
    })

    it('should handle array values', () => {
      const testArr = [1, 2, 3, 'four']
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue(
        JSON.stringify(testArr),
      )

      const result = Storage.getJSON<typeof testArr>('test-key')
      expect(result).toEqual(testArr)
    })
  })

  describe('setJSON', () => {
    it('should stringify and store JSON value', () => {
      const setItemSpy = vi.spyOn(window.localStorage, 'setItem')
      const testObj = { foo: 'bar' }

      const result = Storage.setJSON('test-key', testObj)
      expect(result).toBe(true)
      expect(setItemSpy).toHaveBeenCalledWith(
        'test-key',
        JSON.stringify(testObj),
      )
    })

    it('should handle array values', () => {
      const setItemSpy = vi.spyOn(window.localStorage, 'setItem')
      const testArr = [1, 2, 3]

      const result = Storage.setJSON('test-key', testArr)
      expect(result).toBe(true)
      expect(setItemSpy).toHaveBeenCalledWith(
        'test-key',
        JSON.stringify(testArr),
      )
    })

    it('should handle primitive values', () => {
      const setItemSpy = vi.spyOn(window.localStorage, 'setItem')

      Storage.setJSON('string-key', 'hello')
      expect(setItemSpy).toHaveBeenCalledWith('string-key', '"hello"')

      Storage.setJSON('number-key', 42)
      expect(setItemSpy).toHaveBeenCalledWith('number-key', '42')

      Storage.setJSON('bool-key', true)
      expect(setItemSpy).toHaveBeenCalledWith('bool-key', 'true')
    })

    it('should return false for circular reference', () => {
      const circularObj: Record<string, unknown> = { foo: 'bar' }
      circularObj.self = circularObj // Create circular reference

      const result = Storage.setJSON('test-key', circularObj)
      expect(result).toBe(false)
    })
  })

  describe('has', () => {
    it('should return true when key exists', () => {
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue('some-value')

      expect(Storage.has('test-key')).toBe(true)
    })

    it('should return false when key does not exist', () => {
      vi.spyOn(window.localStorage, 'getItem').mockReturnValue(null)

      expect(Storage.has('nonexistent')).toBe(false)
    })
  })

  describe('clearAll', () => {
    it('should remove all known storage keys', () => {
      // Create fresh spy for this test
      const removeItemSpy = vi.fn()
      const setItemSpy = vi.fn()
      Object.defineProperty(window, 'localStorage', {
        value: {
          getItem: vi.fn(),
          setItem: setItemSpy,
          removeItem: removeItemSpy,
          clear: vi.fn(),
          length: 0,
          key: vi.fn(),
        },
        writable: true,
      })

      const result = Storage.clearAll()
      expect(result).toBe(true)

      // Verify each key was removed
      for (const key of Object.values(STORAGE_KEYS)) {
        expect(removeItemSpy).toHaveBeenCalledWith(key)
      }
    })

    it('should return false on exception', () => {
      vi.spyOn(window.localStorage, 'removeItem').mockImplementation(() => {
        throw new Error('Storage error')
      })

      const result = Storage.clearAll()
      expect(result).toBe(false)
    })
  })
})
