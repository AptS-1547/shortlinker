import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { batchService } from '@/services/batchService'
import type { ImportMode, ImportResponse } from '@/services/types'

export type LinkImportState = 'idle' | 'uploading' | 'success' | 'error'

export function useLinkImport(onSuccess: () => void) {
  const { t } = useTranslation()
  const [state, setState] = useState<LinkImportState>('idle')
  const [result, setResult] = useState<ImportResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [uploadProgress, setUploadProgress] = useState(0)

  const importLinks = useCallback(
    async (file: File, mode: ImportMode) => {
      setState('uploading')
      setUploadProgress(0)
      setError(null)
      try {
        const response = await batchService.importLinks(
          file,
          mode,
          setUploadProgress,
        )
        setResult(response)
        setState('success')
        onSuccess()
      } catch (requestError) {
        setError(
          requestError instanceof Error
            ? requestError.message
            : t('links.import.error'),
        )
        setState('error')
      }
    },
    [onSuccess, t],
  )

  const reset = useCallback(() => {
    setState('idle')
    setResult(null)
    setError(null)
    setUploadProgress(0)
  }, [])

  const fail = useCallback((message: string) => {
    setError(message)
    setState('error')
  }, [])

  return { state, result, error, uploadProgress, importLinks, reset, fail }
}
