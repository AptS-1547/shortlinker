import { ENDPOINTS } from './endpoints'
import { ApiError, adminClient } from './http'
import type {
  GetLinksQuery,
  LinkCreateResult,
  LinkResponse,
  PaginatedLinksResponse,
  PostNewLink,
  StatsResponse,
} from './types'

/**
 * 构建链接查询参数
 */
function buildLinkQueryParams(query?: GetLinksQuery): URLSearchParams {
  const params = new URLSearchParams()

  if (query) {
    if (query.page != null) params.append('page', query.page.toString())
    if (query.page_size != null)
      params.append('page_size', query.page_size.toString())
    if (query.created_after) params.append('created_after', query.created_after)
    if (query.created_before)
      params.append('created_before', query.created_before)
    if (query.only_expired != null)
      params.append('only_expired', query.only_expired.toString())
    if (query.only_active != null)
      params.append('only_active', query.only_active.toString())
    if (query.search) params.append('search', query.search)
  }

  return params
}

/**
 * 构建带查询参数的 URL
 */
function buildLinkUrl(query?: GetLinksQuery): string {
  const params = buildLinkQueryParams(query)
  return params.toString()
    ? `${ENDPOINTS.LINKS.LIST}?${params.toString()}`
    : ENDPOINTS.LINKS.LIST
}

export class LinkService {
  /**
   * 获取所有链接（带筛选）
   */
  async fetchAll(query?: GetLinksQuery): Promise<LinkResponse[]> {
    const url = buildLinkUrl(query)
    const response = await adminClient.get<{
      code?: number
      message?: string
      data?: LinkResponse[]
      pagination?: PaginatedLinksResponse['pagination']
    }>(url)

    // 处理 API 响应格式: { code, message, data: [...], pagination }
    if (response?.data && Array.isArray(response.data)) {
      return response.data
    }

    return []
  }

  /**
   * 获取分页链接（如果后端支持分页响应）
   */
  async fetchPaginated(
    query?: GetLinksQuery,
    signal?: AbortSignal,
  ): Promise<PaginatedLinksResponse> {
    const url = buildLinkUrl(query)
    const response = await adminClient.get<{
      code?: number
      message?: string
      data?: LinkResponse[]
      pagination?: PaginatedLinksResponse['pagination']
    }>(url, { signal })

    // 处理 API 响应格式: { code, message, data: [...], pagination }
    if (response?.data && response.pagination) {
      return {
        code: response.code || 0,
        message: response.message || '',
        data: Array.isArray(response.data) ? response.data : [],
        pagination: response.pagination,
      }
    }

    // 如果响应格式不正确，返回空数据
    return {
      code: 0,
      message: '',
      data: [],
      pagination: {
        page: 1,
        page_size: 10,
        total: 0,
        total_pages: 0,
      },
    }
  }

  /**
   * 获取单个链接
   */
  async fetchOne(code: string): Promise<LinkResponse | null> {
    try {
      const response = await adminClient.get<{ data?: LinkResponse }>(
        ENDPOINTS.LINKS.SINGLE(code),
      )
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
  async create(payload: PostNewLink): Promise<void> {
    await adminClient.post(ENDPOINTS.LINKS.LIST, payload)
    // 使用标签清理（更精细）
    adminClient.invalidateTags(['links-list', 'stats'])
  }

  /**
   * 创建链接（带重复检查）
   * 利用后端的 force 逻辑：如果链接已存在且 force != true，后端返回 409 Conflict
   */
  async createWithCheck(payload: PostNewLink): Promise<LinkCreateResult> {
    try {
      await this.create(payload)
      return { success: true }
    } catch (error) {
      // 后端返回 409 Conflict 表示链接已存在
      if (error instanceof ApiError && error.status === 409) {
        const existingLink = payload.code
          ? await this.fetchOne(payload.code)
          : null
        return {
          success: false,
          exists: true,
          existingLink: existingLink || undefined,
        }
      }
      throw error
    }
  }

  /**
   * 更新链接
   */
  async update(code: string, payload: PostNewLink): Promise<void> {
    await adminClient.put(ENDPOINTS.LINKS.SINGLE(code), payload)
    // 清除该链接 + 列表缓存 + 统计缓存
    adminClient.invalidateTags([`link:${code}`, 'links-list', 'stats'])
  }

  /**
   * 删除链接
   */
  async delete(code: string): Promise<void> {
    await adminClient.delete(ENDPOINTS.LINKS.SINGLE(code))
    // 清除该链接 + 列表缓存 + 统计缓存
    adminClient.invalidateTags([`link:${code}`, 'links-list', 'stats'])
  }

  /**
   * 获取链接统计信息
   */
  async fetchStats(): Promise<StatsResponse> {
    const response = await adminClient.get<{
      code?: number
      data?: StatsResponse
    }>(ENDPOINTS.STATS)
    return (
      response.data || {
        total_links: 0,
        total_clicks: 0,
        active_links: 0,
      }
    )
  }
}

export const linkService = new LinkService()
