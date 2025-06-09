import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'
import tailwindcss from '@tailwindcss/vite'
import { generateVersionInfo } from './scripts/generate-version.js'
import fs from 'fs'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueDevTools(),
    tailwindcss(),
    // 在开发模式下自动生成版本信息
    {
      name: 'generate-version',
      buildStart() {
        const versionFilePath = path.resolve(__dirname, 'src/version.json')
        if (!fs.existsSync(versionFilePath)) {
          console.log('🔄 Generating version info...')
          const versionInfo = generateVersionInfo()
          fs.writeFileSync(versionFilePath, JSON.stringify(versionInfo, null, 2), 'utf8')
        }
      }
    }
  ],
  base: './', // 使用相对路径
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    port: 3000,
    open: true
  },
  build: {
    outDir: 'dist',
    sourcemap: false
  }
})
