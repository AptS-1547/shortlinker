import { useCallback, useEffect, useRef, useState } from 'react'
import { systemConfigService } from '@/services/systemConfigService'
import type { ConfigHistoryResponse } from '@/services/types'

export function useConfigHistory(
  key: string | undefined,
  enabled: boolean,
  limit = 50,
) {
  const [history, setHistory] = useState<ConfigHistoryResponse[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const requestIdRef = useRef(0)

  const reload = useCallback(async () => {
    const requestId = ++requestIdRef.current
    if (!key || !enabled) {
      setHistory([])
      setLoading(false)
      setError(null)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const nextHistory = await systemConfigService.fetchHistory(key, limit)
      if (requestId !== requestIdRef.current) return
      setHistory(nextHistory)
    } catch (requestError) {
      if (requestId !== requestIdRef.current) return
      setError(
        requestError instanceof Error
          ? requestError.message
          : 'Failed to load history',
      )
    } finally {
      if (requestId === requestIdRef.current) setLoading(false)
    }
  }, [enabled, key, limit])

  useEffect(() => {
    void reload()
    return () => {
      requestIdRef.current += 1
    }
  }, [reload])

  return { history, loading, error, reload }
}
