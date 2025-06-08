import { describe, it, expect, vi, beforeEach } from 'vitest'
import { LinkService } from '@/services/linkService'
import type { SerializableShortLink, LinkPayload } from '@/services/types'

describe('LinkService.createWithCheck', () => {
  let service: LinkService

  beforeEach(() => {
    service = new LinkService()
  })

  it('returns existing link when duplicate', async () => {
    const payload: LinkPayload = { code: 'abc', target: 'https://a.com' }
    const existing: SerializableShortLink = {
      short_code: 'abc',
      target_url: 'https://a.com',
      created_at: '',
      expires_at: null
    }
    service.fetchOne = vi.fn().mockResolvedValue(existing)
    service.create = vi.fn()

    const result = await service.createWithCheck(payload)
    expect(result).toEqual({ success: false, exists: true, existingLink: existing })
    expect(service.create).not.toHaveBeenCalled()
  })

  it('creates new link when none exists', async () => {
    const payload: LinkPayload = { code: 'abc', target: 'https://a.com' }
    service.fetchOne = vi.fn().mockResolvedValue(null)
    service.create = vi.fn().mockResolvedValue(undefined)

    const result = await service.createWithCheck(payload)
    expect(service.create).toHaveBeenCalledWith(payload)
    expect(result).toEqual({ success: true })
  })

  it('skips check when force is true', async () => {
    const payload: LinkPayload = { code: 'abc', target: 'https://a.com', force: true }
    const fetchSpy = vi.spyOn(service, 'fetchOne')
    service.create = vi.fn().mockResolvedValue(undefined)

    const result = await service.createWithCheck(payload)
    expect(fetchSpy).not.toHaveBeenCalled()
    expect(service.create).toHaveBeenCalledWith(payload)
    expect(result).toEqual({ success: true })
  })
})
