import { useCallback, useRef, useState } from 'react'
import { hookLogger } from '@/utils/logger'

export interface ConfirmOptions {
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'danger' | 'primary' | 'success'
}

export function useConfirm() {
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(false)
  const [currentOptions, setCurrentOptions] = useState<ConfirmOptions | null>(
    null,
  )
  const resolveRef = useRef<((value: boolean) => void) | null>(null)

  const confirm = useCallback((options: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      resolveRef.current = resolve
      setCurrentOptions(options)
      setIsOpen(true)
    })
  }, [])

  const handleConfirm = useCallback(
    async (onConfirm?: () => void | Promise<void>) => {
      if (onConfirm) {
        setLoading(true)
        try {
          await onConfirm()
          resolveRef.current?.(true)
        } catch (error) {
          hookLogger.error('Confirm action failed:', error)
          resolveRef.current?.(false)
        } finally {
          setLoading(false)
          setIsOpen(false)
        }
      } else {
        resolveRef.current?.(true)
        setIsOpen(false)
      }
    },
    [],
  )

  const handleCancel = useCallback(() => {
    resolveRef.current?.(false)
    setIsOpen(false)
  }, [])

  return {
    isOpen,
    loading,
    currentOptions,
    confirm,
    handleConfirm,
    handleCancel,
  }
}
