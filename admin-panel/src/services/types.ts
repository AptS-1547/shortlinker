export interface SerializableShortLink {
  short_code: string
  target_url: string
  /** RFC3339 格式的创建时间 */
  created_at: string
  /** RFC3339 格式的过期时间，null 表示永不过期 */
  expires_at: string | null
}

export interface LinkPayload {
  code?: string
  target: string
  /** RFC3339 格式的过期时间，例如: 2023-12-31T23:59:59Z */
  expires_at?: string | null
  force?: boolean
}

export interface AuthRequest {
  password: string
}

export interface AuthResponse {
  token: string
  expires_in?: number
}

export interface HealthResponse {
  status: string
  [key: string]: unknown
}

export interface BatchOperationResult {
  success: string[]
  failed: { code: string; error: string }[]
}

export interface QRCodeOptions {
  size?: number
  margin?: number
  errorCorrectionLevel?: 'L' | 'M' | 'Q' | 'H'
}

export interface LinkCreateResult {
  success: boolean
  exists?: boolean
  existingLink?: SerializableShortLink
}

export interface GetLinksQuery {
  page?: number
  page_size?: number
  created_after?: string
  created_before?: string
  only_expired?: boolean
  only_active?: boolean
  search?: string // 如果后端支持搜索功能的话
}

// 新的API响应格式
export interface PaginatedLinksResponse {
  code: number
  data: Record<string, SerializableShortLink>
  pagination: {
    page: number
    page_size: number
    total: number
    total_pages: number
  }
}
