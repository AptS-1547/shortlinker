import * as React from 'react'
import { FiPlus, FiX } from 'react-icons/fi'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'

interface TagInputProps {
  value: string[]
  onChange: (value: string[]) => void
  placeholder?: string
  disabled?: boolean
  className?: string
  /** Validate a single tag before adding */
  validate?: (tag: string) => boolean | string
  /** Maximum number of tags allowed */
  maxTags?: number
}

function TagInput({
  value,
  onChange,
  placeholder = 'Add item...',
  disabled = false,
  className,
  validate,
  maxTags,
}: TagInputProps) {
  const [inputValue, setInputValue] = React.useState('')
  const [error, setError] = React.useState<string | null>(null)
  const [editingIndex, setEditingIndex] = React.useState<number | null>(null)
  const [editValue, setEditValue] = React.useState('')
  const inputRef = React.useRef<HTMLInputElement>(null)
  const editInputRef = React.useRef<HTMLInputElement>(null)

  // Focus edit input when entering edit mode
  React.useEffect(() => {
    if (editingIndex !== null) {
      editInputRef.current?.focus()
      editInputRef.current?.select()
    }
  }, [editingIndex])

  const handleAdd = () => {
    const trimmed = inputValue.trim()
    if (!trimmed) return

    // Duplicate check
    if (value.includes(trimmed)) {
      setError('Item already exists')
      return
    }

    // Max tags check
    if (maxTags && value.length >= maxTags) {
      setError(`Maximum ${maxTags} items allowed`)
      return
    }

    // Custom validation
    if (validate) {
      const result = validate(trimmed)
      if (result !== true) {
        setError(typeof result === 'string' ? result : 'Invalid item')
        return
      }
    }

    setError(null)
    onChange([...value, trimmed])
    setInputValue('')
    inputRef.current?.focus()
  }

  const handleRemove = (index: number) => {
    const newValue = value.filter((_, i) => i !== index)
    onChange(newValue)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleAdd()
    } else if (e.key === 'Backspace' && !inputValue && value.length > 0) {
      // Remove last tag on backspace when input is empty
      handleRemove(value.length - 1)
    }
  }

  const handleStartEdit = (index: number) => {
    if (disabled) return
    setEditingIndex(index)
    setEditValue(value[index])
    setError(null)
  }

  const handleSaveEdit = () => {
    if (editingIndex === null) return

    const trimmed = editValue.trim()
    if (!trimmed) {
      // Empty value - remove the tag
      handleRemove(editingIndex)
      setEditingIndex(null)
      setEditValue('')
      return
    }

    // Check for duplicate (excluding current item)
    const isDuplicate = value.some(
      (item, i) => i !== editingIndex && item === trimmed,
    )
    if (isDuplicate) {
      setError('Item already exists')
      return
    }

    // Custom validation
    if (validate) {
      const result = validate(trimmed)
      if (result !== true) {
        setError(typeof result === 'string' ? result : 'Invalid item')
        return
      }
    }

    // Save the edit
    const newValue = [...value]
    newValue[editingIndex] = trimmed
    onChange(newValue)
    setEditingIndex(null)
    setEditValue('')
    setError(null)
  }

  const handleCancelEdit = () => {
    setEditingIndex(null)
    setEditValue('')
    setError(null)
  }

  const handleEditKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleSaveEdit()
    } else if (e.key === 'Escape') {
      e.preventDefault()
      handleCancelEdit()
    }
  }

  return (
    <div data-slot="tag-input" className={cn('space-y-2', className)}>
      {/* Tag list */}
      {value.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {value.map((item, index) => (
            <Badge
              key={item}
              variant="secondary"
              className="flex items-center gap-1 px-2 py-1"
            >
              {editingIndex === index ? (
                <input
                  ref={editInputRef}
                  type="text"
                  value={editValue}
                  onChange={(e) => {
                    setEditValue(e.target.value)
                    setError(null)
                  }}
                  onKeyDown={handleEditKeyDown}
                  onBlur={handleSaveEdit}
                  className="w-24 bg-transparent font-mono text-xs outline-none"
                  aria-label="Edit tag value"
                />
              ) : (
                <button
                  type="button"
                  onClick={() => handleStartEdit(index)}
                  className="cursor-text font-mono text-xs hover:underline"
                  disabled={disabled}
                  title="Click to edit"
                >
                  {item}
                </button>
              )}
              {!disabled && editingIndex !== index && (
                <button
                  type="button"
                  onClick={() => handleRemove(index)}
                  className="ml-1 rounded-full p-0.5 hover:bg-destructive/20 focus:outline-none"
                  aria-label={`Remove ${item}`}
                >
                  <FiX className="size-3" />
                </button>
              )}
            </Badge>
          ))}
        </div>
      )}

      {/* Input area */}
      {!disabled && (
        <div className="flex gap-2">
          <Input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => {
              setInputValue(e.target.value)
              setError(null)
            }}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            className="flex-1 font-mono text-sm"
            aria-invalid={!!error}
          />
          <Button
            type="button"
            variant="outline"
            size="icon-sm"
            onClick={handleAdd}
            disabled={!inputValue.trim()}
          >
            <FiPlus className="size-4" />
          </Button>
        </div>
      )}

      {/* Error message */}
      {error && <p className="text-sm text-destructive">{error}</p>}
    </div>
  )
}

export { TagInput, type TagInputProps }
