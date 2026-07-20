import { useCallback, useMemo, useState } from 'react'

export interface PaginationOptions {
  initialPage?: number
  initialPageSize?: number
  total?: number
}

export function usePagination(options: PaginationOptions = {}) {
  const [currentPage, setCurrentPage] = useState(options.initialPage || 1)
  const [pageSize, setPageSizeState] = useState(options.initialPageSize || 20)
  const [total, setTotalState] = useState(options.total || 0)

  const totalPages = useMemo(() => {
    return Math.ceil(total / pageSize)
  }, [total, pageSize])

  const hasNext = useMemo(() => {
    return currentPage < totalPages
  }, [currentPage, totalPages])

  const hasPrev = useMemo(() => {
    return currentPage > 1
  }, [currentPage])

  const startIndex = useMemo(() => {
    return (currentPage - 1) * pageSize + 1
  }, [currentPage, pageSize])

  const endIndex = useMemo(() => {
    return Math.min(currentPage * pageSize, total)
  }, [currentPage, pageSize, total])

  const getVisiblePages = useCallback(
    (delta: number = 2) => {
      let start = Math.max(1, currentPage - delta)
      let end = Math.min(totalPages, currentPage + delta)

      const minVisible = Math.min(5, totalPages)
      if (end - start + 1 < minVisible) {
        if (start === 1) {
          end = Math.min(totalPages, start + minVisible - 1)
        } else if (end === totalPages) {
          start = Math.max(1, end - minVisible + 1)
        }
      }

      const pages: number[] = []
      for (let i = start; i <= end; i++) {
        pages.push(i)
      }
      return pages
    },
    [currentPage, totalPages],
  )

  const goToPage = useCallback(
    (page: number) => {
      if (page >= 1 && page <= totalPages) {
        setCurrentPage(page)
      }
    },
    [totalPages],
  )

  const nextPage = useCallback(() => {
    if (currentPage < totalPages) {
      setCurrentPage((prev) => prev + 1)
    }
  }, [currentPage, totalPages])

  const prevPage = useCallback(() => {
    if (currentPage > 1) {
      setCurrentPage((prev) => prev - 1)
    }
  }, [currentPage])

  const setPageSize = useCallback(
    (size: number) => {
      setPageSizeState(size)
      const maxPage = Math.ceil(total / size)
      if (currentPage > maxPage) {
        setCurrentPage(Math.max(1, maxPage))
      }
    },
    [total, currentPage],
  )

  const setTotal = useCallback((value: number) => {
    setTotalState(value)
  }, [])

  const reset = useCallback(() => {
    setCurrentPage(options.initialPage || 1)
    setPageSizeState(options.initialPageSize || 20)
    setTotalState(options.total || 0)
  }, [options.initialPage, options.initialPageSize, options.total])

  return {
    currentPage,
    pageSize,
    total,
    totalPages,
    hasNext,
    hasPrev,
    startIndex,
    endIndex,
    getVisiblePages,
    goToPage,
    nextPage,
    prevPage,
    setPageSize,
    setTotal,
    reset,
  }
}
