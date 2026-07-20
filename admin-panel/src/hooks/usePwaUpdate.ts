import { useRegisterSW } from 'virtual:pwa-register/react'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { pwaLogger } from '@/utils/logger'

// PWA 更新检测 hook
export function usePwaUpdate() {
  const { t } = useTranslation()

  const {
    needRefresh: [needRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegistered(r) {
      pwaLogger.info('Service Worker registered:', r)
      // 每小时检查一次更新
      r && setInterval(() => r.update(), 60 * 60 * 1000)
    },
  })

  useEffect(() => {
    pwaLogger.debug('needRefresh:', needRefresh)
    if (needRefresh) {
      toast.info(t('pwa.updateAvailable'), {
        action: {
          label: t('pwa.refresh'),
          onClick: () => updateServiceWorker(true),
        },
        duration: Infinity,
      })
    }
  }, [needRefresh, t, updateServiceWorker])
}
