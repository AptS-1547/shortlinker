import { authLogger } from '@/utils/logger'
import { ENDPOINTS } from './endpoints'
import { adminClient } from './http'
import type { LoginCredentials } from './types'
import type { AuthSuccessResponse } from './types.generated'

export interface AuthResult {
  expiresIn: number
}

export class AuthService {
  /**
   * 用户登录
   * 后端会设置 httpOnly Cookie，前端不需要处理 token
   * 返回 expires_in（秒）
   */
  async login(credentials: LoginCredentials): Promise<AuthResult> {
    const response = await adminClient.post<{
      code: number
      data: AuthSuccessResponse
    }>(ENDPOINTS.AUTH.LOGIN, credentials)
    return {
      expiresIn: Number(response.data?.expires_in) || 900,
    }
  }

  /**
   * 用户登出
   * 后端会清除 HttpOnly Cookie
   */
  async logout(): Promise<void> {
    try {
      await adminClient.post(ENDPOINTS.AUTH.LOGOUT, {})
    } catch (error) {
      authLogger.error('Logout API failed:', error)
      throw error
    }
  }

  /**
   * 刷新 Token
   * 后端会验证 refresh cookie 并生成新的 access/refresh tokens
   * 返回 expires_in（秒）
   */
  async refreshToken(): Promise<AuthResult> {
    const response = await adminClient.post<{
      code: number
      data: AuthSuccessResponse
    }>(ENDPOINTS.AUTH.REFRESH, {})
    return {
      expiresIn: Number(response.data?.expires_in) || 900,
    }
  }

  /**
   * 验证 Token（通过 Cookie）
   * 返回是否已认证
   */
  async verifyToken(): Promise<boolean> {
    try {
      // 认证验证必须实时，跳过缓存
      await adminClient.get(ENDPOINTS.AUTH.VERIFY, { skipCache: true })
      return true
    } catch {
      // 401 或其他错误都视为未认证
      return false
    }
  }
}

export const authService = new AuthService()
