import { useCallback, useEffect, useState } from 'react'
import { Outlet } from 'react-router-dom'
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from '@/components/ui/sidebar'
import { usePwaUpdate } from '@/hooks/usePwaUpdate'
import { useHealthStore } from '@/stores/healthStore'
import HealthModal from '../HealthModal'
import AppSidebar from './AppSidebar'

// 轮询间隔 60 秒
const POLL_INTERVAL = 60 * 1000

export default function AdminLayout() {
  const checkHealth = useHealthStore((state) => state.checkHealth)
  const healthData = useHealthStore((state) => state.status)
  const loading = useHealthStore((state) => state.loading)

  const [showHealthModal, setShowHealthModal] = useState(false)

  // PWA 更新检测
  usePwaUpdate()

  // 强制刷新
  const handleRefresh = useCallback(() => {
    checkHealth(true)
  }, [checkHealth])

  // 定时轮询 + 页面可见性刷新
  useEffect(() => {
    checkHealth()

    // 每 60 秒轮询一次
    const interval = setInterval(() => {
      checkHealth()
    }, POLL_INTERVAL)

    // 页面可见性变化时刷新
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        checkHealth()
      }
    }
    document.addEventListener('visibilitychange', handleVisibilityChange)

    return () => {
      clearInterval(interval)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
  }, [checkHealth])

  return (
    <SidebarProvider>
      <AppSidebar onOpenHealthModal={() => setShowHealthModal(true)} />
      <SidebarInset>
        {/* Header with sidebar trigger */}
        <header className="flex h-14 items-center gap-4 border-b border-border/50 px-6">
          <SidebarTrigger className="-ml-2" />
          <div className="flex-1" />
        </header>

        {/* Main Content */}
        <main className="flex-1 p-6 lg:p-8">
          <Outlet />
        </main>
      </SidebarInset>

      {/* Health Modal */}
      <HealthModal
        open={showHealthModal}
        onClose={() => setShowHealthModal(false)}
        healthData={healthData}
        onRefresh={handleRefresh}
        loading={loading}
      />
    </SidebarProvider>
  )
}
