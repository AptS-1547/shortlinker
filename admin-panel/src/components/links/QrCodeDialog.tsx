import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiDownload as Download,
  FiLoader as Loader,
  FiX as X,
} from 'react-icons/fi'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { qrcodeService } from '@/services/qrcodeService'
import type { QRCodeOptions } from '@/services/types'

export interface QrCodeDialogProps {
  code: string | null
  onClose: () => void
}

type ErrorCorrectionLevel = 'L' | 'M' | 'Q' | 'H'

interface QrConfig {
  size: number
  margin: number
  errorCorrectionLevel: ErrorCorrectionLevel
  darkColor: string
  lightColor: string
  logoFile: File | null
  logoSize: number // logo 占 QR 码的比例 (0.15 - 0.3)
}

const DEFAULT_CONFIG: QrConfig = {
  size: 300,
  margin: 4,
  errorCorrectionLevel: 'H', // Logo 模式需要高纠错
  darkColor: '#000000',
  lightColor: '#FFFFFF',
  logoFile: null,
  logoSize: 0.2,
}

/**
 * 将 QRCode 和 Logo 合成到 Canvas 上
 */
async function compositeQrWithLogo(
  qrDataUrl: string,
  logoFile: File,
  logoSize: number,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const qrImg = new Image()
    qrImg.onload = () => {
      const canvas = document.createElement('canvas')
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        reject(new Error('Canvas not supported'))
        return
      }

      canvas.width = qrImg.width
      canvas.height = qrImg.height

      // 绘制 QR 码
      ctx.drawImage(qrImg, 0, 0)

      // 加载并绘制 Logo
      const logoImg = new Image()
      logoImg.onload = () => {
        const logoWidth = qrImg.width * logoSize
        const logoHeight = qrImg.height * logoSize
        const logoX = (qrImg.width - logoWidth) / 2
        const logoY = (qrImg.height - logoHeight) / 2

        // 绘制白色背景（圆角矩形）
        const padding = 4
        ctx.fillStyle = '#FFFFFF'
        ctx.beginPath()
        const bgX = logoX - padding
        const bgY = logoY - padding
        const bgW = logoWidth + padding * 2
        const bgH = logoHeight + padding * 2
        const radius = 8
        ctx.roundRect(bgX, bgY, bgW, bgH, radius)
        ctx.fill()

        // 绘制 Logo
        ctx.drawImage(logoImg, logoX, logoY, logoWidth, logoHeight)

        resolve(canvas.toDataURL('image/png'))
      }
      logoImg.onerror = () => reject(new Error('Failed to load logo'))
      logoImg.src = URL.createObjectURL(logoFile)
    }
    qrImg.onerror = () => reject(new Error('Failed to load QR code'))
    qrImg.src = qrDataUrl
  })
}

export function QrCodeDialog({ code, onClose }: QrCodeDialogProps) {
  const { t } = useTranslation()
  const [config, setConfig] = useState<QrConfig>(DEFAULT_CONFIG)
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null)
  const [finalDataUrl, setFinalDataUrl] = useState<string | null>(null)
  const [isGenerating, setIsGenerating] = useState(false)
  const [isDownloading, setIsDownloading] = useState(false)
  const [logoPreview, setLogoPreview] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const qrOptions: QRCodeOptions = useMemo(
    () => ({
      size: config.size,
      margin: config.margin,
      errorCorrectionLevel: config.errorCorrectionLevel,
      color: {
        dark: config.darkColor,
        light: config.lightColor,
      },
    }),
    [
      config.size,
      config.margin,
      config.errorCorrectionLevel,
      config.darkColor,
      config.lightColor,
    ],
  )

  // 生成 QRCode（带防抖）
  useEffect(() => {
    if (!code) {
      setQrDataUrl(null)
      setFinalDataUrl(null)
      return
    }

    setIsGenerating(true)
    const timer = setTimeout(async () => {
      try {
        const dataUrl = await qrcodeService.generateForShortLink(
          code,
          undefined,
          qrOptions,
        )
        setQrDataUrl(dataUrl)
      } catch {
        setQrDataUrl(null)
      } finally {
        setIsGenerating(false)
      }
    }, 300)

    return () => clearTimeout(timer)
  }, [code, qrOptions])

  // 合成 Logo（当 QR 码或 Logo 变化时）
  useEffect(() => {
    if (!qrDataUrl) {
      setFinalDataUrl(null)
      return
    }

    if (!config.logoFile) {
      setFinalDataUrl(qrDataUrl)
      return
    }

    setIsGenerating(true)
    compositeQrWithLogo(qrDataUrl, config.logoFile, config.logoSize)
      .then(setFinalDataUrl)
      .catch(() => setFinalDataUrl(qrDataUrl))
      .finally(() => setIsGenerating(false))
  }, [qrDataUrl, config.logoFile, config.logoSize])

  const handleDownload = useCallback(async () => {
    if (!finalDataUrl || !code) return
    setIsDownloading(true)
    try {
      await qrcodeService.download(finalDataUrl, `${code}-qr.png`)
    } finally {
      setIsDownloading(false)
    }
  }, [finalDataUrl, code])

  const handleClose = useCallback(() => {
    setConfig(DEFAULT_CONFIG)
    setLogoPreview(null)
    onClose()
  }, [onClose])

  const updateConfig = useCallback(
    <K extends keyof QrConfig>(key: K, value: QrConfig[K]) => {
      setConfig((prev) => ({ ...prev, [key]: value }))
    },
    [],
  )

  const handleLogoChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0]
      if (file) {
        // 验证文件类型
        if (!file.type.startsWith('image/')) {
          return
        }
        updateConfig('logoFile', file)
        // 如果有 logo，自动切换到高纠错级别
        if (config.errorCorrectionLevel !== 'H') {
          updateConfig('errorCorrectionLevel', 'H')
        }
        // 生成预览
        const reader = new FileReader()
        reader.onload = (ev) => {
          setLogoPreview(ev.target?.result as string)
        }
        reader.readAsDataURL(file)
      }
    },
    [updateConfig, config.errorCorrectionLevel],
  )

  const handleRemoveLogo = useCallback(() => {
    updateConfig('logoFile', null)
    setLogoPreview(null)
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }, [updateConfig])

  return (
    <Dialog open={!!code} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="sm:max-w-md max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('links.qr.preview', 'QR Code Preview')}</DialogTitle>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* QRCode 预览区 */}
          <div className="flex justify-center">
            <div
              className={cn(
                'relative flex items-center justify-center rounded-lg border bg-white p-4',
                'min-h-[200px] min-w-[200px]',
              )}
              style={{ backgroundColor: config.lightColor }}
            >
              {isGenerating ? (
                <Loader className="h-8 w-8 animate-spin text-muted-foreground" />
              ) : finalDataUrl ? (
                <img
                  src={finalDataUrl}
                  alt="QR Code"
                  className="max-h-[200px] max-w-[200px]"
                />
              ) : (
                <span className="text-sm text-muted-foreground">
                  {t('links.qr.generateFailed', 'Failed to generate')}
                </span>
              )}
            </div>
          </div>

          {/* 配置选项 */}
          <div className="space-y-4">
            {/* Logo 上传 */}
            <div className="space-y-2">
              <Label>{t('links.qr.logo', 'Logo (optional)')}</Label>
              {logoPreview ? (
                <div className="flex items-center gap-3">
                  <img
                    src={logoPreview}
                    alt="Logo preview"
                    className="h-12 w-12 rounded border object-contain"
                  />
                  <div className="flex-1 space-y-1">
                    <p className="text-sm text-muted-foreground truncate">
                      {config.logoFile?.name}
                    </p>
                    <div className="flex items-center gap-2">
                      <Label
                        htmlFor="logo-size"
                        className="text-xs text-muted-foreground"
                      >
                        {t('links.qr.logoSize', 'Size')}:
                      </Label>
                      <Select
                        value={String(config.logoSize)}
                        onValueChange={(v) =>
                          updateConfig('logoSize', Number(v))
                        }
                      >
                        <SelectTrigger className="h-7 w-20 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="0.15">15%</SelectItem>
                          <SelectItem value="0.2">20%</SelectItem>
                          <SelectItem value="0.25">25%</SelectItem>
                          <SelectItem value="0.3">30%</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 shrink-0"
                    onClick={handleRemoveLogo}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                </div>
              ) : (
                <div>
                  <Input
                    ref={fileInputRef}
                    type="file"
                    accept="image/*"
                    onChange={handleLogoChange}
                    className="cursor-pointer"
                  />
                  <p className="mt-1 text-xs text-muted-foreground">
                    {t(
                      'links.qr.logoHint',
                      'Recommended: Square PNG with transparent background',
                    )}
                  </p>
                </div>
              )}
            </div>

            {/* 尺寸 + 边距 */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="qr-size">{t('links.qr.size', 'Size')}</Label>
                <div className="flex items-center gap-2">
                  <Input
                    id="qr-size"
                    type="number"
                    min={100}
                    max={1000}
                    step={50}
                    value={config.size}
                    onChange={(e) =>
                      updateConfig('size', Number(e.target.value) || 300)
                    }
                    className="w-full"
                  />
                  <span className="text-sm text-muted-foreground">px</span>
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="qr-margin">
                  {t('links.qr.margin', 'Margin')}
                </Label>
                <Input
                  id="qr-margin"
                  type="number"
                  min={0}
                  max={10}
                  value={config.margin}
                  onChange={(e) =>
                    updateConfig('margin', Number(e.target.value) || 0)
                  }
                />
              </div>
            </div>

            {/* 颜色 */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="qr-dark">
                  {t('links.qr.foreground', 'Foreground')}
                </Label>
                <div className="flex gap-2">
                  <Input
                    id="qr-dark"
                    type="color"
                    value={config.darkColor}
                    onChange={(e) => updateConfig('darkColor', e.target.value)}
                    className="h-9 w-12 cursor-pointer p-1"
                  />
                  <Input
                    type="text"
                    value={config.darkColor}
                    onChange={(e) => updateConfig('darkColor', e.target.value)}
                    className="flex-1 font-mono text-sm"
                    maxLength={7}
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="qr-light">
                  {t('links.qr.background', 'Background')}
                </Label>
                <div className="flex gap-2">
                  <Input
                    id="qr-light"
                    type="color"
                    value={config.lightColor}
                    onChange={(e) => updateConfig('lightColor', e.target.value)}
                    className="h-9 w-12 cursor-pointer p-1"
                  />
                  <Input
                    type="text"
                    value={config.lightColor}
                    onChange={(e) => updateConfig('lightColor', e.target.value)}
                    className="flex-1 font-mono text-sm"
                    maxLength={7}
                  />
                </div>
              </div>
            </div>

            {/* 纠错级别 */}
            <div className="space-y-2">
              <Label>{t('links.qr.errorLevel', 'Error Correction')}</Label>
              <Select
                value={config.errorCorrectionLevel}
                onValueChange={(value) =>
                  updateConfig(
                    'errorCorrectionLevel',
                    value as ErrorCorrectionLevel,
                  )
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="L">L - Low (7%)</SelectItem>
                  <SelectItem value="M">M - Medium (15%)</SelectItem>
                  <SelectItem value="Q">Q - Quartile (25%)</SelectItem>
                  <SelectItem value="H">H - High (30%)</SelectItem>
                </SelectContent>
              </Select>
              {config.logoFile && config.errorCorrectionLevel !== 'H' && (
                <p className="text-xs text-amber-600">
                  {t(
                    'links.qr.errorLevelWarning',
                    'Recommended: Use "H" level when adding a logo',
                  )}
                </p>
              )}
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            {t('common.close', 'Close')}
          </Button>
          <Button
            onClick={handleDownload}
            disabled={!finalDataUrl || isGenerating || isDownloading}
          >
            {isDownloading ? (
              <>
                <Loader className="mr-2 h-4 w-4 animate-spin" />
                {t('links.qr.downloading', 'Downloading...')}
              </>
            ) : (
              <>
                <Download className="mr-2 h-4 w-4" />
                {t('links.qr.downloadPng', 'Download PNG')}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
