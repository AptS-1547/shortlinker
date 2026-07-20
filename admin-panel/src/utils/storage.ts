/**
 * localStorage 统一管理工具
 * 提供类型安全的存储操作，统一错误处理
 */

import { storageLogger } from './logger'

// 存储键常量
export const STORAGE_KEYS = {
  LANGUAGE: 'preferred-language',
  THEME: 'theme',
  LINKS_PAGE_SIZE: 'links-page-size',
  LINKS_VISIBLE_COLUMNS: 'links-visible-columns',
} as const

type StorageKey = (typeof STORAGE_KEYS)[keyof typeof STORAGE_KEYS] | string

/**
 * 检查 localStorage 是否可用
 */
function isStorageAvailable(): boolean {
  if (typeof window === 'undefined') return false
  try {
    const testKey = '__storage_test__'
    window.localStorage.setItem(testKey, testKey)
    window.localStorage.removeItem(testKey)
    return true
  } catch {
    return false
  }
}

/**
 * Storage 工具类
 * 统一管理 localStorage 操作，提供错误处理和类型安全
 */
export const Storage = {
  /**
   * 获取字符串值
   */
  get(key: StorageKey): string | null {
    if (!isStorageAvailable()) return null
    try {
      return localStorage.getItem(key)
    } catch (error) {
      storageLogger.error(`Failed to get "${key}":`, error)
      return null
    }
  },

  /**
   * 设置字符串值
   */
  set(key: StorageKey, value: string): boolean {
    if (!isStorageAvailable()) return false
    try {
      localStorage.setItem(key, value)
      return true
    } catch (error) {
      storageLogger.error(`Failed to set "${key}":`, error)
      return false
    }
  },

  /**
   * 移除值
   */
  remove(key: StorageKey): boolean {
    if (!isStorageAvailable()) return false
    try {
      localStorage.removeItem(key)
      return true
    } catch (error) {
      storageLogger.error(`Failed to remove "${key}":`, error)
      return false
    }
  },

  /**
   * 获取 JSON 值
   */
  getJSON<T>(key: StorageKey): T | null {
    const value = this.get(key)
    if (value === null) return null
    try {
      return JSON.parse(value) as T
    } catch (error) {
      storageLogger.error(`Failed to parse JSON for "${key}":`, error)
      return null
    }
  },

  /**
   * 设置 JSON 值
   */
  setJSON(key: StorageKey, value: unknown): boolean {
    try {
      return this.set(key, JSON.stringify(value))
    } catch (error) {
      storageLogger.error(`Failed to stringify JSON for "${key}":`, error)
      return false
    }
  },

  /**
   * 检查键是否存在
   */
  has(key: StorageKey): boolean {
    return this.get(key) !== null
  },

  /**
   * 清除所有应用相关的存储
   */
  clearAll(): boolean {
    if (!isStorageAvailable()) return false
    try {
      for (const key of Object.values(STORAGE_KEYS)) {
        localStorage.removeItem(key)
      }
      return true
    } catch (error) {
      storageLogger.error('Failed to clear all:', error)
      return false
    }
  },
}
