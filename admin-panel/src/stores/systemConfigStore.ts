import { create } from 'zustand'
import { SystemConfigAPI } from '@/services/api'
import type { ConfigItemResponse } from '@/services/types'
import { extractErrorMessage } from '@/utils/errorHandler'

interface SystemConfigState {
  configs: ConfigItemResponse[]
  fetching: boolean
  updating: boolean
  reloading: boolean
  error: string | null

  // Actions
  fetchConfigs: (signal?: AbortSignal) => Promise<void>
  updateConfig: (
    key: string,
    value: string,
  ) => Promise<{ requires_restart: boolean; message: string | null }>
  reloadConfigs: () => Promise<void>
}

export const useSystemConfigStore = create<SystemConfigState>((set, get) => ({
  configs: [],
  fetching: false,
  updating: false,
  reloading: false,
  error: null,

  fetchConfigs: async (signal?: AbortSignal) => {
    set({ fetching: true, error: null })
    try {
      // 默认跳过缓存，确保每次切换到配置页面都获取最新数据
      const configs = await SystemConfigAPI.fetchAll(signal, true)
      set({ configs })
    } catch (err) {
      // 忽略取消错误
      if (err instanceof Error && err.name === 'AbortError') {
        return
      }
      set({ error: extractErrorMessage(err, 'Failed to fetch configs') })
      throw err
    } finally {
      set({ fetching: false })
    }
  },

  updateConfig: async (key: string, value: string) => {
    set({ updating: true, error: null })

    // 保存旧状态用于回滚
    const oldConfigs = get().configs

    // 乐观更新
    const newConfigs = oldConfigs.map((config) =>
      config.key === key
        ? { ...config, value, updated_at: new Date().toISOString() }
        : config,
    )
    set({ configs: newConfigs })

    try {
      const result = await SystemConfigAPI.update(key, value)

      return {
        requires_restart: result.requires_restart,
        message: result.message,
      }
    } catch (err) {
      // 回滚到旧状态
      set({
        configs: oldConfigs,
        error: extractErrorMessage(err, 'Failed to update config'),
      })
      throw err
    } finally {
      set({ updating: false })
    }
  },

  reloadConfigs: async () => {
    set({ reloading: true, error: null })
    try {
      await SystemConfigAPI.reload()
      // 重新获取配置
      await get().fetchConfigs()
    } catch (err) {
      set({ error: extractErrorMessage(err, 'Failed to reload configs') })
      throw err
    } finally {
      set({ reloading: false })
    }
  },
}))

// Selector for loading state
export const useSystemConfigLoading = () =>
  useSystemConfigStore(
    (state) => state.fetching || state.updating || state.reloading,
  )
