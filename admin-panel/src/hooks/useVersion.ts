import { useMemo } from 'react'
import { appConfig } from '@/config/app'

export function useVersion() {
  const version = appConfig.shortlinkerVersion

  // version 是从 appConfig 单例获取的常量，不需要作为依赖项
  const displayVersion = useMemo(() => {
    const cleanVersion = version.replace('-dirty', '')
    return cleanVersion.startsWith('v') ? cleanVersion : `v${cleanVersion}`
  }, [])

  const versionType = useMemo(() => {
    const ver = version.toLowerCase()

    if (ver.includes('alpha')) return 'alpha'
    if (ver.includes('beta')) return 'beta'
    if (ver.includes('rc')) return 'rc'

    return 'release'
  }, [])

  const isDevelopment = useMemo(() => versionType === 'alpha', [versionType])
  const isPreRelease = useMemo(
    () => ['beta', 'rc'].includes(versionType),
    [versionType],
  )

  const versionBadgeColor = useMemo(() => {
    switch (versionType) {
      case 'alpha':
        return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
      case 'beta':
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300'
      case 'rc':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
      case 'release':
        return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
    }
  }, [versionType])

  const versionLabel = useMemo(() => {
    switch (versionType) {
      case 'alpha':
        return 'Alpha'
      case 'beta':
        return 'Beta'
      case 'rc':
        return 'RC'
      case 'release':
        return 'Release'
      default:
        return 'Unknown'
    }
  }, [versionType])

  return {
    displayVersion,
    isDevelopment,
    isPreRelease,
    versionBadgeColor,
    versionLabel,
  }
}
