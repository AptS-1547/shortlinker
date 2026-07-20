/**
 * 剪贴板操作工具函数
 */

import { clipboardLogger } from './logger'

/**
 * 复制文本到剪贴板的结果
 */
export interface CopyResult {
  success: boolean
  error?: Error
}

/**
 * 复制文本到剪贴板
 * @param text - 要复制的文本
 * @returns 复制结果
 */
export async function copyToClipboard(text: string): Promise<CopyResult> {
  try {
    // 优先使用现代 Clipboard API
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text)
      return { success: true }
    }

    // 降级方案：使用旧的 execCommand 方法
    const textArea = document.createElement('textarea')
    textArea.value = text
    textArea.style.position = 'fixed'
    textArea.style.left = '-999999px'
    textArea.style.top = '-999999px'
    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()

    try {
      const successful = document.execCommand('copy')
      document.body.removeChild(textArea)

      if (successful) {
        return { success: true }
      } else {
        return {
          success: false,
          error: new Error('execCommand("copy") failed'),
        }
      }
    } catch (error) {
      document.body.removeChild(textArea)
      return {
        success: false,
        error: error instanceof Error ? error : new Error('Unknown error'),
      }
    }
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error : new Error('Unknown error'),
    }
  }
}

/**
 * 从剪贴板读取文本
 * @returns 剪贴板中的文本，失败则返回 null
 */
export async function readFromClipboard(): Promise<string | null> {
  try {
    if (navigator.clipboard?.readText) {
      return await navigator.clipboard.readText()
    }

    // 旧浏览器不支持读取剪贴板
    clipboardLogger.warn('Clipboard API not available')
    return null
  } catch (error) {
    clipboardLogger.error('Failed to read from clipboard:', error)
    return null
  }
}

/**
 * 检查是否支持剪贴板操作
 * @returns 是否支持
 */
export function isClipboardSupported(): boolean {
  return !!navigator.clipboard?.writeText
}
