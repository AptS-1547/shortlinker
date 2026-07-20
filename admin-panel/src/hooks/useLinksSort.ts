import { useCallback, useMemo, useState } from 'react'
import type { LinkResponse } from '@/services/types'

export type SortField = 'clicks' | 'created_at' | null
export type SortDirection = 'asc' | 'desc'

export function useLinksSort(links: LinkResponse[]) {
  const [sortField, setSortField] = useState<SortField>(null)
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc')

  const handleSort = useCallback(
    (field: SortField) => {
      if (sortField === field) {
        setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'))
      } else {
        setSortField(field)
        setSortDirection('desc')
      }
    },
    [sortField],
  )

  const sortedLinks = useMemo(() => {
    if (!sortField) return links

    return [...links].sort((a, b) => {
      let comparison = 0
      if (sortField === 'clicks') {
        comparison = (a.click_count || 0) - (b.click_count || 0)
      } else if (sortField === 'created_at') {
        comparison =
          new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
      }
      return sortDirection === 'asc' ? comparison : -comparison
    })
  }, [links, sortField, sortDirection])

  return {
    sortField,
    sortDirection,
    sortedLinks,
    handleSort,
  }
}
