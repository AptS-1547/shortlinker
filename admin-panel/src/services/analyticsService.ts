/**
 * Analytics API 服务
 */
import { appConfig } from '@/config/app'
import { ENDPOINTS } from './endpoints'
import { adminClient } from './http'
import type {
  AnalyticsQuery,
  DeviceAnalyticsResponse,
  GeoStats,
  LinkAnalytics,
  ReferrerStats,
  TopLink,
  TrendData,
} from './types.generated'

/**
 * API 响应包装类型
 */
interface ApiResponse<T> {
  code: number
  message: string
  data: T
}

/**
 * Analytics 服务类
 */
class AnalyticsService {
  /**
   * 获取点击趋势
   */
  async getTrends(params?: AnalyticsQuery): Promise<TrendData> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<ApiResponse<TrendData>>(
      `${ENDPOINTS.ANALYTICS.TRENDS}${query}`,
      { skipCache: true },
    )
    return response.data
  }

  /**
   * 获取热门链接
   */
  async getTopLinks(params?: AnalyticsQuery): Promise<TopLink[]> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<ApiResponse<TopLink[]>>(
      `${ENDPOINTS.ANALYTICS.TOP}${query}`,
      { skipCache: true },
    )
    return response.data
  }

  /**
   * 获取来源统计
   */
  async getReferrers(params?: AnalyticsQuery): Promise<ReferrerStats[]> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<ApiResponse<ReferrerStats[]>>(
      `${ENDPOINTS.ANALYTICS.REFERRERS}${query}`,
      { skipCache: true },
    )
    return response.data
  }

  /**
   * 获取地理位置分布
   */
  async getGeoStats(params?: AnalyticsQuery): Promise<GeoStats[]> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<ApiResponse<GeoStats[]>>(
      `${ENDPOINTS.ANALYTICS.GEO}${query}`,
      { skipCache: true },
    )
    return response.data
  }

  /**
   * 获取单链接详细统计
   */
  async getLinkAnalytics(
    code: string,
    params?: AnalyticsQuery,
  ): Promise<LinkAnalytics> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<ApiResponse<LinkAnalytics>>(
      `${ENDPOINTS.ANALYTICS.LINK(code)}${query}`,
      { skipCache: true },
    )
    return response.data
  }

  /**
   * 获取单链接设备分析
   */
  async getLinkDeviceStats(
    code: string,
    params?: AnalyticsQuery,
  ): Promise<DeviceAnalyticsResponse> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<
      ApiResponse<DeviceAnalyticsResponse>
    >(`${ENDPOINTS.ANALYTICS.LINK_DEVICES(code)}${query}`, { skipCache: true })
    return response.data
  }

  /**
   * 获取设备分析
   */
  async getDeviceStats(
    params?: AnalyticsQuery,
  ): Promise<DeviceAnalyticsResponse> {
    const query = this.buildQueryString(params)
    const response = await adminClient.get<
      ApiResponse<DeviceAnalyticsResponse>
    >(`${ENDPOINTS.ANALYTICS.DEVICES}${query}`, { skipCache: true })
    return response.data
  }

  /**
   * 导出报告 (返回 Blob)
   */
  async exportReport(params?: AnalyticsQuery): Promise<Blob> {
    const query = this.buildQueryString(params)
    // 直接使用 fetch 获取 blob
    const baseUrl = typeof window !== 'undefined' ? window.location.origin : ''
    const response = await fetch(
      `${baseUrl}${appConfig.adminRoutePrefix}${ENDPOINTS.ANALYTICS.EXPORT}${query}`,
      {
        credentials: 'include',
      },
    )
    if (!response.ok) {
      throw new Error('Export failed')
    }
    return response.blob()
  }

  /**
   * 构建查询字符串
   */
  private buildQueryString(params?: AnalyticsQuery): string {
    if (!params) return ''

    const searchParams = new URLSearchParams()
    if (params.start_date) searchParams.set('start_date', params.start_date)
    if (params.end_date) searchParams.set('end_date', params.end_date)
    if (params.group_by) searchParams.set('group_by', params.group_by)
    if (params.limit) searchParams.set('limit', params.limit.toString())

    const queryString = searchParams.toString()
    return queryString ? `?${queryString}` : ''
  }
}

export const analyticsService = new AnalyticsService()
export { AnalyticsService }
