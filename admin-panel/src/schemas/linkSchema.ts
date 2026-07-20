import { z } from 'zod'
import {
  isReservedShortCode,
  isValidHttpUrl,
  isValidShortCode,
} from '@/utils/validators'

/**
 * 链接表单验证 Schema
 */
export const linkSchema = z.object({
  code: z
    .string()
    .optional()
    .refine((val) => !val || isValidShortCode(val), {
      message:
        'Invalid short code format. Use letters, numbers, underscores, hyphens, or paths like abc/def',
    })
    .refine((val) => !val || !isReservedShortCode(val), {
      message: 'This code is reserved and cannot be used',
    }),
  target: z.string().min(1, 'Target URL is required').refine(isValidHttpUrl, {
    message: 'Invalid URL format. Must be http:// or https://',
  }),
  password: z.string().nullable().optional(),
  expires_at: z
    .string()
    .nullable()
    .optional()
    .refine((val) => !val || new Date(val) > new Date(), {
      message: 'Expiration date must be in the future',
    }),
  force: z.boolean().optional(),
})

export type LinkFormData = z.infer<typeof linkSchema>
