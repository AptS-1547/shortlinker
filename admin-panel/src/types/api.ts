export interface ShortLink {
  code: string
  url: string
  expires_at?: string
  created_at: string
  updated_at: string
}

export interface CreateLinkRequest {
  code?: string
  url: string
  expires_at?: string
}

export interface UpdateLinkRequest {
  url?: string
  expires_at?: string
}

export interface HealthStatus {
  status: 'healthy' | 'unhealthy'
  timestamp: string
}

export interface ApiError {
  error: string
  message?: string
}
