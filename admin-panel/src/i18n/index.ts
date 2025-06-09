import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zh from './locales/zh.json'
import ja from './locales/ja.json'

const messages = {
  en,
  zh,
  ja
}

// 语言配置映射
export const languageConfig = {
  zh: {
    flag: '🇨🇳',
    name: '中文',
    code: 'zh'
  },
  en: {
    flag: '🇺🇸',
    name: 'English',
    code: 'en'
  },
  ja: {
    flag: '🇯🇵',
    name: '日本語',
    code: 'ja'
  }
} as const

// 支持的语言列表
export const supportedLanguages = Object.keys(languageConfig) as Array<keyof typeof languageConfig>

// 获取语言显示名称
export const getLanguageDisplayName = (locale: string) => {
  return languageConfig[locale as keyof typeof languageConfig]?.name || locale
}

// 获取语言旗帜
export const getLanguageFlag = (locale: string) => {
  return languageConfig[locale as keyof typeof languageConfig]?.flag || '🌐'
}

// 获取语言翻译键名
export const getLanguageKey = (lang: string) => {
  const keyMap: Record<string, string> = {
    zh: 'chinese',
    en: 'english',
    ja: 'japanese'
  }
  return keyMap[lang] || lang
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
  if (browserLang.startsWith('ja')) {
    return 'ja'
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
