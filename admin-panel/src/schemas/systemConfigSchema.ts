import { z } from 'zod'
import type { ValueType } from '@/services/types'

// 为了兼容性保留，使用后端生成的 ValueType
export type ConfigValueType = ValueType

/**
 * 根据配置类型创建动态验证 schema
 */
export function createConfigValueSchema(valueType: ConfigValueType) {
  switch (valueType) {
    case 'bool':
      // bool 类型只接受 'true' 或 'false'
      return z.enum(['true', 'false'], {
        message: 'Must be "true" or "false"',
      })

    case 'int':
      // int 类型必须是有效的整数字符串
      return z
        .string()
        .regex(/^-?\d+$/, 'Must be a valid integer (e.g., 42, -10)')
        .refine(
          (val) => {
            const num = Number.parseInt(val, 10)
            return !Number.isNaN(num)
          },
          { message: 'Must be a valid integer' },
        )

    case 'float':
      // float 类型必须是有效的浮点数字符串
      return z
        .string()
        .regex(
          /^-?(\d+\.?\d*|\d*\.?\d+)$/,
          'Must be a valid number (e.g., 3.14, -2.5, 42)',
        )
        .refine(
          (val) => {
            const num = Number.parseFloat(val)
            return !Number.isNaN(num) && Number.isFinite(num)
          },
          { message: 'Must be a valid number' },
        )

    case 'json':
      // json 类型必须是有效的 JSON 字符串
      return z.string().refine(
        (val) => {
          if (val.trim() === '') {
            return false
          }
          try {
            JSON.parse(val)
            return true
          } catch {
            return false
          }
        },
        { message: 'Must be valid JSON' },
      )

    case 'stringarray':
    case 'enumarray':
      // 数组类型必须是有效的 JSON 字符串数组
      return z.string().refine(
        (val) => {
          try {
            const arr = JSON.parse(val)
            return (
              Array.isArray(arr) &&
              arr.every((item) => typeof item === 'string')
            )
          } catch {
            return false
          }
        },
        { message: 'Must be a valid string array' },
      )

    default:
      // string 类型接受任何字符串
      return z.string()
  }
}

/**
 * 配置编辑表单的 schema
 */
export function createConfigFormSchema(valueType: ConfigValueType) {
  return z.object({
    value: createConfigValueSchema(valueType),
  })
}
