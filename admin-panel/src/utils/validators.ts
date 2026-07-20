/**
 * 表单验证工具函数
 */

// TypeScript 全局类型声明
declare global {
  interface Window {
    __APP_CONFIG__?: {
      basePath?: string
      adminRoutePrefix?: string
      healthRoutePrefix?: string
      shortlinkerVersion?: string
    }
  }
}

/**
 * 危险协议列表
 */
const DANGEROUS_PROTOCOLS = [
  'javascript:',
  'data:',
  'file:',
  'vbscript:',
  'about:',
  'blob:',
]

/**
 * 静态保留字列表 - 前端路由路径（固定）
 */
const STATIC_RESERVED_CODES = [
  'login',
  'dashboard',
  'links',
  'analytics',
  'settings',
] as const

/**
 * 提取路径的第一段（去掉前导斜杠）
 * 例如: '/admin/links' → 'admin', '/health' → 'health'
 */
function extractFirstSegment(path: string): string | null {
  const normalized = path.startsWith('/') ? path.slice(1) : path
  const firstSegment = normalized.split('/')[0]
  return firstSegment || null
}

/**
 * 获取完整的保留字列表（静态 + 动态）
 * 动态部分从 window.__APP_CONFIG__ 运行时配置读取
 * @returns 保留字数组（小写）
 */
export function getReservedShortCodes(): string[] {
  const reserved: string[] = [...STATIC_RESERVED_CODES]

  // 从运行时配置读取 API 前缀
  if (typeof window !== 'undefined' && window.__APP_CONFIG__) {
    const config = window.__APP_CONFIG__

    const adminSegment = extractFirstSegment(
      config.adminRoutePrefix || '/admin',
    )
    const healthSegment = extractFirstSegment(
      config.healthRoutePrefix || '/health',
    )

    if (adminSegment && !reserved.includes(adminSegment)) {
      reserved.push(adminSegment)
    }
    if (healthSegment && !reserved.includes(healthSegment)) {
      reserved.push(healthSegment)
    }
  } else {
    // 非浏览器环境或配置未加载，使用默认值
    if (!reserved.includes('admin')) reserved.push('admin')
    if (!reserved.includes('health')) reserved.push('health')
  }

  return reserved
}

/**
 * 验证是否是有效的 URL
 * @param url - URL 字符串
 * @returns 是否有效
 */
export function isValidUrl(url: string): boolean {
  if (!url || typeof url !== 'string') {
    return false
  }

  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

/**
 * URL 验证结果
 */
export interface UrlValidationResult {
  isValid: boolean
  error?: string
}

/**
 * 验证是否是安全的 HTTP/HTTPS URL
 * @param url - URL 字符串
 * @returns 验证结果
 */
export function validateSafeUrl(url: string): UrlValidationResult {
  if (!url || typeof url !== 'string') {
    return { isValid: false, error: 'URL is required' }
  }

  const trimmedUrl = url.trim().toLowerCase()

  // 检查危险协议
  for (const proto of DANGEROUS_PROTOCOLS) {
    if (trimmedUrl.startsWith(proto)) {
      return { isValid: false, error: `Dangerous protocol blocked: ${proto}` }
    }
  }

  try {
    const parsed = new URL(url)

    // 只允许 http 和 https
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
      return {
        isValid: false,
        error: `Invalid protocol: ${parsed.protocol}. Only http:// and https:// are allowed`,
      }
    }

    return { isValid: true }
  } catch {
    return { isValid: false, error: 'Invalid URL format' }
  }
}

/**
 * 验证是否是有效的 HTTP/HTTPS URL（向后兼容）
 * @param url - URL 字符串
 * @returns 是否有效
 */
export function isValidHttpUrl(url: string): boolean {
  return validateSafeUrl(url).isValid
}

/**
 * 验证是否是有效的短链接代码（仅检查格式）
 * 允许字母、数字、下划线、短横线、点号，以及 / 作为路径分隔符
 *
 * 规则：
 * - 不能以 / 开头或结尾
 * - 不能有连续的 /
 * - 每段路径只允许字母、数字、下划线、短横线、点号
 *
 * 注意：此函数不检查保留字，使用 validateShortCode() 进行完整验证
 *
 * @param code - 短链接代码
 * @param minLength - 最小长度（默认 1）
 * @param maxLength - 最大长度（默认 128）
 * @returns 是否有效
 */
export function isValidShortCode(
  code: string,
  minLength: number = 1,
  maxLength: number = 128,
): boolean {
  if (!code || typeof code !== 'string') {
    return false
  }

  if (code.length < minLength || code.length > maxLength) {
    return false
  }

  // 允许字母、数字、下划线、短横线、点号，以及 / 作为路径分隔符
  // 不能以 / 开头或结尾，不能有连续的 /
  const pattern = /^[a-zA-Z0-9_.-]+(?:\/[a-zA-Z0-9_.-]+)*$/
  return pattern.test(code)
}

/**
 * 检查短链接代码是否为保留字（大小写不敏感）
 * 对于带路径的 code（如 admin/test），检查第一段是否为保留字
 * @param code - 短链接代码
 * @returns 是否为保留字
 */
export function isReservedShortCode(code: string): boolean {
  if (!code || typeof code !== 'string') {
    return false
  }

  // 提取第一段路径进行检查（如 admin/test -> admin）
  const firstSegment = code.split('/')[0].toLowerCase()
  const reserved = getReservedShortCodes()
  return reserved.includes(firstSegment)
}

/**
 * 短链接代码验证结果
 */
export interface ShortCodeValidationResult {
  isValid: boolean
  error?: string
}

/**
 * 验证短链接代码（包含格式检查和保留字检查）
 * @param code - 短链接代码
 * @param minLength - 最小长度（默认 1）
 * @param maxLength - 最大长度（默认 128）
 * @returns 验证结果
 */
export function validateShortCode(
  code: string,
  minLength: number = 1,
  maxLength: number = 128,
): ShortCodeValidationResult {
  // 基本格式检查
  if (!isValidShortCode(code, minLength, maxLength)) {
    return {
      isValid: false,
      error:
        'Invalid short code format. Use only letters, numbers, underscores, hyphens, dots and slashes',
    }
  }

  // 保留字检查
  if (isReservedShortCode(code)) {
    return {
      isValid: false,
      error: `"${code}" is a reserved keyword and cannot be used`,
    }
  }

  return { isValid: true }
}

/**
 * 验证密码强度
 * @param password - 密码字符串
 * @returns 密码强度对象
 */
export interface PasswordStrength {
  isValid: boolean
  score: number // 0-4，0最弱，4最强
  feedback: string[]
}

export function validatePasswordStrength(password: string): PasswordStrength {
  const feedback: string[] = []
  let score = 0

  if (!password || typeof password !== 'string') {
    return {
      isValid: false,
      score: 0,
      feedback: ['Password is required'],
    }
  }

  // 长度检查
  if (password.length < 8) {
    feedback.push('Password should be at least 8 characters')
  } else {
    score++
  }

  if (password.length >= 12) {
    score++
  }

  // 包含小写字母
  if (/[a-z]/.test(password)) {
    score++
  } else {
    feedback.push('Password should contain lowercase letters')
  }

  // 包含大写字母
  if (/[A-Z]/.test(password)) {
    score++
  } else {
    feedback.push('Password should contain uppercase letters')
  }

  // 包含数字
  if (/\d/.test(password)) {
    score++
  } else {
    feedback.push('Password should contain numbers')
  }

  // 包含特殊字符
  if (/[!@#$%^&*(),.?":{}|<>]/.test(password)) {
    score++
  } else {
    feedback.push('Password should contain special characters')
  }

  // 限制分数最大值为 4
  score = Math.min(score, 4)

  return {
    isValid: password.length >= 8 && score >= 2,
    score,
    feedback,
  }
}

/**
 * 验证邮箱地址
 * @param email - 邮箱字符串
 * @returns 是否有效
 */
export function isValidEmail(email: string): boolean {
  if (!email || typeof email !== 'string') {
    return false
  }

  // 简单的邮箱验证正则
  const pattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return pattern.test(email)
}

/**
 * 验证日期是否在有效范围内
 * @param date - 日期字符串
 * @param minDate - 最小日期（可选）
 * @param maxDate - 最大日期（可选）
 * @returns 是否有效
 */
export function isValidDate(
  date: string,
  minDate?: string,
  maxDate?: string,
): boolean {
  if (!date || typeof date !== 'string') {
    return false
  }

  try {
    const dateObj = new Date(date)

    // 检查是否是有效日期
    if (Number.isNaN(dateObj.getTime())) {
      return false
    }

    // 检查最小日期
    if (minDate) {
      const minDateObj = new Date(minDate)
      if (dateObj < minDateObj) {
        return false
      }
    }

    // 检查最大日期
    if (maxDate) {
      const maxDateObj = new Date(maxDate)
      if (dateObj > maxDateObj) {
        return false
      }
    }

    return true
  } catch {
    return false
  }
}

/**
 * 验证字符串是否为空或只包含空白字符
 * @param str - 字符串
 * @returns 是否为空
 */
export function isEmpty(str: string | null | undefined): boolean {
  return !str || str.trim().length === 0
}

/**
 * 验证字符串长度是否在范围内
 * @param str - 字符串
 * @param min - 最小长度
 * @param max - 最大长度
 * @returns 是否在范围内
 */
export function isLengthInRange(
  str: string,
  min: number,
  max: number,
): boolean {
  if (!str || typeof str !== 'string') {
    return false
  }

  const length = str.length
  return length >= min && length <= max
}
