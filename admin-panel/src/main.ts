import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { useTheme } from '@/composables/useTheme'

import App from './App.vue'
import router from './router'
import i18n from './i18n'

import './assets/main.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

// 在应用挂载前初始化主题
const themeComposable = useTheme()

app.mount('#app')
