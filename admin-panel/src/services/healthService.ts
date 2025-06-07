import { healthClient } from './http'
import type { HealthResponse } from './types'

export class HealthService {
  /**
   * 检查服务健康状态
   */
  async check(): Promise<HealthResponse> {
    try {
      return await healthClient.get('')
    } catch (error) {
      console.error('Health check failed:', error)
      throw error
    }
  }
}

export const healthService = new HealthService()
