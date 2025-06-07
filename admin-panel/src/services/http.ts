import axios, { type AxiosInstance, type AxiosError } from 'axios'

// ==================== 配置管理 ====================
export class ApiConfig {
  private static instance: ApiConfig

  public readonly baseUrl: string
  public readonly adminRoutePrefix: string
  public readonly healthRoutePrefix: string

  private constructor() {
    this.baseUrl = this.getBaseUrl()
    this.adminRoutePrefix = this.getEnvValue('ADMIN_ROUTE_PREFIX', '/admin')
    this.healthRoutePrefix = this.getEnvValue('HEALTH_ROUTE_PREFIX', '/health')
  }

  static getInstance(): ApiConfig {
    if (!ApiConfig.instance) {
      ApiConfig.instance = new ApiConfig()
    }
    return ApiConfig.instance
  }

  private getEnvValue(key: string, defaultValue: string = ''): string {
    return import.meta.env[`VITE_${key}`] || defaultValue
  }

  private getBaseUrl(): string {
    if (import.meta.env.PROD) {
      return typeof window !== 'undefined' ? window.location.origin : ''
    }
    return this.getEnvValue('API_BASE_URL', 'http://127.0.0.1:8080')
  }
}

// ==================== 错误处理 ====================
export class ApiError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

const handleAuthError = (): void => {
  if (typeof window !== 'undefined') {
    localStorage.removeItem('adminToken')
    window.location.href = '/login'
  }
}

export const createErrorHandler = (context: string) => {
  return (error: unknown): never => {
    if (axios.isAxiosError(error)) {
      const axiosError = error as AxiosError

      // 网络错误
      if (axiosError.code === 'ECONNREFUSED' || axiosError.code === 'ERR_NETWORK') {
        throw new ApiError('Network Error: Cannot connect to server', undefined, 'NETWORK_ERROR')
      }

      // HTTP 状态码错误
      const status = axiosError.response?.status
      switch (status) {
        case 401:
          // 如果是登录接口，不要自动重定向
          if (axiosError.config?.url?.includes('/auth/login')) {
            throw new ApiError('Invalid credentials', 401, 'INVALID_CREDENTIALS')
          }
          handleAuthError()
          throw new ApiError(
            `401: Unauthorized access${context ? ` to ${context}` : ''}`,
            401,
            'UNAUTHORIZED',
          )
        case 404:
          throw new ApiError(`404: ${context || 'Endpoint'} not found`, 404, 'NOT_FOUND')
        case 500:
          throw new ApiError(
            `500: ${context ? `${context} failed` : 'Internal server error'}`,
            500,
            'SERVER_ERROR',
          )
        default:
          throw new ApiError(
            `${status}: ${axiosError.response?.statusText || 'Unknown error'}`,
            status,
            'HTTP_ERROR',
          )
      }
    }

    throw error instanceof Error ? error : new ApiError('Unknown error occurred')
  }
}

// ==================== HTTP 客户端 ====================
export class HttpClient {
  private client: AxiosInstance

  constructor(baseURL: string, context: string = '') {
    this.client = axios.create({
      baseURL,
      withCredentials: true,
    })

    this.setupInterceptors(context)
  }

  private setupInterceptors(context: string): void {
    // 请求拦截器
    this.client.interceptors.request.use((config) => {
      if (typeof window !== 'undefined') {
        const token = localStorage.getItem('adminToken')
        if (token) {
          config.headers.Authorization = `Bearer ${token}`
        }
      }
      return config
    })

    // 响应拦截器
    this.client.interceptors.response.use((response) => response, createErrorHandler(context))
  }

  async get<T = any>(url: string): Promise<T> {
    const response = await this.client.get(url)
    return response.data
  }

  async post<T = any>(url: string, data?: any): Promise<T> {
    const response = await this.client.post(url, data)
    return response.data
  }

  async put<T = any>(url: string, data?: any): Promise<T> {
    const response = await this.client.put(url, data)
    return response.data
  }

  async delete<T = any>(url: string): Promise<T> {
    const response = await this.client.delete(url)
    return response.data
  }
}

// ==================== 客户端实例 ====================
export const config = ApiConfig.getInstance()
export const adminClient = new HttpClient(`${config.baseUrl}${config.adminRoutePrefix}`, 'admin')
export const healthClient = new HttpClient(`${config.baseUrl}${config.healthRoutePrefix}`, 'health')
