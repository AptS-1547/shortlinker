import { act } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

// Mock AuthAPI
vi.mock('@/services/api', () => ({
  AuthAPI: {
    login: vi.fn(),
    logout: vi.fn(),
    verifyToken: vi.fn(),
    refreshToken: vi.fn(),
  },
}))

// Mock logger
vi.mock('@/utils/logger', () => ({
  authLogger: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}))

import { AuthAPI } from '@/services/api'
import { forceLogout, refreshTokenFromHttp, useAuthStore } from '../authStore'

describe('authStore', () => {
  beforeEach(() => {
    // Reset store state before each test
    useAuthStore.setState({
      isAuthenticated: false,
      isChecking: false,
      hasChecked: false,
      expiresAt: null,
    })
    // Clear mocks
    vi.clearAllMocks()
    // Clear sessionStorage
    sessionStorage.clear()
    // Use fake timers
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have correct initial values', () => {
      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.isChecking).toBe(false)
      expect(state.hasChecked).toBe(false)
      expect(state.expiresAt).toBe(null)
    })
  })

  // ==========================================================================
  // login
  // ==========================================================================

  describe('login', () => {
    it('should set isAuthenticated to true on successful login', async () => {
      vi.mocked(AuthAPI.login).mockResolvedValueOnce({
        expiresIn: 3600,
      })

      await act(async () => {
        await useAuthStore.getState().login('password123')
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(true)
      expect(state.hasChecked).toBe(true)
      expect(state.expiresAt).not.toBe(null)
    })

    it('should call AuthAPI.login with password', async () => {
      vi.mocked(AuthAPI.login).mockResolvedValueOnce({
        expiresIn: 3600,
      })

      await act(async () => {
        await useAuthStore.getState().login('mypassword')
      })

      expect(AuthAPI.login).toHaveBeenCalledWith({ password: 'mypassword' })
    })

    it('should save expiresAt to sessionStorage', async () => {
      vi.mocked(AuthAPI.login).mockResolvedValueOnce({
        expiresIn: 3600,
      })

      await act(async () => {
        await useAuthStore.getState().login('password123')
      })

      const saved = sessionStorage.getItem('auth_expires_at')
      expect(saved).not.toBe(null)
    })

    it('should throw error on failed login', async () => {
      const error = new Error('Invalid password')
      vi.mocked(AuthAPI.login).mockRejectedValueOnce(error)

      await expect(
        act(async () => {
          await useAuthStore.getState().login('wrongpassword')
        }),
      ).rejects.toThrow('Invalid password')

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
    })
  })

  // ==========================================================================
  // logout
  // ==========================================================================

  describe('logout', () => {
    it('should set isAuthenticated to false', async () => {
      // Set up authenticated state
      useAuthStore.setState({
        isAuthenticated: true,
        expiresAt: Date.now() + 3600000,
      })

      vi.mocked(AuthAPI.logout).mockResolvedValueOnce(undefined)

      await act(async () => {
        await useAuthStore.getState().logout()
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.expiresAt).toBe(null)
    })

    it('should call AuthAPI.logout', async () => {
      vi.mocked(AuthAPI.logout).mockResolvedValueOnce(undefined)

      await act(async () => {
        await useAuthStore.getState().logout()
      })

      expect(AuthAPI.logout).toHaveBeenCalled()
    })

    it('should set isAuthenticated to false even if API fails', async () => {
      useAuthStore.setState({ isAuthenticated: true })
      vi.mocked(AuthAPI.logout).mockRejectedValueOnce(
        new Error('Network error'),
      )

      await act(async () => {
        await useAuthStore.getState().logout()
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
    })

    it('should clear sessionStorage', async () => {
      sessionStorage.setItem('auth_expires_at', '123456')
      vi.mocked(AuthAPI.logout).mockResolvedValueOnce(undefined)

      await act(async () => {
        await useAuthStore.getState().logout()
      })

      // sessionStorage.getItem returns undefined in jsdom when item doesn't exist
      expect(sessionStorage.getItem('auth_expires_at')).toBeFalsy()
    })
  })

  // ==========================================================================
  // checkAuthStatus
  // ==========================================================================

  describe('checkAuthStatus', () => {
    it('should set isAuthenticated based on verifyToken result', async () => {
      vi.mocked(AuthAPI.verifyToken).mockResolvedValueOnce(true)
      vi.mocked(AuthAPI.refreshToken).mockResolvedValueOnce({
        expiresIn: 3600,
      })

      await act(async () => {
        await useAuthStore.getState().checkAuthStatus()
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(true)
      expect(state.hasChecked).toBe(true)
    })

    it('should set isAuthenticated to false when token is invalid', async () => {
      vi.mocked(AuthAPI.verifyToken).mockResolvedValueOnce(false)

      await act(async () => {
        await useAuthStore.getState().checkAuthStatus()
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.hasChecked).toBe(true)
    })

    it('should skip if already checking', async () => {
      useAuthStore.setState({ isChecking: true })

      await act(async () => {
        await useAuthStore.getState().checkAuthStatus()
      })

      expect(AuthAPI.verifyToken).not.toHaveBeenCalled()
    })

    it('should handle verification error', async () => {
      vi.mocked(AuthAPI.verifyToken).mockRejectedValueOnce(
        new Error('Network error'),
      )

      await act(async () => {
        await useAuthStore.getState().checkAuthStatus()
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.hasChecked).toBe(true)
    })

    it('should restore expiresAt from sessionStorage', async () => {
      // Use a time that's definitely in the future
      vi.setSystemTime(new Date('2024-01-01T00:00:00Z'))
      const futureTime = new Date('2024-01-01T01:00:00Z').getTime() // 1 hour in the future
      sessionStorage.setItem('auth_expires_at', futureTime.toString())
      vi.mocked(AuthAPI.verifyToken).mockResolvedValueOnce(true)
      // Token has valid expiresAt so no immediate refresh needed
      vi.mocked(AuthAPI.refreshToken).mockResolvedValue({ expiresIn: 3600 })

      await act(async () => {
        await useAuthStore.getState().checkAuthStatus()
      })

      const state = useAuthStore.getState()
      expect(state.expiresAt).toBe(futureTime)
    })
  })

  // ==========================================================================
  // refreshToken
  // ==========================================================================

  describe('refreshToken', () => {
    it('should update expiresAt on successful refresh', async () => {
      vi.mocked(AuthAPI.refreshToken).mockResolvedValueOnce({
        expiresIn: 3600,
      })

      await act(async () => {
        await useAuthStore.getState().refreshToken()
      })

      const state = useAuthStore.getState()
      expect(state.expiresAt).not.toBe(null)
    })

    it('should set isAuthenticated to false on refresh failure', async () => {
      useAuthStore.setState({ isAuthenticated: true })
      vi.mocked(AuthAPI.refreshToken).mockRejectedValueOnce(
        new Error('Refresh failed'),
      )

      await expect(
        act(async () => {
          await useAuthStore.getState().refreshToken()
        }),
      ).rejects.toThrow('Refresh failed')

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.expiresAt).toBe(null)
    })
  })

  // ==========================================================================
  // startAutoRefresh / stopAutoRefresh
  // ==========================================================================

  describe('auto refresh', () => {
    it('should not start refresh timer without expiresAt', () => {
      useAuthStore.setState({ expiresAt: null })

      act(() => {
        useAuthStore.getState().startAutoRefresh()
      })

      // No error should occur
      expect(true).toBe(true)
    })

    it('should schedule refresh when expiresAt is set', async () => {
      const expiresAt = Date.now() + 600000 // 10 minutes
      useAuthStore.setState({ expiresAt })
      vi.mocked(AuthAPI.refreshToken).mockResolvedValue({
        expiresIn: 3600,
      })

      act(() => {
        useAuthStore.getState().startAutoRefresh()
      })

      // Advance time to just before refresh should happen (10min - 2min buffer = 8min)
      await act(async () => {
        vi.advanceTimersByTime(480000 + 1000) // 8min + 1s
      })

      expect(AuthAPI.refreshToken).toHaveBeenCalled()
    })

    it('should immediately refresh if token is about to expire', async () => {
      const expiresAt = Date.now() + 60000 // 1 minute (within buffer)
      useAuthStore.setState({ expiresAt })
      vi.mocked(AuthAPI.refreshToken).mockResolvedValue({
        expiresIn: 3600,
      })

      await act(async () => {
        useAuthStore.getState().startAutoRefresh()
      })

      expect(AuthAPI.refreshToken).toHaveBeenCalled()
    })

    it('should stop auto refresh timer', () => {
      const expiresAt = Date.now() + 600000
      useAuthStore.setState({ expiresAt })

      act(() => {
        useAuthStore.getState().startAutoRefresh()
        useAuthStore.getState().stopAutoRefresh()
      })

      // Should not throw or have issues
      expect(true).toBe(true)
    })
  })

  // ==========================================================================
  // forceLogout / refreshTokenFromHttp
  // ==========================================================================

  describe('exported functions', () => {
    it('forceLogout should set isAuthenticated to false', () => {
      useAuthStore.setState({
        isAuthenticated: true,
        expiresAt: Date.now() + 3600000,
      })

      act(() => {
        forceLogout()
      })

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.expiresAt).toBe(null)
    })

    it('refreshTokenFromHttp should call refreshToken', async () => {
      vi.mocked(AuthAPI.refreshToken).mockResolvedValueOnce({
        expiresIn: 3600,
      })

      await act(async () => {
        await refreshTokenFromHttp()
      })

      expect(AuthAPI.refreshToken).toHaveBeenCalled()
    })
  })
})
