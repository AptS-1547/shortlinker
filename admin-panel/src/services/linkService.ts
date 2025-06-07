import { adminClient } from './http'
import { ApiError } from './http'
import type { SerializableShortLink, LinkPayload, LinkCreateResult } from './types'

export class LinkService {
  /**
   * 获取所有链接
   */
  async fetchAll(): Promise<Record<string, SerializableShortLink>> {
    const response = await adminClient.get('/link')
    return response.data || {}
  }

  /**
   * 获取单个链接
   */
  async fetchOne(code: string): Promise<SerializableShortLink | null> {
    try {
      const response = await adminClient.get(`/link/${code}`)
      return response.data || null
    } catch (error) {
      if (error instanceof ApiError && error.status === 404) {
        return null
      }
      throw error
    }
  }

  /**
   * 创建链接
   */
  async create(payload: LinkPayload): Promise<void> {
    await adminClient.post('/link', payload)
  }

  /**
   * 创建链接（带重复检查）
   */
  async createWithCheck(payload: LinkPayload): Promise<LinkCreateResult> {
    if (payload.code && !payload.force) {
      const existingLink = await this.fetchOne(payload.code)
      if (existingLink) {
        return {
          success: false,
          exists: true,
          existingLink,
        }
      }
    }

    await this.create(payload)
    return { success: true }
  }

  /**
   * 更新链接
   */
  async update(code: string, payload: LinkPayload): Promise<void> {
    await adminClient.put(`/link/${code}`, payload)
  }

  /**
   * 删除链接
   */
  async delete(code: string): Promise<void> {
    await adminClient.delete(`/link/${code}`)
  }
}

export const linkService = new LinkService()
