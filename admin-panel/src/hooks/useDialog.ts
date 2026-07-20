import { useCallback, useState } from 'react'

/**
 * 通用对话框状态管理 hook
 * 减少对话框状态管理的重复代码
 *
 * @example
 * ```tsx
 * const formDialog = useDialog<LinkData>()
 * const deleteDialog = useDialog<LinkData>()
 *
 * // 打开创建对话框
 * formDialog.open()
 *
 * // 打开编辑对话框
 * formDialog.open(linkData)
 *
 * // 关闭对话框
 * formDialog.close()
 *
 * // 在组件中使用
 * <Dialog open={formDialog.isOpen} onOpenChange={formDialog.setIsOpen}>
 *   {formDialog.data ? '编辑' : '创建'}
 * </Dialog>
 * ```
 */
export function useDialog<T = unknown>(defaultOpen = false) {
  const [isOpen, setIsOpen] = useState(defaultOpen)
  const [data, setData] = useState<T | null>(null)

  const open = useCallback((item?: T) => {
    setData(item ?? null)
    setIsOpen(true)
  }, [])

  const close = useCallback(() => {
    setIsOpen(false)
    // 延迟清除数据，等待关闭动画完成
    setTimeout(() => setData(null), 150)
  }, [])

  const toggle = useCallback(() => {
    setIsOpen((prev) => !prev)
  }, [])

  return {
    /** 对话框是否打开 */
    isOpen,
    /** 设置对话框打开状态 */
    setIsOpen,
    /** 传入的数据（编辑模式时有值） */
    data,
    /** 设置数据 */
    setData,
    /** 打开对话框，可选传入数据 */
    open,
    /** 关闭对话框并清除数据 */
    close,
    /** 切换对话框状态 */
    toggle,
    /** 是否为编辑模式（有数据） */
    isEditMode: data !== null,
  }
}

export type UseDialogReturn<T> = ReturnType<typeof useDialog<T>>
