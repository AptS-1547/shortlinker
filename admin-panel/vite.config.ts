import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { VitePWA } from 'vite-plugin-pwa'

// https://vite.dev/config/
export default defineConfig({
  base: './',
  plugins: [
    react(),
    tailwindcss(),
    VitePWA({
      registerType: 'prompt',
      includeAssets: ['favicon.ico', 'apple-touch-icon.png'],
      manifest: {
        name: 'Shortlinker Admin',
        short_name: 'Shortlinker',
        description: '短链接管理后台',
        theme_color: '#3b82f6',
        background_color: '#ffffff',
        display: 'standalone',
        // 使用占位符，后端会替换成实际路径
        start_url: '%BASE_PATH%/',
        scope: '%BASE_PATH%/',
        icons: [
          {
            src: '%BASE_PATH%/pwa-192x192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: '%BASE_PATH%/pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'any maskable',
          },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
        navigateFallback: 'index.html',
        navigateFallbackDenylist: [/^\/api\//, /^\/admin\//, /^\/health\//],
      },
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    // 确保只有一个 React 实例，防止 hooks 失效
    dedupe: ['react', 'react-dom'],
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          // React 核心
          if (
            id.includes('node_modules/react') ||
            id.includes('node_modules/react-dom') ||
            id.includes('node_modules/react-router')
          ) {
            return 'react-vendor'
          }
          // Radix UI 组件库
          if (id.includes('node_modules/@radix-ui')) {
            return 'ui-vendor'
          }
          // 工具库
          if (
            id.includes('node_modules/axios') ||
            id.includes('node_modules/date-fns') ||
            id.includes('node_modules/zustand') ||
            id.includes('node_modules/i18next')
          ) {
            return 'utils'
          }
        },
      },
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/admin': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      '/health': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      '/*': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
})
