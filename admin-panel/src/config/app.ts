/**
 * 应用配置管理
 * 支持从多个来源获取配置，优先级如下：
 * 1. 环境变量 (import.meta.env) - 仅开发模式
 * 2. 运行时注入配置 (window.__APP_CONFIG__)
 * 3. 默认值
 */

import { configLogger } from '@/utils/logger'

interface AppConfig {
  basePath: string
  adminRoutePrefix: string
  healthRoutePrefix: string
  shortlinkerVersion: string
  apiBaseUrl: string
}

/**
 * 从环境变量获取配置值
 */
function getEnvValue(key: string): string | undefined {
  const envKey = `VITE_${key.toUpperCase()}`
  const value = import.meta.env[envKey]
  return value !== undefined ? String(value) : undefined
}

/**
 * 从运行时注入配置获取值
 */
function getRuntimeValue(key: keyof AppConfig): string | undefined {
  if (typeof window === 'undefined') return undefined

  const config = window.__APP_CONFIG__
  if (!config) return undefined

  const value = config[key as keyof typeof config]
  // 检查是否是未替换的占位符
  if (
    value &&
    typeof value === 'string' &&
    value.startsWith('%') &&
    value.endsWith('%')
  ) {
    return undefined
  }

  return value
}

/**
 * 获取配置值，优先级：环境变量 > 运行时配置 > 默认值
 */
function getConfigValue(
  key: keyof AppConfig,
  defaultValue: string,
  envKey?: string,
): string {
  // 开发模式：优先使用环境变量
  if (import.meta.env.DEV) {
    const envValue = getEnvValue(envKey || key)
    if (envValue !== undefined) {
      configLogger.info(`Using ${key} from env:`, envValue)
      return envValue
    }
  }

  // 运行时注入配置（生产模式主要使用这个）
  const runtimeValue = getRuntimeValue(key)
  if (runtimeValue !== undefined) {
    configLogger.info(`Using ${key} from runtime:`, runtimeValue)
    return runtimeValue
  }

  // 默认值
  configLogger.info(`Using ${key} from default:`, defaultValue)
  return defaultValue
}

/**
 * 应用配置单例
 */
class Config implements AppConfig {
  private _basePath: string
  private _adminRoutePrefix: string
  private _healthRoutePrefix: string
  private _shortlinkerVersion: string
  private _apiBaseUrl: string

  constructor() {
    // 初始化所有配置
    this._basePath = getConfigValue('basePath', '/', 'BASE_URL')
    this._adminRoutePrefix = getConfigValue('adminRoutePrefix', '/admin')
    this._healthRoutePrefix = getConfigValue('healthRoutePrefix', '/health')
    this._shortlinkerVersion = getConfigValue(
      'shortlinkerVersion',
      'unknown',
      'VERSION',
    )
    this._apiBaseUrl = getConfigValue('apiBaseUrl', '', 'API_BASE_URL')

    // 打印配置信息（仅开发模式）
    if (import.meta.env.DEV) {
      configLogger.info('Application configuration loaded:', {
        basePath: this._basePath,
        adminRoutePrefix: this._adminRoutePrefix,
        healthRoutePrefix: this._healthRoutePrefix,
        shortlinkerVersion: this._shortlinkerVersion,
        apiBaseUrl: this._apiBaseUrl,
        mode: import.meta.env.MODE,
      })
    }
  }

  get basePath(): string {
    return this._basePath
  }

  get adminRoutePrefix(): string {
    return this._adminRoutePrefix
  }

  get healthRoutePrefix(): string {
    return this._healthRoutePrefix
  }

  get shortlinkerVersion(): string {
    return this._shortlinkerVersion
  }

  get apiBaseUrl(): string {
    return this._apiBaseUrl
  }

  /**
   * 重新加载配置（用于热更新或配置变更）
   */
  reload(): void {
    this._basePath = getConfigValue('basePath', '/', 'BASE_URL')
    this._adminRoutePrefix = getConfigValue('adminRoutePrefix', '/admin')
    this._healthRoutePrefix = getConfigValue('healthRoutePrefix', '/health')
    this._shortlinkerVersion = getConfigValue(
      'shortlinkerVersion',
      'unknown',
      'VERSION',
    )
    this._apiBaseUrl = getConfigValue('apiBaseUrl', '', 'API_BASE_URL')

    configLogger.info('Configuration reloaded')
  }
}

// 导出配置单例
export const appConfig = new Config()

// 导出类型
export type { AppConfig }
