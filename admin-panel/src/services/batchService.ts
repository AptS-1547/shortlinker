import axios from 'axios'
import { appConfig } from '@/config/app'
import { ENDPOINTS } from './endpoints'
import { adminClient } from './http'
import type {
  BatchCreateRequest,
  BatchDeleteRequest,
  BatchResponse,
  BatchUpdateRequest,
  GetLinksQuery,
  ImportMode,
  ImportResponse,
  PostNewLink,
} from './types'

interface ApiResponse<T> {
  code: number
  data: T
}

export class BatchService {
  /**
   * 批量创建链接
   */
  async createLinks(links: PostNewLink[]): Promise<BatchResponse> {
    const response = await adminClient.post<ApiResponse<BatchResponse>>(
      ENDPOINTS.LINKS.BATCH,
      { links } satisfies BatchCreateRequest,
    )

    // 清除链接列表和统计缓存
    adminClient.invalidateTags(['links-list', 'stats'])

    return response.data
  }

  /**
   * 批量更新链接
   */
  async updateLinks(
    updates: { code: string; payload: PostNewLink }[],
  ): Promise<BatchResponse> {
    const response = await adminClient.put<ApiResponse<BatchResponse>>(
      ENDPOINTS.LINKS.BATCH,
      { updates } satisfies BatchUpdateRequest,
    )

    // 清除链接列表 + 被更新的链接缓存 + 统计缓存
    const linkTags = updates.map((u) => `link:${u.code}`)
    adminClient.invalidateTags(['links-list', 'stats', ...linkTags])

    return response.data
  }

  /**
   * 批量删除链接
   */
  async deleteLinks(codes: string[]): Promise<BatchResponse> {
    const response = await adminClient.delete<ApiResponse<BatchResponse>>(
      ENDPOINTS.LINKS.BATCH,
      { codes } satisfies BatchDeleteRequest,
    )

    // 清除链接列表 + 被删除的链接缓存 + 统计缓存
    const linkTags = codes.map((code) => `link:${code}`)
    adminClient.invalidateTags(['links-list', 'stats', ...linkTags])

    return response.data
  }

  /**
   * 导出链接为 CSV
   *
   * 使用隐藏 iframe 触发浏览器原生下载，避免内存缓冲和页面闪烁
   */
  async exportLinks(query?: Partial<GetLinksQuery>): Promise<void> {
    // 1. 构建查询参数
    const params = new URLSearchParams()
    if (query?.search) params.set('search', query.search)
    if (query?.created_after) params.set('created_after', query.created_after)
    if (query?.created_before)
      params.set('created_before', query.created_before)
    if (query?.only_expired) params.set('only_expired', 'true')
    if (query?.only_active) params.set('only_active', 'true')

    const queryString = params.toString()

    // 2. 预检：确保认证有效（使用 /auth/verify 快速检查）
    // 如果 token 过期，adminClient 会自动刷新
    await adminClient.get(ENDPOINTS.AUTH.VERIFY)

    // 3. 构建完整 URL
    const baseUrl = import.meta.env.PROD
      ? window.location.origin
      : appConfig.apiBaseUrl || 'http://127.0.0.1:8080'
    const exportUrl = `${baseUrl}${appConfig.adminRoutePrefix}${ENDPOINTS.LINKS.EXPORT}${queryString ? `?${queryString}` : ''}`

    // 4. 使用隐藏 iframe 触发下载（完全无感，无闪烁）
    const iframe = document.createElement('iframe')
    iframe.style.display = 'none'
    iframe.src = exportUrl

    // 5. 注入 DOM 触发请求
    document.body.appendChild(iframe)

    // 6. 延迟清理 iframe（60 秒后删除）
    // 注意：不能监听 onload，因为 Content-Disposition: attachment 不触发 load 事件
    setTimeout(() => {
      if (document.body.contains(iframe)) {
        document.body.removeChild(iframe)
      }
    }, 60000)
  }

  /**
   * 从 CSV 导入链接
   */
  async importLinks(
    file: File,
    mode: ImportMode,
    onProgress?: (percent: number) => void,
  ): Promise<ImportResponse> {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('mode', mode)

    // 使用 axios 直接请求以发送 multipart/form-data
    const baseUrl = import.meta.env.PROD
      ? window.location.origin
      : appConfig.apiBaseUrl || 'http://127.0.0.1:8080'

    const response = await axios.post<ApiResponse<ImportResponse>>(
      `${baseUrl}${appConfig.adminRoutePrefix}${ENDPOINTS.LINKS.IMPORT}`,
      formData,
      {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
        withCredentials: true,
        onUploadProgress: onProgress
          ? (progressEvent) => {
              if (progressEvent.total) {
                const percent = Math.round(
                  (progressEvent.loaded * 100) / progressEvent.total,
                )
                onProgress(percent)
              }
            }
          : undefined,
      },
    )

    // 清除链接列表和统计缓存
    adminClient.invalidateTags(['links-list', 'stats'])

    return response.data.data
  }
}

export const batchService = new BatchService()
