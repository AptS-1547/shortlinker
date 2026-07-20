// ============ 从生成文件重新导出 ============

export type {
  BatchCreateRequest,
  BatchDeleteRequest,
  BatchFailedItem,
  BatchResponse,
  BatchUpdateItem,
  BatchUpdateRequest,
  ConfigHistoryResponse,
  ConfigItemResponse,
  ConfigUpdateRequest,
  ConfigUpdateResponse,
  GetLinksQuery,
  HealthChecks,
  HealthResponse,
  HealthStorageBackend,
  HealthStorageCheck,
  ImportFailedItem,
  ImportMode,
  ImportResponse,
  LinkResponse,
  LoginCredentials,
  PaginationInfo,
  PostNewLink,
  StatsResponse,
  ValueType,
} from './types.generated'
export { ErrorCode } from './types.generated'

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
  existingLink?: import('./types.generated').LinkResponse
}

// 分页响应包装（组合 PaginationInfo）
export interface PaginatedLinksResponse {
  code: number
  message: string
  data: import('./types.generated').LinkResponse[]
  pagination: import('./types.generated').PaginationInfo
}
