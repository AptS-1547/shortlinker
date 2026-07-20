import { qrcodeLogger } from '@/utils/logger'
import { ApiError, config } from './http'
import type { QRCodeOptions } from './types'

export class QRCodeService {
  /**
   * 生成二维码
   */
  async generate(url: string, options: QRCodeOptions = {}): Promise<string> {
    const {
      size = 200,
      margin = 4,
      errorCorrectionLevel = 'M',
      color = {},
    } = options
    const { dark = '#000000', light = '#FFFFFF' } = color

    try {
      const QRCode = await import('qrcode')

      return await QRCode.toDataURL(url, {
        width: size,
        margin,
        errorCorrectionLevel,
        type: 'image/png',
        color: {
          dark,
          light,
        },
      })
    } catch (error) {
      qrcodeLogger.error('Failed to generate QR code:', error)
      // 备用方案：使用在线服务
      const encodedUrl = encodeURIComponent(url)
      const darkColor = (color?.dark ?? '#000000').replace('#', '')
      const lightColor = (color?.light ?? '#FFFFFF').replace('#', '')
      return `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodedUrl}&format=png&ecc=${errorCorrectionLevel}&bgcolor=${lightColor}&color=${darkColor}&qzone=${margin}`
    }
  }

  /**
   * 为短链接生成二维码
   */
  async generateForShortLink(
    shortCode: string,
    baseUrl?: string,
    options: QRCodeOptions = {},
  ): Promise<string> {
    const domain =
      baseUrl ||
      config.baseUrl ||
      (typeof window !== 'undefined' ? window.location.origin : '')

    // QR Code 链接自动添加 utm_source=qrcode 用于流量来源追踪
    const fullUrl = `${domain}/${shortCode}?utm_source=qrcode`
    return this.generate(fullUrl, options)
  }

  /**
   * 下载二维码
   */
  async download(
    dataUrl: string,
    filename: string = 'qrcode.png',
  ): Promise<void> {
    try {
      let downloadUrl: string

      if (dataUrl.startsWith('data:')) {
        downloadUrl = dataUrl
      } else {
        const response = await fetch(dataUrl)
        const blob = await response.blob()
        downloadUrl = window.URL.createObjectURL(blob)
      }

      const link = document.createElement('a')
      link.href = downloadUrl
      link.download = filename
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)

      if (!dataUrl.startsWith('data:')) {
        window.URL.revokeObjectURL(downloadUrl)
      }
    } catch (error) {
      qrcodeLogger.error('Failed to download QR code:', error)
      throw new ApiError('Failed to download QR code')
    }
  }
}

export const qrcodeService = new QRCodeService()
