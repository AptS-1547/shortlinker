import { create } from 'zustand'
import { AuthAPI } from '@/services/api'
import { authLogger } from '@/utils/logger'

// Token 刷新提前量（秒）- 在过期前多少秒刷新
const REFRESH_BUFFER_SECONDS = 120

// sessionStorage key
const STORAGE_KEY = 'auth_expires_at'

// 模块级定时器
let refreshTimer: ReturnType<typeof setTimeout> | null = null

// 辅助函数：保存 expiresAt 到 sessionStorage
const saveExpiresAt = (expiresAt: number): void => {
  try {
    sessionStorage.setItem(STORAGE_KEY, expiresAt.toString())
  } catch {
    // sessionStorage 不可用时忽略
  }
}

// 辅助函数：从 sessionStorage 读取 expiresAt
const loadExpiresAt = (): number | null => {
  try {
    const saved = sessionStorage.getItem(STORAGE_KEY)
    if (saved) {
      const expiresAt = parseInt(saved, 10)
      // 检查是否已过期
      if (expiresAt > Date.now()) {
        return expiresAt
      }
      // 已过期，清除
      sessionStorage.removeItem(STORAGE_KEY)
    }
  } catch {
    // sessionStorage 不可用时忽略
  }
  return null
}

// 辅助函数：清除 sessionStorage
const clearExpiresAt = (): void => {
  try {
    sessionStorage.removeItem(STORAGE_KEY)
  } catch {
    // sessionStorage 不可用时忽略
  }
}

interface AuthState {
  isAuthenticated: boolean
  isChecking: boolean
  hasChecked: boolean
  expiresAt: number | null

  checkAuthStatus: () => Promise<boolean>
  login: (password: string) => Promise<void>
  logout: () => Promise<void>
  refreshToken: () => Promise<void>
  startAutoRefresh: () => void
  stopAutoRefresh: () => void
}

export const useAuthStore = create<AuthState>((set, get) => ({
  // Initial state
  isAuthenticated: false,
  isChecking: false,
  hasChecked: false,
  expiresAt: null,

  checkAuthStatus: async () => {
    // Skip if already checking
    if (get().isChecking) {
      authLogger.info('checkAuthStatus: already checking, skipping')
      return get().isAuthenticated
    }

    authLogger.info('checkAuthStatus: starting verification')

    // 尝试从 sessionStorage 恢复 expiresAt
    if (!get().expiresAt) {
      const savedExpiresAt = loadExpiresAt()
      if (savedExpiresAt) {
        authLogger.info('checkAuthStatus: restored expiresAt from storage')
        set({ expiresAt: savedExpiresAt })
      }
    }

    set({ isChecking: true })
    try {
      // Verify token via API call - cookie is automatically sent
      const isValid = await AuthAPI.verifyToken()
      authLogger.info('checkAuthStatus: verify result =', isValid)
      set({ isAuthenticated: isValid, hasChecked: true })

      // 如果验证成功，检查是否需要刷新 token
      if (isValid) {
        const expiresAt = get().expiresAt
        const now = Date.now()
        const timeUntilExpiry = expiresAt ? expiresAt - now : null
        const shouldRefreshNow =
          !expiresAt || expiresAt - now < REFRESH_BUFFER_SECONDS * 1000

        authLogger.info('checkAuthStatus: expiresAt =', expiresAt)
        authLogger.info(
          'checkAuthStatus: timeUntilExpiry =',
          timeUntilExpiry ? `${Math.round(timeUntilExpiry / 1000)}s` : 'null',
        )
        authLogger.info('checkAuthStatus: shouldRefreshNow =', shouldRefreshNow)

        if (shouldRefreshNow) {
          // 没有过期时间信息或快过期了，需要刷新
          authLogger.info('checkAuthStatus: triggering immediate refresh')
          try {
            await get().refreshToken()
          } catch {
            // 刷新失败不影响当前认证状态
            authLogger.warn(
              'checkAuthStatus: refresh failed, will retry on next check',
            )
          }
        } else {
          // token 还很新，只启动定时器等待下次刷新
          authLogger.info(
            'checkAuthStatus: token still valid, scheduling refresh',
          )
          get().startAutoRefresh()
        }
      }

      return isValid
    } catch (error) {
      authLogger.error('checkAuthStatus: failed', error)
      set({ isAuthenticated: false, hasChecked: true })
      return false
    } finally {
      set({ isChecking: false })
    }
  },

  login: async (password: string) => {
    authLogger.info('login: attempting login')
    try {
      const result = await AuthAPI.login({ password })
      const expiresAt = Date.now() + result.expiresIn * 1000
      authLogger.info('login: success, expiresIn =', result.expiresIn, 's')
      set({ isAuthenticated: true, hasChecked: true, expiresAt })
      saveExpiresAt(expiresAt)
      get().startAutoRefresh()
    } catch (error) {
      authLogger.error('login: failed', error)
      set({ isAuthenticated: false })
      throw error
    }
  },

  logout: async () => {
    authLogger.info('logout: logging out')
    get().stopAutoRefresh()
    clearExpiresAt()
    try {
      await AuthAPI.logout()
      authLogger.info('logout: API call success')
    } catch (error) {
      authLogger.error('logout: API call failed', error)
    } finally {
      // Always set as logged out regardless of API result
      set({ isAuthenticated: false, expiresAt: null })
    }
  },

  refreshToken: async () => {
    authLogger.info('refreshToken: starting refresh')
    try {
      const result = await AuthAPI.refreshToken()
      const expiresAt = Date.now() + result.expiresIn * 1000
      set({ expiresAt })
      saveExpiresAt(expiresAt)
      authLogger.info(
        'refreshToken: success, expiresIn =',
        result.expiresIn,
        's, next refresh in',
        result.expiresIn - REFRESH_BUFFER_SECONDS,
        's',
      )
      get().startAutoRefresh()
    } catch (error) {
      authLogger.error('refreshToken: failed', error)
      get().stopAutoRefresh()
      clearExpiresAt()
      set({ isAuthenticated: false, expiresAt: null })
      throw error
    }
  },

  startAutoRefresh: () => {
    const expiresAt = get().expiresAt
    if (!expiresAt) {
      authLogger.info('startAutoRefresh: no expiresAt, skipping')
      return
    }

    // 清除现有定时器
    if (refreshTimer) {
      authLogger.info('startAutoRefresh: clearing existing timer')
      clearTimeout(refreshTimer)
      refreshTimer = null
    }

    // 计算刷新时间（提前 REFRESH_BUFFER_SECONDS 秒刷新）
    const refreshIn = expiresAt - Date.now() - REFRESH_BUFFER_SECONDS * 1000
    if (refreshIn <= 0) {
      // 已经过期或即将过期，立即刷新
      authLogger.info(
        'startAutoRefresh: token expired or expiring soon, refreshing now',
      )
      get().refreshToken()
      return
    }

    authLogger.info(
      'startAutoRefresh: scheduled in',
      Math.round(refreshIn / 1000),
      's',
    )
    refreshTimer = setTimeout(() => {
      authLogger.info('startAutoRefresh: timer triggered, refreshing')
      get().refreshToken()
    }, refreshIn)
  },

  stopAutoRefresh: () => {
    if (refreshTimer) {
      clearTimeout(refreshTimer)
      refreshTimer = null
      authLogger.info('Auto refresh stopped')
    }
  },
}))

// 导出用于 http.ts 调用的登出方法（避免循环依赖）
export const forceLogout = (): void => {
  useAuthStore.getState().stopAutoRefresh()
  clearExpiresAt()
  useAuthStore.setState({ isAuthenticated: false, expiresAt: null })
}

// 导出用于 http.ts 调用的刷新方法（避免循环依赖）
export const refreshTokenFromHttp = (): Promise<void> => {
  return useAuthStore.getState().refreshToken()
}
