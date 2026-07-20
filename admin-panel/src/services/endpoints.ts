/**
 * API 端点集中管理
 *
 * 所有 API 路径在此定义，改版本号或路径只需改一处。
 */

const V1 = '/v1'

export const ENDPOINTS = {
  AUTH: {
    LOGIN: `${V1}/auth/login`,
    LOGOUT: `${V1}/auth/logout`,
    REFRESH: `${V1}/auth/refresh`,
    VERIFY: `${V1}/auth/verify`,
  },
  LINKS: {
    LIST: `${V1}/links`,
    SINGLE: (code: string) => `${V1}/links/${code}`,
    BATCH: `${V1}/links/batch`,
    EXPORT: `${V1}/links/export`,
    IMPORT: `${V1}/links/import`,
  },
  STATS: `${V1}/stats`,
  CONFIG: {
    LIST: `${V1}/config`,
    SINGLE: (key: string) => `${V1}/config/${encodeURIComponent(key)}`,
    HISTORY: (key: string) => `${V1}/config/${encodeURIComponent(key)}/history`,
    ACTION: (key: string) => `${V1}/config/${encodeURIComponent(key)}/action`,
    EXECUTE_AND_SAVE: (key: string) =>
      `${V1}/config/${encodeURIComponent(key)}/execute-and-save`,
    RELOAD: `${V1}/config/reload`,
    SCHEMA: `${V1}/config/schema`,
  },
  ANALYTICS: {
    TRENDS: `${V1}/analytics/trends`,
    TOP: `${V1}/analytics/top`,
    REFERRERS: `${V1}/analytics/referrers`,
    GEO: `${V1}/analytics/geo`,
    LINK: (code: string) => `${V1}/links/${code}/analytics`,
    LINK_DEVICES: (code: string) => `${V1}/links/${code}/analytics/devices`,
    EXPORT: `${V1}/analytics/export`,
    DEVICES: `${V1}/analytics/devices`,
  },
} as const

/**
 * 缓存策略用的路径模式（带参数占位符）
 */
export const ENDPOINT_PATTERNS = {
  LINKS_SINGLE: `${V1}/links/:code`,
  CONFIG_SINGLE: `${V1}/config/:key`,
  CONFIG_HISTORY: `${V1}/config/:key/history`,
} as const
