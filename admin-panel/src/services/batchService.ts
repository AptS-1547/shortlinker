import { linkService } from './linkService'
import type { BatchOperationResult, LinkPayload } from './types'

type BatchOperation<T> = (item: T) => Promise<void>
type BatchItemIdentifier<T> = (item: T) => string

export class BatchService {
  /**
   * 执行批量操作
   */
  private async execute<T>(
    items: T[],
    operation: BatchOperation<T>,
    getIdentifier: BatchItemIdentifier<T>,
  ): Promise<BatchOperationResult> {
    const result: BatchOperationResult = {
      success: [],
      failed: [],
    }

    const results = await Promise.allSettled(
      items.map(async (item) => {
        const identifier = getIdentifier(item)
        try {
          await operation(item)
          return { success: true, identifier }
        } catch (error) {
          return {
            success: false,
            identifier,
            error: error instanceof Error ? error.message : 'Unknown error',
          }
        }
      }),
    )

    results.forEach((promiseResult) => {
      if (promiseResult.status === 'fulfilled') {
        const { success, identifier, error } = promiseResult.value
        if (success) {
          result.success.push(identifier)
        } else {
          result.failed.push({ code: identifier, error: error! })
        }
      } else {
        result.failed.push({
          code: 'unknown',
          error:
            promiseResult.reason instanceof Error
              ? promiseResult.reason.message
              : 'Promise rejected',
        })
      }
    })

    return result
  }

  /**
   * 批量删除链接
   */
  async deleteLinks(codes: string[]): Promise<BatchOperationResult> {
    return this.execute(
      codes,
      (code) => linkService.delete(code),
      (code) => code,
    )
  }

  /**
   * 批量创建链接
   */
  async createLinks(links: LinkPayload[]): Promise<BatchOperationResult> {
    return this.execute(
      links,
      (link) => linkService.create(link),
      (link) => link.code || 'auto-generated',
    )
  }

  /**
   * 批量更新链接
   */
  async updateLinks(
    updates: { code: string; payload: LinkPayload }[],
  ): Promise<BatchOperationResult> {
    return this.execute(
      updates,
      (update) => linkService.update(update.code, update.payload),
      (update) => update.code,
    )
  }
}

export const batchService = new BatchService()
