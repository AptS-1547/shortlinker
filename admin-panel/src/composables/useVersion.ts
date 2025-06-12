import { ref, computed } from 'vue'
import versionInfo from '@/version.json'

export interface VersionInfo {
  version: string
  buildTime: string
  git: {
    branch: string
    commitHash: string
    commitShort: string
    commitDate: string
  }
}

const version = ref<VersionInfo>(versionInfo)

export function useVersion() {
  const displayVersion = computed(() => {
    // 清理版本号显示，移除 dirty 后缀用于显示
    return version.value.version.replace('-dirty', '')
  })

  const isDevelopment = computed(() => {
    // 检查是否为开发分支或包含提交计数的版本（如 v1.0.0-2-gxxxxxx）
    const hasCommitCount = /-\d+-g[a-f0-9]+(-dirty)?$/.test(version.value.version)
    const isDevBranch = version.value.git.branch !== 'main' && version.value.git.branch !== 'master'
    return isDevBranch || hasCommitCount
  })

  const isPreRelease = computed(() => {
    // 检查是否包含预发布标识符（alpha, beta, rc等），但不是开发版本
    const hasPreReleaseTag = /-(?:alpha|beta|rc|pre|dev)\b/.test(version.value.version)
    const hasCommitCount = /-\d+-g[a-f0-9]+(-dirty)?$/.test(version.value.version)

    // 如果包含提交计数，则优先判断为开发版本
    if (hasCommitCount) return false

    // 检查是否包含 - 但不是 dirty 标记
    const hasDash = version.value.version.includes('-') && !version.value.version.includes('-dirty')

    return hasPreReleaseTag || hasDash
  })

  const buildDate = computed(() => {
    try {
      return new Date(version.value.buildTime).toLocaleString()
    } catch {
      return version.value.buildTime
    }
  })

  const commitDate = computed(() => {
    try {
      return new Date(version.value.git.commitDate).toLocaleString()
    } catch {
      return version.value.git.commitDate
    }
  })

  const versionBadgeColor = computed(() => {
    if (isDevelopment.value) return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300'
    if (isPreRelease.value) return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
    return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
  })

  const versionLabel = computed(() => {
    if (isDevelopment.value) return 'Development'
    if (isPreRelease.value) return 'Pre-release'
    return 'Release'
  })

  return {
    version: version.value,
    displayVersion,
    isDevelopment,
    isPreRelease,
    buildDate,
    commitDate,
    versionBadgeColor,
    versionLabel
  }
}
