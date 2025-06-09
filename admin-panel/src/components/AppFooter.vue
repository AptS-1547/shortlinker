<template>
  <footer class="bg-gradient-to-r from-slate-50 to-gray-50 dark:from-gray-900 dark:to-gray-800 border-t border-gray-200/50 dark:border-gray-700/50 mt-8">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
      <div class="flex flex-col md:flex-row items-center justify-between gap-3">
        <!-- 项目信息 -->
        <div class="flex items-center gap-3">
          <div class="w-6 h-6 bg-gradient-to-br from-indigo-500 to-indigo-600 dark:from-indigo-400 dark:to-indigo-500 rounded-lg flex items-center justify-center shadow-sm">
            <LinkIcon className="w-4 h-4 text-white" />
          </div>
          <div>
            <h3 class="text-base font-bold bg-gradient-to-r from-gray-800 via-indigo-800 to-purple-800 dark:from-gray-200 dark:via-indigo-400 dark:to-purple-400 bg-clip-text text-transparent">
              ShortLinker
            </h3>
            <div class="flex items-center gap-2">
              <p class="text-xs text-gray-500 dark:text-gray-400 font-medium">
                Admin Panel {{ displayVersion }}
              </p>
              <span
                :class="[
                  'inline-flex items-center px-1.5 py-0.5 rounded text-xs font-bold',
                  versionBadgeColor
                ]"
                :title="`${versionLabel} - Built: ${buildDate}`"
              >
                {{ versionLabel }}
              </span>
            </div>
          </div>
        </div>

        <!-- 中间信息 -->
        <div class="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
          <span class="font-medium">Open Source</span>
          <div class="w-1 h-1 bg-gray-400 dark:bg-gray-500 rounded-full"></div>
          <a
            href="https://opensource.org/licenses/MIT"
            target="_blank"
            rel="noopener noreferrer"
            class="font-medium hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors"
            title="MIT License"
          >
            MIT License
          </a>
          <div class="w-1 h-1 bg-gray-400 dark:bg-gray-500 rounded-full hidden sm:block"></div>
          <span class="text-xs text-gray-500 dark:text-gray-400 hidden sm:inline">
            <a
              href="https://vuejs.org"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-emerald-600 dark:hover:text-emerald-400 transition-colors"
              title="Vue.js"
            >
              Vue 3
            </a>
            +
            <a
              href="https://www.typescriptlang.org"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              title="TypeScript"
            >
              TypeScript
            </a>
            +
            <a
              href="https://tailwindcss.com"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-cyan-600 dark:hover:text-cyan-400 transition-colors"
              title="Tailwind CSS"
            >
              Tailwind
            </a>
          </span>
        </div>

        <!-- 右侧链接 -->
        <div class="flex items-center gap-3">
          <a
            href="https://github.com/AptS-1547/shortlinker"
            target="_blank"
            rel="noopener noreferrer"
            class="text-sm text-gray-600 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 font-medium transition-colors"
            title="View on GitHub"
          >
            GitHub
          </a>
          <div class="w-1 h-1 bg-gray-400 dark:bg-gray-500 rounded-full"></div>
          <a
            href="https://shortlinker.docs.ecaps.top"
            target="_blank"
            rel="noopener noreferrer"
            class="text-sm text-gray-600 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 font-medium transition-colors"
            title="Documentation"
          >
            Docs
          </a>
        </div>
      </div>

      <!-- 版权信息 -->
      <div class="mt-3 pt-3 border-t border-gray-200/50 dark:border-gray-700/50">
        <div class="flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-gray-500 dark:text-gray-400">
          <p>
            © {{ currentYear }} ShortLinker. Built with ❤️ for the open source community.
          </p>
          <p class="flex items-center gap-1">
            <span>Powered by</span>
            <a
              href="https://www.rust-lang.org"
              target="_blank"
              rel="noopener noreferrer"
              class="font-mono bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-orange-700 dark:text-orange-400 font-semibold hover:bg-orange-50 dark:hover:bg-orange-900/20 transition-colors"
              title="Rust Programming Language"
            >
              Rust
            </a>
            <span>+</span>
            <a
              href="https://vuejs.org"
              target="_blank"
              rel="noopener noreferrer"
              class="font-mono bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-emerald-700 dark:text-emerald-400 font-semibold hover:bg-emerald-50 dark:hover:bg-emerald-900/20 transition-colors"
              title="Vue.js Framework"
            >
              Vue 3
            </a>
          </p>
        </div>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { LinkIcon } from '@/components/icons'
import { useVersion } from '@/composables/useVersion'

const currentYear = computed(() => new Date().getFullYear())
const { displayVersion, versionBadgeColor, versionLabel, buildDate } = useVersion()
</script>

<style scoped>
/* 添加一些微妙的动画效果 */
footer {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
}

.dark footer {
  background: linear-gradient(135deg, #111827 0%, #1f2937 100%);
}

/* 响应式优化 */
@media (max-width: 768px) {
  .flex-col {
    text-align: center;
  }
}
</style>
