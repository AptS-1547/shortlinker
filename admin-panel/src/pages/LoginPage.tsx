import { type FormEvent, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiAlertTriangle as AlertTriangle,
  FiGlobe as Languages,
  FiLink as Link,
  FiLoader as Loader2,
  FiMonitor as Monitor,
  FiMoon as Moon,
  FiSun as Sun,
} from 'react-icons/fi'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { appConfig } from '@/config/app'
import { useTheme } from '@/hooks/useTheme'
import { useVersion } from '@/hooks/useVersion'
import {
  changeLanguage,
  getLanguageFlag,
  getLanguageKey,
  type SupportedLanguage,
  supportedLanguages,
} from '@/i18n'
import { HealthAPI } from '@/services/api'
import { useAuthStore } from '@/stores/authStore'
import { authLogger } from '@/utils/logger'

export default function LoginPage() {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const login = useAuthStore((state) => state.login)
  const { isDark, setTheme } = useTheme()
  const { displayVersion, versionBadgeColor, versionLabel } = useVersion()

  const [password, setPassword] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState('')

  const apiBaseUrl =
    appConfig.apiBaseUrl ||
    (typeof window !== 'undefined'
      ? window.location.origin
      : 'http://127.0.0.1:8080')

  const isProgressMessage =
    error.includes(t('auth.authenticating')) ||
    error.includes(t('auth.verifying'))

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()

    if (!password.trim()) {
      setError(t('auth.errors.passwordRequired'))
      return
    }

    setIsSubmitting(true)
    setError('')

    try {
      setError(t('auth.authenticating'))
      await login(password.trim())

      setError(t('auth.verifying'))
      await HealthAPI.check()

      navigate('/dashboard')
    } catch (err) {
      authLogger.error('Authentication failed:', err)

      if (err instanceof Error) {
        if (
          err.message.includes('Network Error') ||
          err.message.includes('ECONNREFUSED')
        ) {
          setError(t('auth.errors.networkError'))
        } else if (
          err.message.includes('401') ||
          err.message.includes('INVALID_CREDENTIALS')
        ) {
          setError(t('auth.errors.unauthorized'))
        } else if (err.message.includes('404')) {
          setError(t('auth.errors.notFound'))
        } else if (err.message.includes('500')) {
          setError(t('auth.errors.serverError'))
        } else {
          setError(t('auth.errors.authFailed'))
        }
      } else {
        setError(t('auth.errors.authFailed'))
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleChangeLanguage = (lang: SupportedLanguage) => {
    changeLanguage(lang)
  }

  return (
    <div className="min-h-screen bg-muted/30 flex items-center justify-center p-4 relative">
      {/* Top-right settings buttons */}
      <div className="absolute top-4 right-4 flex items-center gap-2">
        {/* Theme toggle */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              {isDark ? (
                <Moon className="h-4 w-4" />
              ) : (
                <Sun className="h-4 w-4" />
              )}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => setTheme('light')}>
              <Sun className="h-4 w-4 mr-2" />
              {t('theme.light')}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setTheme('dark')}>
              <Moon className="h-4 w-4 mr-2" />
              {t('theme.dark')}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setTheme('system')}>
              <Monitor className="h-4 w-4 mr-2" />
              {t('theme.system')}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Language toggle */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <Languages className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            {supportedLanguages.map((lang) => (
              <DropdownMenuItem
                key={lang}
                onClick={() => handleChangeLanguage(lang)}
                className={i18n.language === lang ? 'bg-accent' : ''}
              >
                <span className="mr-2">{getLanguageFlag(lang)}</span>
                {t(`langswitcher.language.${getLanguageKey(lang)}`)}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="w-full max-w-sm">
        {/* Logo */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-primary text-primary-foreground mb-4">
            <Link className="w-6 h-6" />
          </div>
          <h1 className="text-xl font-semibold text-foreground">ShortLinker</h1>
          <p className="text-sm text-muted-foreground mt-1">
            {t('auth.description')}
          </p>
        </div>

        {/* Login Form */}
        <Card className="border-border/50">
          <CardContent className="pt-6">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="password">{t('auth.password')}</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder={t('auth.passwordPlaceholder')}
                  disabled={isSubmitting}
                  required
                />
              </div>

              {error && (
                <div
                  className={`flex items-center gap-2 p-3 rounded-lg text-sm ${
                    isProgressMessage
                      ? 'bg-primary/10 text-primary'
                      : 'bg-destructive/10 text-destructive'
                  }`}
                >
                  {isProgressMessage ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <AlertTriangle className="w-4 h-4" />
                  )}
                  <span>{error}</span>
                </div>
              )}

              <Button type="submit" className="w-full" disabled={isSubmitting}>
                {isSubmitting ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    {error.includes(t('auth.authenticating'))
                      ? t('auth.authenticating')
                      : error.includes(t('auth.verifying'))
                        ? t('auth.verifying')
                        : t('auth.processing')}
                  </>
                ) : (
                  t('auth.login')
                )}
              </Button>
            </form>
          </CardContent>
        </Card>

        {/* Server info */}
        <p className="text-center text-xs text-muted-foreground mt-6">
          {t('auth.serverConnectionDesc', { url: apiBaseUrl })}
        </p>

        {/* Version info */}
        <div className="flex items-center justify-center gap-2 mt-2">
          <span className="text-xs text-muted-foreground">
            {displayVersion}
          </span>
          <span
            className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${versionBadgeColor}`}
          >
            {versionLabel}
          </span>
        </div>
      </div>
    </div>
  )
}
