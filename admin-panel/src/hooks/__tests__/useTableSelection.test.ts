import { act, renderHook } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { useTableSelection } from '../useTableSelection'

interface TestItem {
  id: string
  name: string
}

const createMockItems = (): TestItem[] => [
  { id: '1', name: 'Item 1' },
  { id: '2', name: 'Item 2' },
  { id: '3', name: 'Item 3' },
]

const getItemId = (item: TestItem) => item.id

describe('useTableSelection', () => {
  // ==========================================================================
  // Initial state
  // ==========================================================================

  describe('initial state', () => {
    it('should have empty selection', () => {
      const { result } = renderHook(() =>
        useTableSelection(createMockItems(), { getItemId }),
      )
      expect(result.current.selected.size).toBe(0)
    })

    it('should have isAllSelected as false', () => {
      const { result } = renderHook(() =>
        useTableSelection(createMockItems(), { getItemId }),
      )
      expect(result.current.isAllSelected).toBe(false)
    })

    it('should have isSomeSelected as false', () => {
      const { result } = renderHook(() =>
        useTableSelection(createMockItems(), { getItemId }),
      )
      expect(result.current.isSomeSelected).toBe(false)
    })

    it('should return false for isSelected on any item', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )
      expect(result.current.isSelected(items[0])).toBe(false)
    })
  })

  // ==========================================================================
  // toggle
  // ==========================================================================

  describe('toggle', () => {
    it('should select an item', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
      })

      expect(result.current.isSelected(items[0])).toBe(true)
      expect(result.current.selected.has('1')).toBe(true)
    })

    it('should deselect a selected item', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
      })
      act(() => {
        result.current.toggle(items[0])
      })

      expect(result.current.isSelected(items[0])).toBe(false)
    })

    it('should toggle multiple items independently', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
        result.current.toggle(items[2])
      })

      expect(result.current.isSelected(items[0])).toBe(true)
      expect(result.current.isSelected(items[1])).toBe(false)
      expect(result.current.isSelected(items[2])).toBe(true)
    })
  })

  // ==========================================================================
  // toggleAll
  // ==========================================================================

  describe('toggleAll', () => {
    it('should select all items when none selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggleAll()
      })

      expect(result.current.isAllSelected).toBe(true)
      expect(result.current.selected.size).toBe(3)
    })

    it('should deselect all items when all selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggleAll()
      })
      act(() => {
        result.current.toggleAll()
      })

      expect(result.current.isAllSelected).toBe(false)
      expect(result.current.selected.size).toBe(0)
    })

    it('should select all when some are selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
      })
      act(() => {
        result.current.toggleAll()
      })

      expect(result.current.isAllSelected).toBe(true)
      expect(result.current.selected.size).toBe(3)
    })
  })

  // ==========================================================================
  // selectAll and deselectAll
  // ==========================================================================

  describe('selectAll', () => {
    it('should select all items', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })

      expect(result.current.isAllSelected).toBe(true)
      items.forEach((item) => {
        expect(result.current.isSelected(item)).toBe(true)
      })
    })

    it('should keep all selected if already all selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })
      act(() => {
        result.current.selectAll()
      })

      expect(result.current.isAllSelected).toBe(true)
    })
  })

  describe('deselectAll', () => {
    it('should deselect all items', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })
      act(() => {
        result.current.deselectAll()
      })

      expect(result.current.selected.size).toBe(0)
      expect(result.current.isAllSelected).toBe(false)
    })

    it('should work when nothing is selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.deselectAll()
      })

      expect(result.current.selected.size).toBe(0)
    })
  })

  // ==========================================================================
  // isAllSelected and isSomeSelected
  // ==========================================================================

  describe('isAllSelected', () => {
    it('should return false for empty items', () => {
      const { result } = renderHook(() => useTableSelection([], { getItemId }))
      expect(result.current.isAllSelected).toBe(false)
    })

    it('should return true when all items selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })

      expect(result.current.isAllSelected).toBe(true)
    })

    it('should return false when some items selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
      })

      expect(result.current.isAllSelected).toBe(false)
    })
  })

  describe('isSomeSelected', () => {
    it('should return false when none selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )
      expect(result.current.isSomeSelected).toBe(false)
    })

    it('should return true when some but not all selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
      })

      expect(result.current.isSomeSelected).toBe(true)
    })

    it('should return false when all selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })

      expect(result.current.isSomeSelected).toBe(false)
    })
  })

  // ==========================================================================
  // getSelected
  // ==========================================================================

  describe('getSelected', () => {
    it('should return empty array when none selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      expect(result.current.getSelected()).toEqual([])
    })

    it('should return selected items', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
        result.current.toggle(items[2])
      })

      const selected = result.current.getSelected()
      expect(selected).toHaveLength(2)
      expect(selected).toContainEqual(items[0])
      expect(selected).toContainEqual(items[2])
    })

    it('should return all items when all selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })

      expect(result.current.getSelected()).toEqual(items)
    })
  })

  // ==========================================================================
  // getSelectedCount
  // ==========================================================================

  describe('getSelectedCount', () => {
    it('should return 0 when none selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      expect(result.current.getSelectedCount()).toBe(0)
    })

    it('should return correct count when some selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.toggle(items[0])
        result.current.toggle(items[1])
      })

      expect(result.current.getSelectedCount()).toBe(2)
    })

    it('should return total count when all selected', () => {
      const items = createMockItems()
      const { result } = renderHook(() =>
        useTableSelection(items, { getItemId }),
      )

      act(() => {
        result.current.selectAll()
      })

      expect(result.current.getSelectedCount()).toBe(3)
    })
  })

  // ==========================================================================
  // Items change
  // ==========================================================================

  describe('items change', () => {
    it('should maintain selection when items update', () => {
      const items = createMockItems()
      const { result, rerender } = renderHook(
        ({ items }) => useTableSelection(items, { getItemId }),
        { initialProps: { items } },
      )

      act(() => {
        result.current.toggle(items[0])
      })

      const newItems = [...items, { id: '4', name: 'Item 4' }]
      rerender({ items: newItems })

      expect(result.current.isSelected(items[0])).toBe(true)
      expect(result.current.selected.has('1')).toBe(true)
    })
  })

  // ==========================================================================
  // Numeric IDs
  // ==========================================================================

  describe('numeric IDs', () => {
    interface NumericItem {
      id: number
      name: string
    }

    const numericItems: NumericItem[] = [
      { id: 1, name: 'Item 1' },
      { id: 2, name: 'Item 2' },
    ]

    it('should work with numeric IDs', () => {
      const { result } = renderHook(() =>
        useTableSelection(numericItems, { getItemId: (item) => item.id }),
      )

      act(() => {
        result.current.toggle(numericItems[0])
      })

      expect(result.current.isSelected(numericItems[0])).toBe(true)
      expect(result.current.selected.has(1)).toBe(true)
    })
  })
})
