/**
 * 全局类型声明
 */

interface RuntimeConfig {
  basePath?: string
  adminRoutePrefix?: string
  healthRoutePrefix?: string
  shortlinkerVersion?: string
}

declare global {
  interface Window {
    /**
     * 运行时注入的应用配置
     * 由后端（Rust）在 HTML 中注入
     */
    __APP_CONFIG__?: RuntimeConfig
  }
}

export {}
