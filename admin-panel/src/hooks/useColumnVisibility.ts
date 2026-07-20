import { useCallback, useState } from 'react'
import type { ColumnKey } from '@/components/links/LinksTable'
import { DEFAULT_VISIBLE_COLUMNS } from '@/components/links/LinksTable'
import { STORAGE_KEYS, Storage } from '@/utils/storage'

function loadVisibleColumns(): ColumnKey[] {
  try {
    const parsed = Storage.getJSON<ColumnKey[]>(
      STORAGE_KEYS.LINKS_VISIBLE_COLUMNS,
    )
    if (
      Array.isArray(parsed) &&
      parsed.every((col) => DEFAULT_VISIBLE_COLUMNS.includes(col))
    ) {
      return parsed
    }
  } catch {
    // 忽略解析错误
  }
  return DEFAULT_VISIBLE_COLUMNS
}

export function useColumnVisibility() {
  const [visibleColumns, setVisibleColumns] =
    useState<ColumnKey[]>(loadVisibleColumns)

  const handleColumnToggle = useCallback((col: ColumnKey, checked: boolean) => {
    setVisibleColumns((prev) => {
      const next = checked ? [...prev, col] : prev.filter((c) => c !== col)
      Storage.setJSON(STORAGE_KEYS.LINKS_VISIBLE_COLUMNS, next)
      return next
    })
  }, [])

  return {
    visibleColumns,
    handleColumnToggle,
  }
}
