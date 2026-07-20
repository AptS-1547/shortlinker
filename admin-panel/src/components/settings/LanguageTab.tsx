import { useTranslation } from 'react-i18next'

import {
  changeLanguage,
  getLanguageFlag,
  getLanguageKey,
  type SupportedLanguage,
  supportedLanguages,
} from '@/i18n'
import { cn } from '@/lib/utils'

export function LanguageTab() {
  const { t, i18n } = useTranslation()

  const handleLanguageChange = (lang: SupportedLanguage) => {
    changeLanguage(lang)
  }

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-lg font-medium">{t('settings.language.title')}</h3>
        <p className="text-sm text-muted-foreground">
          {t('settings.language.description')}
        </p>
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3">
        {supportedLanguages.map((lang) => {
          const isSelected = i18n.language === lang
          return (
            <button
              type="button"
              key={lang}
              onClick={() => handleLanguageChange(lang)}
              className={cn(
                'flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-all',
                isSelected
                  ? 'border-primary bg-primary/5'
                  : 'border-border hover:border-primary/50 hover:bg-muted/50',
              )}
            >
              <span className="text-2xl">{getLanguageFlag(lang)}</span>
              <span
                className={cn(
                  'text-sm font-medium',
                  isSelected ? 'text-primary' : 'text-foreground',
                )}
              >
                {t(`langswitcher.language.${getLanguageKey(lang)}`)}
              </span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
