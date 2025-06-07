import { config, ApiError } from './http'
import type { QRCodeOptions } from './types'

export class QRCodeService {
  /**
   * 生成二维码
   */
  async generate(url: string, options: QRCodeOptions = {}): Promise<string> {
    const { size = 200, margin = 4, errorCorrectionLevel = 'M' } = options

    try {
      const QRCode = await import('qrcode')

      return await QRCode.toDataURL(url, {
        width: size,
        margin,
        errorCorrectionLevel,
        type: 'image/png',
        quality: 1.0,
        color: {
          dark: '#000000',
          light: '#FFFFFF',
        },
        rendererOpts: {
          quality: 1.0,
          scale: Math.max(4, Math.ceil(size / 100)),
          compressionLevel: 6,
          filters: 'all',
          palette: false,
          backgroundIndex: 0,
          resolution: 300,
        },
      })
    } catch (error) {
      console.error('Failed to generate QR code:', error)
      // 备用方案：使用在线服务
      const encodedUrl = encodeURIComponent(url)
      return `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodedUrl}&format=png&ecc=${errorCorrectionLevel}&bgcolor=ffffff&color=000000&qzone=4`
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
      baseUrl || config.baseUrl || (typeof window !== 'undefined' ? window.location.origin : '')

    const fullUrl = `${domain}/${shortCode}`
    return this.generate(fullUrl, options)
  }

  /**
   * 下载二维码
   */
  async download(dataUrl: string, filename: string = 'qrcode.png'): Promise<void> {
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
      console.error('Failed to download QR code:', error)
      throw new ApiError('Failed to download QR code')
    }
  }
}

export const qrcodeService = new QRCodeService()
