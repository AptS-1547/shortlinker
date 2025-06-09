<template>
  <div class="relative" ref="themeDropdown">
    <button
      @click="toggleThemeMenu"
      class="flex items-center space-x-1 sm:space-x-2 bg-white/10 dark:bg-black/10 px-2 py-1 sm:px-3 rounded-full hover:bg-white/20 dark:hover:bg-black/20 transition-all duration-200 transform hover:scale-105"
    >
      <component :is="getCurrentIcon()" class="w-3 h-3 sm:w-4 sm:h-4" />
      <span class="text-xs sm:text-sm font-medium hidden sm:inline">{{ getCurrentLabel() }}</span>
      <ChevronDownIcon class="w-2 h-2 sm:w-3 sm:h-3 text-white/60 dark:text-gray-400 transition-transform duration-200" :class="{ 'rotate-180': showThemeMenu }" />
    </button>

    <!-- Theme Dropdown -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="showThemeMenu"
        class="absolute right-0 mt-2 w-36 sm:w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50"
      >
        <button
          v-for="themeOption in themeOptions"
          :key="themeOption.value"
          @click="selectTheme(themeOption.value)"
          class="w-full text-left px-3 sm:px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center space-x-2"
          :class="{ 'bg-gray-100 dark:bg-gray-700 font-medium': theme === themeOption.value }"
        >
          <component :is="themeOption.icon" class="w-4 h-4" />
          <span>{{ $t(`theme.${themeOption.value}`) }}</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTheme, type Theme } from '@/composables/useTheme'
import { ChevronDownIcon } from '@heroicons/vue/24/outline'

const { t } = useI18n()
const { theme, setTheme } = useTheme()

const showThemeMenu = ref(false)
const themeDropdown = ref<HTMLElement>()

// 主题选项
const themeOptions = [
  {
    value: 'light' as Theme,
    icon: () => h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24' }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z' })
    ])
  },
  {
    value: 'dark' as Theme,
    icon: () => h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24' }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z' })
    ])
  },
  {
    value: 'system' as Theme,
    icon: () => h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24' }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z' })
    ])
  }
]

const getCurrentIcon = () => {
  return themeOptions.find(option => option.value === theme.value)?.icon || themeOptions[2].icon
}

const getCurrentLabel = () => {
  return t(`theme.${theme.value}`)
}

const toggleThemeMenu = () => {
  showThemeMenu.value = !showThemeMenu.value
}

const selectTheme = (newTheme: Theme) => {
  setTheme(newTheme)
  showThemeMenu.value = false
}

// 点击外部关闭菜单
const handleClickOutside = (event: Event) => {
  if (themeDropdown.value && !themeDropdown.value.contains(event.target as Node)) {
    showThemeMenu.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
