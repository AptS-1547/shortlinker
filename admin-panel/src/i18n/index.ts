import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import { i18nLogger } from '@/utils/logger'
import { STORAGE_KEYS, Storage } from '@/utils/storage'

// 语言配置映射
export const languageConfig = {
  zh: {
    flag: '🇨🇳',
    name: '中文',
    code: 'zh',
  },
  en: {
    flag: '🇺🇸',
    name: 'English',
    code: 'en',
  },
  ja: {
    flag: '🇯🇵',
    name: '日本語',
    code: 'ja',
  },
  fr: {
    flag: '🇫🇷',
    name: 'Français',
    code: 'fr',
  },
  ru: {
    flag: '🇷🇺',
    name: 'Русский',
    code: 'ru',
  },
} as const

export type SupportedLanguage = keyof typeof languageConfig

// 支持的语言列表
export const supportedLanguages = Object.keys(
  languageConfig,
) as SupportedLanguage[]

// 获取语言显示名称
export const getLanguageDisplayName = (locale: string) => {
  return languageConfig[locale as SupportedLanguage]?.name || locale
}

// 获取语言旗帜
export const getLanguageFlag = (locale: string) => {
  return languageConfig[locale as SupportedLanguage]?.flag || '🌐'
}

// 获取语言翻译键名
export const getLanguageKey = (lang: string) => {
  const keyMap: Record<string, string> = {
    zh: 'chinese',
    en: 'english',
    ja: 'japanese',
    fr: 'french',
    ru: 'russian',
  }
  return keyMap[lang] || lang
}

// 语言包加载器（懒加载）
const languageLoaders: Record<SupportedLanguage, () => Promise<unknown>> = {
  en: () => import('./locales/en.json'),
  zh: () => import('./locales/zh.json'),
  ja: () => import('./locales/ja.json'),
  fr: () => import('./locales/fr.json'),
  ru: () => import('./locales/ru.json'),
}

// 已加载的语言缓存
const loadedLanguages = new Set<string>()

/**
 * 加载指定语言的资源
 */
export async function loadLanguage(lang: SupportedLanguage): Promise<void> {
  if (loadedLanguages.has(lang)) return

  const loader = languageLoaders[lang]
  if (!loader) return

  try {
    const module = await loader()
    const translations = (module as { default: unknown }).default
    i18n.addResourceBundle(lang, 'translation', translations, true, true)
    loadedLanguages.add(lang)
  } catch (error) {
    i18nLogger.error(`[i18n] Failed to load language: ${lang}`, error)
  }
}

/**
 * 切换语言
 */
export async function changeLanguage(lang: SupportedLanguage): Promise<void> {
  await loadLanguage(lang)
  await i18n.changeLanguage(lang)
  Storage.set(STORAGE_KEYS.LANGUAGE, lang)
}

// 获取用户首选语言
function getDefaultLocale(): SupportedLanguage {
  // 1. 从 localStorage 获取用户设置
  const saved = Storage.get(STORAGE_KEYS.LANGUAGE)
  if (saved && supportedLanguages.includes(saved as SupportedLanguage)) {
    return saved as SupportedLanguage
  }

  // 2. 从浏览器语言获取
  const browserLang = navigator.language.toLowerCase()
  if (browserLang.startsWith('zh')) return 'zh'
  if (browserLang.startsWith('ja')) return 'ja'
  if (browserLang.startsWith('fr')) return 'fr'
  if (browserLang.startsWith('ru')) return 'ru'

  // 3. 默认英语
  return 'en'
}

// 初始化 i18n
const defaultLang = getDefaultLocale()

i18n.use(initReactI18next).init({
  lng: defaultLang,
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
  // 不预加载资源，使用懒加载
  resources: {},
})

// 初始加载默认语言和英语（作为 fallback）
Promise.all([
  loadLanguage('en'),
  defaultLang !== 'en' ? loadLanguage(defaultLang) : Promise.resolve(),
])

export default i18n
