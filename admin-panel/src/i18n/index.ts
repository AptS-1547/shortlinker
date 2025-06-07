import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zh from './locales/zh.json'

const messages = {
  en,
  zh
}

// 获取用户首选语言
function getDefaultLocale(): string {
  // 1. 从 localStorage 获取用户设置
  const saved = localStorage.getItem('preferred-language')
  if (saved && messages[saved as keyof typeof messages]) {
    return saved
  }

  // 2. 从浏览器语言获取
  const browserLang = navigator.language.toLowerCase()
  if (browserLang.startsWith('zh')) {
    return 'zh'
  }

  // 3. 默认英语
  return 'en'
}

export const i18n = createI18n({
  legacy: false,
  locale: getDefaultLocale(),
  fallbackLocale: 'en',
  messages,
  globalInjection: true
})

export default i18n
