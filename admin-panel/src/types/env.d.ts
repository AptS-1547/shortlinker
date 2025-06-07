interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_ADMIN_ROUTE_PREFIX: string
  readonly VITE_HEALTH_ROUTE_PREFIX: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
