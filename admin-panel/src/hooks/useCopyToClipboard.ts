import { useCallback, useRef, useState } from 'react'
import { type CopyResult, copyToClipboard } from '@/utils/clipboard'

export function useCopyToClipboard() {
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const copy = useCallback(
    async (text: string, resetDelay: number = 2000): Promise<boolean> => {
      setError(null)

      const result: CopyResult = await copyToClipboard(text)

      if (result.success) {
        setCopied(true)

        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current)
        }

        timeoutRef.current = setTimeout(() => {
          setCopied(false)
        }, resetDelay)

        return true
      } else {
        setError(result.error || new Error('Copy failed'))
        return false
      }
    },
    [],
  )

  const reset = useCallback(() => {
    setCopied(false)
    setError(null)
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
      timeoutRef.current = null
    }
  }, [])

  return { copied, error, copy, reset }
}
