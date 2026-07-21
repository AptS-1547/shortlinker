// ============ OpenAPI 生成类型的稳定导出入口 ============

import type { components } from './api.generated'

export { ErrorCode } from './api.generated'

export type ActionType = components['schemas']['ActionType']
export type AnalyticsQuery = components['schemas']['AnalyticsQuery']
export type AuthSuccessResponse = components['schemas']['AuthSuccessResponse']
export type BatchCreateRequest = components['schemas']['BatchCreateRequest']
export type BatchDeleteRequest = components['schemas']['BatchDeleteRequest']
export type BatchFailedItem = components['schemas']['BatchFailedItem']
export type BatchResponse = components['schemas']['BatchResponse']
export type BatchUpdateItem = components['schemas']['BatchUpdateItem']
export type BatchUpdateRequest = components['schemas']['BatchUpdateRequest']
export type CategoryStatsResponse =
  components['schemas']['CategoryStatsResponse']
export type ConfigActionRequest = components['schemas']['ConfigActionRequest']
export type ConfigActionResponse = components['schemas']['ConfigActionResponse']
export type ConfigHistoryResponse =
  components['schemas']['ConfigHistoryResponse']
export type ConfigItemResponse = components['schemas']['ConfigItemResponse']
export type ConfigSchema = components['schemas']['ConfigSchema']
export type ConfigUpdateRequest = components['schemas']['ConfigUpdateRequest']
export type ConfigUpdateResponse = components['schemas']['ConfigUpdateResponse']
export type DeviceAnalyticsResponse =
  components['schemas']['DeviceAnalyticsResponse']
export type EnumOption = components['schemas']['EnumOption']
export type ExecuteAndSaveResponse =
  components['schemas']['ExecuteAndSaveResponse']
export type ExportQuery = components['schemas']['ExportQuery']
export type GeoStats = components['schemas']['GeoStats']
export type GetLinksQuery = components['schemas']['GetLinksQuery']
export type GroupBy = components['schemas']['GroupBy']
export type HealthCacheCheck = components['schemas']['HealthCacheCheck']
export type HealthChecks = components['schemas']['HealthChecks']
export type HealthResponse = components['schemas']['HealthResponse']
export type HealthStorageBackend = components['schemas']['HealthStorageBackend']
export type HealthStorageCheck = components['schemas']['HealthStorageCheck']
export type HttpMethod = components['schemas']['HttpMethod']
export type ImportFailedItem = components['schemas']['ImportFailedItem']
export type ImportMode = components['schemas']['ImportMode']
export type ImportResponse = components['schemas']['ImportResponse']
export type LinkAnalytics = components['schemas']['LinkAnalytics']
export type LinkResponse = components['schemas']['LinkResponse']
export type LoginCredentials = components['schemas']['LoginCredentials']
export type MessageResponse = components['schemas']['MessageResponse']
export type PaginationInfo = components['schemas']['PaginationInfo']
export type PostNewLink = components['schemas']['PostNewLink']
export type ReferrerStats = components['schemas']['ReferrerStats']
export type ReloadResponse = components['schemas']['ReloadResponse']
export type SameSitePolicy = components['schemas']['SameSitePolicy']
export type StatsResponse = components['schemas']['StatsResponse']
export type TopLink = components['schemas']['TopLink']
export type TrendData = components['schemas']['TrendData']
export type ValueType = components['schemas']['ValueType']

// ============ 前端专用类型（保留） ============

export interface QRCodeOptions {
  size?: number
  margin?: number
  errorCorrectionLevel?: 'L' | 'M' | 'Q' | 'H'
  color?: {
    dark?: string
    light?: string
  }
}

export interface LinkCreateResult {
  success: boolean
  exists?: boolean
  existingLink?: LinkResponse
}

// 分页响应包装（组合 PaginationInfo）
export interface PaginatedLinksResponse {
  code: number
  message: string
  data: LinkResponse[]
  pagination: PaginationInfo
}
