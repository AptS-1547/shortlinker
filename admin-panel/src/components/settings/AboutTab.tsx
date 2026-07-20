import { useTranslation } from 'react-i18next'
import {
  FiBookOpen as BookOpen,
  FiExternalLink as ExternalLink,
  FiGithub as Github,
  FiHeart as Heart,
  FiLink as LinkIcon,
} from 'react-icons/fi'

import { useVersion } from '@/hooks/useVersion'
import { cn } from '@/lib/utils'

export function AboutTab() {
  const { t } = useTranslation()
  const { displayVersion, versionBadgeColor, versionLabel } = useVersion()

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">{t('settings.about.title')}</h3>
        <p className="text-sm text-muted-foreground">
          {t('settings.about.description')}
        </p>
      </div>

      {/* Project Info */}
      <div className="flex items-center gap-4">
        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
          <LinkIcon className="h-6 w-6" />
        </div>
        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-lg">ShortLinker</span>
            <span className="text-sm text-muted-foreground">
              {displayVersion}
            </span>
            <span
              className={cn(
                'text-xs px-2 py-0.5 rounded font-medium',
                versionBadgeColor,
              )}
            >
              {versionLabel}
            </span>
          </div>
          <span className="text-sm text-muted-foreground">
            {t('settings.about.tagline')}
          </span>
        </div>
      </div>

      {/* Open Source Info */}
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground">
        <span>{t('settings.about.openSource')}</span>
        <span>•</span>
        <a
          href="https://github.com/AptS-1547/shortlinker/blob/master/LICENSE"
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-emerald-500 transition-colors"
        >
          MIT License
        </a>
        <span>•</span>
        <span className="flex items-center gap-1">
          <a
            href="https://react.dev/"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-cyan-500 transition-colors"
          >
            React
          </a>
          <span>+</span>
          <a
            href="https://www.typescriptlang.org/"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-blue-500 transition-colors"
          >
            TypeScript
          </a>
          <span>+</span>
          <a
            href="https://tailwindcss.com/"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-teal-500 transition-colors"
          >
            Tailwind
          </a>
        </span>
      </div>

      {/* Links */}
      <div className="flex flex-wrap gap-3">
        <a
          href="https://github.com/AptS-1547/shortlinker"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-border hover:bg-muted/50 transition-colors text-sm"
        >
          <Github className="h-4 w-4" />
          GitHub
          <ExternalLink className="h-3 w-3 text-muted-foreground" />
        </a>
        <a
          href="https://esap.cc/docs"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-border hover:bg-muted/50 transition-colors text-sm"
        >
          <BookOpen className="h-4 w-4" />
          {t('settings.about.docs')}
          <ExternalLink className="h-3 w-3 text-muted-foreground" />
        </a>
      </div>

      {/* Copyright & Powered By */}
      <div className="pt-4 border-t border-border space-y-2">
        <p className="text-sm text-muted-foreground flex items-center gap-1">
          © {new Date().getFullYear()} ShortLinker.{' '}
          {t('settings.about.builtWith')}{' '}
          <Heart
            className="h-3 w-3 text-red-500"
            style={{ fill: 'currentColor' }}
          />{' '}
          {t('settings.about.forCommunity')}
        </p>
        <p className="text-sm text-muted-foreground">
          Powered by{' '}
          <a
            href="https://www.rust-lang.org/"
            target="_blank"
            rel="noopener noreferrer"
            className="text-orange-500 font-medium hover:text-orange-400 transition-colors"
          >
            Rust
          </a>{' '}
          +{' '}
          <a
            href="https://react.dev/"
            target="_blank"
            rel="noopener noreferrer"
            className="text-cyan-500 font-medium hover:text-cyan-400 transition-colors"
          >
            React
          </a>
        </p>
      </div>
    </div>
  )
}
