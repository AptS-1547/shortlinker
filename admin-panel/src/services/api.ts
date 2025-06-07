// ==================== 导出类型 ====================
export * from './types'

// ==================== 导出服务实例 ====================
export { linkService as LinkAPI } from './linkService'
export { authService as AuthAPI } from './authService'
export { healthService as HealthAPI } from './healthService'
export { batchService as BatchAPI } from './batchService'
export { qrcodeService as QRCodeAPI } from './qrcodeService'

// ==================== 导出HTTP相关 ====================
export { ApiError, config } from './http'

// ==================== 导出原有函数以保持兼容性 ====================
import { linkService } from './linkService'
import { authService } from './authService'
import { healthService } from './healthService'
import { batchService } from './batchService'
import { qrcodeService } from './qrcodeService'

export const fetchLinks = linkService.fetchAll.bind(linkService)
export const fetchLink = linkService.fetchOne.bind(linkService)
export const createLink = linkService.create.bind(linkService)
export const createLinkWithCheck = linkService.createWithCheck.bind(linkService)
export const updateLink = linkService.update.bind(linkService)
export const deleteLink = linkService.delete.bind(linkService)

export const login = authService.login.bind(authService)
export const verifyToken = authService.verifyToken.bind(authService)

export const fetchHealth = healthService.check.bind(healthService)

export const batchDeleteLinks = batchService.deleteLinks.bind(batchService)
export const batchCreateLinks = batchService.createLinks.bind(batchService)
export const batchUpdateLinks = batchService.updateLinks.bind(batchService)

export const generateQRCode = qrcodeService.generate.bind(qrcodeService)
export const generateShortLinkQRCode = qrcodeService.generateForShortLink.bind(qrcodeService)
export const downloadQRCode = qrcodeService.download.bind(qrcodeService)
