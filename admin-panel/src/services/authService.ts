import { adminClient } from './http'
import { ApiError } from './http'
import type { AuthRequest, AuthResponse } from './types'

export class AuthService {
  /**
   * 用户登录
   */
  async login(credentials: AuthRequest): Promise<AuthResponse> {
    const response = await adminClient.post('/auth/login', credentials)
    return {
      token: credentials.password,
      expires_in: response.expires_in,
    }
  }

  /**
   * 验证 Token
   */
  async verifyToken(): Promise<boolean> {
    try {
      await adminClient.get('/auth/verify')
      return true
    } catch (error) {
      console.error('Token verification failed:', error)
      return false
    }
  }
}

export const authService = new AuthService()
