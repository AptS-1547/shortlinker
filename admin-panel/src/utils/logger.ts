/**
 * 环境感知的日志工具
 * 生产环境静默，开发环境输出
 */

const isDev = import.meta.env.DEV

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface Logger {
  debug: (...args: unknown[]) => void
  info: (...args: unknown[]) => void
  warn: (...args: unknown[]) => void
  error: (...args: unknown[]) => void
}

function createLogger(prefix: string): Logger {
  const log = (level: LogLevel, ...args: unknown[]) => {
    if (!isDev && level !== 'error') return

    const timestamp = new Date().toISOString().slice(11, 23)
    const tag = `[${timestamp}] [${prefix}]`

    switch (level) {
      case 'debug':
        console.debug(tag, ...args)
        break
      case 'info':
        console.log(tag, ...args)
        break
      case 'warn':
        console.warn(tag, ...args)
        break
      case 'error':
        console.error(tag, ...args)
        break
    }
  }

  return {
    debug: (...args) => log('debug', ...args),
    info: (...args) => log('info', ...args),
    warn: (...args) => log('warn', ...args),
    error: (...args) => log('error', ...args),
  }
}

// 预定义的 logger 实例
export const authLogger = createLogger('Auth')
export const httpLogger = createLogger('HTTP')
export const configLogger = createLogger('Config')
export const linksLogger = createLogger('Links')
export const routerLogger = createLogger('Router')
export const i18nLogger = createLogger('i18n')
export const storageLogger = createLogger('Storage')
export const dateLogger = createLogger('Date')
export const urlLogger = createLogger('URL')
export const clipboardLogger = createLogger('Clipboard')
export const qrcodeLogger = createLogger('QRCode')
export const healthLogger = createLogger('Health')
export const dashboardLogger = createLogger('Dashboard')
export const hookLogger = createLogger('Hook')
export const pwaLogger = createLogger('PWA')

// 通用 logger（不建议使用，优先用具名 logger）
export const logger = createLogger('App')

export { createLogger }
