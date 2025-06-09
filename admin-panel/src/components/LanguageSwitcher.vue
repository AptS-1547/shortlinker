<template>
  <div class="relative" ref="languageDropdown">
    <button
      @click="toggleLanguageMenu"
      class="flex items-center space-x-1 sm:space-x-2 bg-white/10 dark:bg-black/10 px-2 py-1 sm:px-3 rounded-full hover:bg-white/20 dark:hover:bg-black/20 transition-all duration-200 transform hover:scale-105"
    >
      <svg class="w-3 h-3 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
      </svg>
      <span class="text-xs sm:text-sm font-medium">{{ currentLanguage }}</span>
      <ChevronDownIcon class="w-2 h-2 sm:w-3 sm:h-3 text-white/60 dark:text-gray-400 transition-transform duration-200" :class="{ 'rotate-180': showLanguageMenu }" />
    </button>

    <!-- Language Dropdown -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="showLanguageMenu"
        class="absolute right-0 mt-2 w-36 sm:w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50"
      >
        <button
          v-for="lang in supportedLanguages"
          :key="lang"
          @click="changeLanguage(lang)"
          class="w-full text-left px-3 sm:px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center space-x-2"
          :class="{ 'bg-gray-100 dark:bg-gray-700 font-medium': locale === lang }"
        >
          <span>{{ getLanguageFlag(lang) }}</span>
          <span>{{ $t(`langswitcher.language.${getLanguageKey(lang)}`) }}</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDownIcon } from '@heroicons/vue/24/outline'
import { supportedLanguages, getLanguageDisplayName, getLanguageFlag, getLanguageKey } from '@/i18n'

const { locale, t } = useI18n()
const showLanguageMenu = ref(false)
const languageDropdown = ref<HTMLElement>()

const currentLanguage = computed(() => {
  return getLanguageDisplayName(locale.value)
})

const toggleLanguageMenu = () => {
  showLanguageMenu.value = !showLanguageMenu.value
}

const changeLanguage = (newLocale: string) => {
  locale.value = newLocale
  localStorage.setItem('preferred-language', newLocale)
  showLanguageMenu.value = false
}

// 点击外部关闭菜单
const handleClickOutside = (event: Event) => {
  if (languageDropdown.value && !languageDropdown.value.contains(event.target as Node)) {
    showLanguageMenu.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
