import { useCallback, useMemo, useState } from 'react'

export interface TableSelectionOptions<T> {
  getItemId: (item: T) => string | number
}

export function useTableSelection<T>(
  items: T[],
  options: TableSelectionOptions<T>,
) {
  const [selected, setSelected] = useState<Set<string | number>>(new Set())

  const isSelected = useCallback(
    (item: T): boolean => {
      return selected.has(options.getItemId(item))
    },
    [selected, options],
  )

  const isAllSelected = useMemo((): boolean => {
    if (items.length === 0) return false
    return items.every((item) => selected.has(options.getItemId(item)))
  }, [items, selected, options])

  const isSomeSelected = useMemo((): boolean => {
    return selected.size > 0 && !isAllSelected
  }, [selected.size, isAllSelected])

  const toggle = useCallback(
    (item: T): void => {
      const id = options.getItemId(item)
      setSelected((prev) => {
        const newSet = new Set(prev)
        if (newSet.has(id)) {
          newSet.delete(id)
        } else {
          newSet.add(id)
        }
        return newSet
      })
    },
    [options],
  )

  const toggleAll = useCallback((): void => {
    if (isAllSelected) {
      setSelected(new Set())
    } else {
      const newSet = new Set<string | number>()
      items.forEach((item) => {
        newSet.add(options.getItemId(item))
      })
      setSelected(newSet)
    }
  }, [items, isAllSelected, options])

  const selectAll = useCallback((): void => {
    const newSet = new Set<string | number>()
    items.forEach((item) => {
      newSet.add(options.getItemId(item))
    })
    setSelected(newSet)
  }, [items, options])

  const deselectAll = useCallback((): void => {
    setSelected(new Set())
  }, [])

  const getSelected = useCallback((): T[] => {
    return items.filter((item) => selected.has(options.getItemId(item)))
  }, [items, selected, options])

  const getSelectedCount = useCallback((): number => {
    return selected.size
  }, [selected.size])

  return {
    selected,
    isSelected,
    isAllSelected,
    isSomeSelected,
    toggle,
    toggleAll,
    selectAll,
    deselectAll,
    getSelected,
    getSelectedCount,
  }
}
