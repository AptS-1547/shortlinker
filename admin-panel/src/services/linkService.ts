import { adminClient } from './http'
import { ApiError } from './http'
import type { SerializableShortLink, LinkPayload, LinkCreateResult, GetLinksQuery, PaginatedLinksResponse } from './types'

export class LinkService {
  /**
   * 获取所有链接（带筛选）
   */
  async fetchAll(query?: GetLinksQuery): Promise<Record<string, SerializableShortLink>> {
    const params = new URLSearchParams()

    if (query) {
      if (query.page !== undefined) params.append('page', query.page.toString())
      if (query.page_size !== undefined) params.append('page_size', query.page_size.toString())
      if (query.created_after) params.append('created_after', query.created_after)
      if (query.created_before) params.append('created_before', query.created_before)
      if (query.only_expired !== undefined) params.append('only_expired', query.only_expired.toString())
      if (query.only_active !== undefined) params.append('only_active', query.only_active.toString())
    }

    const url = params.toString() ? `/link?${params.toString()}` : '/link'
    const response = await adminClient.get(url)
    return response.data || {}
  }

  /**
   * 获取分页链接（如果后端支持分页响应）
   */
  async fetchPaginated(query?: GetLinksQuery): Promise<PaginatedLinksResponse> {
    const params = new URLSearchParams()

    if (query) {
      if (query.page !== undefined) params.append('page', query.page.toString())
      if (query.page_size !== undefined) params.append('page_size', query.page_size.toString())
      if (query.created_after) params.append('created_after', query.created_after)
      if (query.created_before) params.append('created_before', query.created_before)
      if (query.only_expired !== undefined) params.append('only_expired', query.only_expired.toString())
      if (query.only_active !== undefined) params.append('only_active', query.only_active.toString())
      if (query.search) params.append('search', query.search)
    }

    const url = params.toString() ? `/link?${params.toString()}` : '/link'
    const response = await adminClient.get(url)

    // 处理新的API响应格式: { code, data, pagination }
    if (response && response.data && response.pagination) {
      return {
        code: response.code || 0,
        data: response.data || {},
        pagination: response.pagination
      }
    }

    // 如果响应格式不正确，返回空数据
    return {
      code: 0,
      data: {},
      pagination: {
        page: 1,
        page_size: 10,
        total: 0,
        total_pages: 0
      }
    }
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
