import { useCallback, useEffect, useState } from 'react'
import { STORAGE_KEYS, Storage } from '@/utils/storage'

export type Theme = 'light' | 'dark' | 'system'

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = Storage.get(STORAGE_KEYS.THEME) as Theme
    return saved && ['light', 'dark', 'system'].includes(saved)
      ? saved
      : 'system'
  })
  const [isDark, setIsDark] = useState(() => {
    const saved = Storage.get(STORAGE_KEYS.THEME) as Theme
    const prefersDark = window.matchMedia(
      '(prefers-color-scheme: dark)',
    ).matches
    return saved === 'dark' || (saved !== 'light' && prefersDark)
  })

  const updateDarkMode = useCallback((currentTheme: Theme) => {
    let dark: boolean
    if (currentTheme === 'system') {
      dark = window.matchMedia('(prefers-color-scheme: dark)').matches
    } else {
      dark = currentTheme === 'dark'
    }

    setIsDark(dark)

    const htmlElement = document.documentElement
    const bodyElement = document.body

    if (dark) {
      htmlElement.classList.add('dark')
      htmlElement.style.backgroundColor = '#0f172a'
      bodyElement.style.backgroundColor = '#0f172a'
    } else {
      htmlElement.classList.remove('dark')
      htmlElement.style.backgroundColor = '#f8fafc'
      bodyElement.style.backgroundColor = '#f8fafc'
    }
  }, [])

  const setTheme = useCallback(
    (newTheme: Theme) => {
      setThemeState(newTheme)
      Storage.set(STORAGE_KEYS.THEME, newTheme)
      updateDarkMode(newTheme)
    },
    [updateDarkMode],
  )

  useEffect(() => {
    const savedTheme = Storage.get(STORAGE_KEYS.THEME) as Theme
    if (savedTheme && ['light', 'dark', 'system'].includes(savedTheme)) {
      setThemeState(savedTheme)
      updateDarkMode(savedTheme)
    } else {
      updateDarkMode('system')
    }

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = () => {
      const currentTheme =
        (Storage.get(STORAGE_KEYS.THEME) as Theme) || 'system'
      if (currentTheme === 'system') {
        updateDarkMode('system')
      }
    }
    mediaQuery.addEventListener('change', handler)
    return () => mediaQuery.removeEventListener('change', handler)
  }, [updateDarkMode])

  return { theme, isDark, setTheme }
}
