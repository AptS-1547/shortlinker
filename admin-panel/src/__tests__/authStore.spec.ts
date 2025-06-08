import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach } from 'vitest'
import { useAuthStore } from '@/stores/auth'

describe('useAuthStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  it('login stores token', () => {
    const store = useAuthStore()
    store.login('secret')
    expect(store.token).toBe('secret')
    expect(localStorage.getItem('adminToken')).toBe('secret')
    expect(store.isAuthenticated).toBe(true)
  })

  it('logout clears token', () => {
    const store = useAuthStore()
    store.login('secret')
    store.logout()
    expect(store.token).toBeNull()
    expect(localStorage.getItem('adminToken')).toBeNull()
    expect(store.isAuthenticated).toBe(false)
  })
})
