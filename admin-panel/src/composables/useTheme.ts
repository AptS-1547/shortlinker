import { ref, computed, watch, onMounted } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

const theme = ref<Theme>('system')
const isDark = ref(false)

export function useTheme() {
  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme
    localStorage.setItem('theme', newTheme)
    updateDarkMode()
  }

  const updateDarkMode = () => {
    if (theme.value === 'system') {
      isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
    } else {
      isDark.value = theme.value === 'dark'
    }

    // 更新 document class 和背景色
    const htmlElement = document.documentElement
    const bodyElement = document.body
    
    if (isDark.value) {
      htmlElement.classList.add('dark')
      htmlElement.style.backgroundColor = '#0f172a' // slate-900
      bodyElement.style.backgroundColor = '#0f172a'
    } else {
      htmlElement.classList.remove('dark')
      htmlElement.style.backgroundColor = '#f8fafc' // slate-50
      bodyElement.style.backgroundColor = '#f8fafc'
    }
  }

  const initTheme = () => {
    const savedTheme = localStorage.getItem('theme') as Theme
    if (savedTheme && ['light', 'dark', 'system'].includes(savedTheme)) {
      theme.value = savedTheme
    }
    
    updateDarkMode()

    // 监听系统主题变化
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', updateDarkMode)
  }

  onMounted(() => {
    initTheme()
  })

  watch(theme, updateDarkMode)

  return {
    theme: computed(() => theme.value),
    isDark: computed(() => isDark.value),
    setTheme
  }
}
