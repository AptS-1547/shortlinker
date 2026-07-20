import { lazy, Suspense, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiAlertTriangle as AlertTriangle,
  FiHome as Home,
  FiRefreshCw as RefreshCw,
} from 'react-icons/fi'
import {
  createBrowserRouter,
  Navigate,
  Outlet,
  useLocation,
  useRouteError,
} from 'react-router-dom'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { useAuthStore } from '@/stores/authStore'
import { routerLogger } from '@/utils/logger'

// 从运行时配置或环境变量获取基础路径
const getBasePath = () => {
  // 确保在浏览器环境中运行
  if (typeof window === 'undefined') {
    return import.meta.env.BASE_URL || '/'
  }

  // 优先从 window 对象获取（Rust 可以注入）
  if (window.__APP_CONFIG__) {
    const config = window.__APP_CONFIG__
    if (config.basePath && config.basePath !== '%BASE_PATH%') {
      routerLogger.warn('Using base path from Rust config:', config.basePath)
      return config.basePath
    }
  }

  // 其次从 meta 标签获取
  const metaBase = document
    .querySelector('meta[name="base-path"]')
    ?.getAttribute('content')
  if (metaBase && metaBase !== '%BASE_PATH%') {
    routerLogger.warn('Using base path from meta tag:', metaBase)
    return metaBase
  }

  // 最后使用构建时的配置
  routerLogger.warn(
    'Using base path from import.meta.env:',
    import.meta.env.BASE_URL,
  )
  return import.meta.env.BASE_URL || '/'
}

const basePath = getBasePath()

// 路由懒加载
const LoginPage = lazy(() => import('@/pages/LoginPage'))
const AdminLayout = lazy(() => import('@/components/layout/AdminLayout'))
const DashboardPage = lazy(() => import('@/pages/DashboardPage'))
const LinksPage = lazy(() => import('@/pages/LinksPage'))
const LinkDetailPage = lazy(() => import('@/pages/LinkDetailPage'))
const AnalyticsPage = lazy(() => import('@/pages/AnalyticsPage'))
const SettingsPage = lazy(() => import('@/pages/SettingsPage'))

// 加载占位
function LoadingFallback() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
    </div>
  )
}

// 路由错误边界
function RouteErrorBoundary() {
  const error = useRouteError() as Error
  const { t } = useTranslation()

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
            <AlertTriangle className="h-6 w-6 text-destructive" />
          </div>
          <CardTitle>{t('errors.boundary.title')}</CardTitle>
          <CardDescription>{t('errors.boundary.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <details className="text-sm text-muted-foreground">
              <summary className="cursor-pointer font-medium">
                {t('errors.boundary.technicalDetails')}
              </summary>
              <pre className="mt-2 overflow-auto rounded bg-muted p-2 text-xs">
                {error.message || String(error)}
              </pre>
            </details>
          )}
          <div className="flex gap-2">
            <Button onClick={() => window.location.reload()} className="flex-1">
              <RefreshCw className="mr-2 h-4 w-4" />
              {t('errors.boundary.reload')}
            </Button>
            <Button
              variant="outline"
              onClick={() => {
                window.location.href = '/'
              }}
              className="flex-1"
            >
              <Home className="mr-2 h-4 w-4" />
              {t('errors.boundary.goHome')}
            </Button>
          </div>
          <p className="text-xs text-center text-muted-foreground">
            {t('errors.boundary.persistentIssue')}
          </p>
        </CardContent>
      </Card>
    </div>
  )
}

// 懒加载包装组件
function LazyComponent({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<LoadingFallback />}>{children}</Suspense>
}

// 受保护路由组件
function ProtectedRoute() {
  const location = useLocation()
  const { isAuthenticated, isChecking, hasChecked, checkAuthStatus } =
    useAuthStore()

  useEffect(() => {
    // Only check if not already checked and not currently checking
    if (!hasChecked && !isChecking) {
      checkAuthStatus()
    }
  }, [hasChecked, isChecking, checkAuthStatus])

  // Show loading while checking auth status
  if (!hasChecked || isChecking) {
    return <LoadingFallback />
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  return <Outlet />
}

// 登录页路由守卫（已登录则重定向）
function LoginGuard() {
  const { isAuthenticated, isChecking, hasChecked, checkAuthStatus } =
    useAuthStore()

  useEffect(() => {
    // Check auth status on mount if not checked yet
    if (!hasChecked && !isChecking) {
      checkAuthStatus()
    }
  }, [hasChecked, isChecking, checkAuthStatus])

  // Show loading while checking
  if (!hasChecked || isChecking) {
    return <LoadingFallback />
  }

  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />
  }

  return (
    <LazyComponent>
      <LoginPage />
    </LazyComponent>
  )
}

const routes = [
  {
    path: '/login',
    element: <LoginGuard />,
    errorElement: <RouteErrorBoundary />,
  },
  {
    path: '/',
    element: <ProtectedRoute />,
    errorElement: <RouteErrorBoundary />,
    children: [
      {
        element: (
          <LazyComponent>
            <AdminLayout />
          </LazyComponent>
        ),
        children: [
          {
            index: true,
            element: <Navigate to="/dashboard" replace />,
          },
          {
            path: 'dashboard',
            element: (
              <LazyComponent>
                <DashboardPage />
              </LazyComponent>
            ),
          },
          {
            path: 'links',
            element: (
              <LazyComponent>
                <LinksPage />
              </LazyComponent>
            ),
          },
          {
            path: 'links/:code',
            element: (
              <LazyComponent>
                <LinkDetailPage />
              </LazyComponent>
            ),
          },
          {
            path: 'analytics',
            element: (
              <LazyComponent>
                <AnalyticsPage />
              </LazyComponent>
            ),
          },
          {
            path: 'settings',
            element: (
              <LazyComponent>
                <SettingsPage />
              </LazyComponent>
            ),
          },
        ],
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/dashboard" replace />,
  },
]

// 只在非根路径时设置 basename，避免 react-router 处理 '/' 时的问题
export const router = createBrowserRouter(
  routes,
  basePath !== '/' ? { basename: basePath } : undefined,
)

export default router
