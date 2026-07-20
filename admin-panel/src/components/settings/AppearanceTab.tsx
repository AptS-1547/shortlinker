import { useTranslation } from 'react-i18next'
import {
  FiMonitor as Monitor,
  FiMoon as Moon,
  FiSun as Sun,
} from 'react-icons/fi'

import { useTheme } from '@/hooks/useTheme'
import { cn } from '@/lib/utils'

export function AppearanceTab() {
  const { t } = useTranslation()
  const { theme, setTheme } = useTheme()

  const themeOptions = [
    {
      value: 'light' as const,
      label: t('theme.light'),
      icon: Sun,
    },
    {
      value: 'dark' as const,
      label: t('theme.dark'),
      icon: Moon,
    },
    {
      value: 'system' as const,
      label: t('theme.system'),
      icon: Monitor,
    },
  ]

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-lg font-medium">{t('settings.theme.title')}</h3>
        <p className="text-sm text-muted-foreground">
          {t('settings.theme.description')}
        </p>
      </div>
      <div className="grid grid-cols-3 gap-3">
        {themeOptions.map((option) => {
          const Icon = option.icon
          const isSelected = theme === option.value
          return (
            <button
              type="button"
              key={option.value}
              onClick={() => setTheme(option.value)}
              className={cn(
                'flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-all',
                isSelected
                  ? 'border-primary bg-primary/5'
                  : 'border-border hover:border-primary/50 hover:bg-muted/50',
              )}
            >
              <Icon
                className={cn(
                  'h-6 w-6',
                  isSelected ? 'text-primary' : 'text-muted-foreground',
                )}
              />
              <span
                className={cn(
                  'text-sm font-medium',
                  isSelected ? 'text-primary' : 'text-foreground',
                )}
              >
                {option.label}
              </span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
