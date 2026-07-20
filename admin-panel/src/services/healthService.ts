import { healthLogger } from '@/utils/logger'
import { healthClient } from './http'
import type { HealthResponse } from './types'

interface HealthApiResponse {
  code: number
  data: HealthResponse
}

export class HealthService {
  /**
   * 检查服务健康状态
   */
  async check(): Promise<HealthResponse> {
    try {
      const response = await healthClient.get<HealthApiResponse>('/health')
      // 解包 { code, data } 格式，返回 data 部分
      return response.data
    } catch (error) {
      healthLogger.error('Health check failed:', error)
      throw error
    }
  }
}

export const healthService = new HealthService()
