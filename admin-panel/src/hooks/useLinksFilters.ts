import { useCallback, useEffect, useRef, useState } from 'react'
import type { GetLinksQuery } from '@/services/types'

export type StatusFilter = 'all' | 'active' | 'expired'

export interface LinksFilterState {
  searchQuery: string
  statusFilter: StatusFilter
  createdAfter: Date | undefined
  createdBefore: Date | undefined
}

interface UseLinksFiltersOptions {
  onFilterChange: (query: GetLinksQuery) => void
  debounceMs?: number
}

export function useLinksFilters({
  onFilterChange,
  debounceMs = 300,
}: UseLinksFiltersOptions) {
  const [searchQuery, setSearchQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [createdAfter, setCreatedAfter] = useState<Date | undefined>(undefined)
  const [createdBefore, setCreatedBefore] = useState<Date | undefined>(
    undefined,
  )

  const hasFetched = useRef(false)
  const prevFilter = useRef({
    searchQuery,
    statusFilter,
    createdAfter,
    createdBefore,
  })

  const buildQuery = useCallback((): GetLinksQuery => {
    return {
      search: searchQuery || null,
      only_active: statusFilter === 'active' ? true : null,
      only_expired: statusFilter === 'expired' ? true : null,
      created_after: createdAfter?.toISOString() ?? null,
      created_before: createdBefore?.toISOString() ?? null,
      page: 1,
      page_size: null,
    }
  }, [searchQuery, statusFilter, createdAfter, createdBefore])

  // biome-ignore lint/correctness/useExhaustiveDependencies: 仅监听筛选变化
  useEffect(() => {
    const query = buildQuery()

    // 首次请求
    if (!hasFetched.current) {
      hasFetched.current = true
      prevFilter.current = {
        searchQuery,
        statusFilter,
        createdAfter,
        createdBefore,
      }
      onFilterChange(query)
      return
    }

    // 检查筛选是否真正变化
    const filterChanged =
      prevFilter.current.searchQuery !== searchQuery ||
      prevFilter.current.statusFilter !== statusFilter ||
      prevFilter.current.createdAfter !== createdAfter ||
      prevFilter.current.createdBefore !== createdBefore

    if (!filterChanged) return

    // 筛选变化，防抖
    prevFilter.current = {
      searchQuery,
      statusFilter,
      createdAfter,
      createdBefore,
    }
    const timer = setTimeout(() => {
      onFilterChange(query)
    }, debounceMs)
    return () => clearTimeout(timer)
  }, [searchQuery, statusFilter, createdAfter, createdBefore])

  const clearDateFilters = useCallback(() => {
    setCreatedAfter(undefined)
    setCreatedBefore(undefined)
  }, [])

  return {
    // State
    searchQuery,
    statusFilter,
    createdAfter,
    createdBefore,
    // Setters
    setSearchQuery,
    setStatusFilter,
    setCreatedAfter,
    setCreatedBefore,
    clearDateFilters,
    // Helpers
    buildQuery,
  }
}
