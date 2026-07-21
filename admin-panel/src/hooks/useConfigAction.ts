import { useCallback, useState } from 'react'
import { systemConfigService } from '@/services/systemConfigService'
import type { ActionType } from '@/services/types'

export function useConfigAction() {
  const [executing, setExecuting] = useState(false)

  const executeAndSave = useCallback(
    async (key: string, action: ActionType) => {
      setExecuting(true)
      try {
        return await systemConfigService.executeAndSave(key, action)
      } finally {
        setExecuting(false)
      }
    },
    [],
  )

  return { executing, executeAndSave }
}
