import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider } from 'react-router-dom'
import { ErrorBoundary } from './components/ErrorBoundary'
import { Toaster } from './components/ui/sonner'
import { router } from './router'
import './i18n'
import './index.css'

// Initialize theme immediately to prevent flash
const initTheme = () => {
  const savedTheme = localStorage.getItem('theme')
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const isDark =
    savedTheme === 'dark' || (savedTheme !== 'light' && prefersDark)

  if (isDark) {
    document.documentElement.classList.add('dark')
    document.documentElement.style.backgroundColor = '#0f172a'
    document.body.style.backgroundColor = '#0f172a'
  }
}
initTheme()

const rootElement = document.getElementById('root')
if (!rootElement) throw new Error('Root element not found')

createRoot(rootElement).render(
  <StrictMode>
    <ErrorBoundary>
      <RouterProvider router={router} />
      <Toaster />
    </ErrorBoundary>
  </StrictMode>,
)
