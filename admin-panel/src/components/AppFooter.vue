<template>
  <footer class="bg-gradient-to-r from-slate-50 to-gray-50 border-t border-gray-200/50 mt-8">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
      <div class="flex flex-col md:flex-row items-center justify-between gap-3">
        <!-- 项目信息 -->
        <div class="flex items-center gap-3">
          <div class="w-6 h-6 bg-gradient-to-br from-indigo-500 to-indigo-600 rounded-lg flex items-center justify-center shadow-sm">
            <LinkIcon className="w-4 h-4 text-white" />
          </div>
          <div>
            <h3 class="text-base font-bold bg-gradient-to-r from-gray-800 via-indigo-800 to-purple-800 bg-clip-text text-transparent">
              ShortLinker
            </h3>
            <p class="text-xs text-gray-500 font-medium">
              Admin Panel v1.0.0
            </p>
          </div>
        </div>

        <!-- 中间信息 -->
        <div class="flex items-center gap-4 text-sm text-gray-600">
          <span class="font-medium">Open Source</span>
          <div class="w-1 h-1 bg-gray-400 rounded-full"></div>
          <span class="font-medium">MIT License</span>
          <div class="w-1 h-1 bg-gray-400 rounded-full hidden sm:block"></div>
          <span class="text-xs text-gray-500 hidden sm:inline">
            Vue 3 + TypeScript + Tailwind
          </span>
        </div>

        <!-- 右侧链接 -->
        <div class="flex items-center gap-3">
          <a
            href="https://github.com/AptS-1547/shortlinker"
            target="_blank"
            rel="noopener noreferrer"
            class="text-sm text-gray-600 hover:text-indigo-600 font-medium transition-colors"
            title="View on GitHub"
          >
            GitHub
          </a>
          <div class="w-1 h-1 bg-gray-400 rounded-full"></div>
          <a
            href="https://shortlinker.docs.ecaps.top"
            target="_blank"
            rel="noopener noreferrer"
            class="text-sm text-gray-600 hover:text-indigo-600 font-medium transition-colors"
            title="Documentation"
          >
            Docs
          </a>
        </div>
      </div>

      <!-- 版权信息 -->
      <div class="mt-3 pt-3 border-t border-gray-200/50">
        <div class="flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-gray-500">
          <p>
            © {{ currentYear }} ShortLinker. Built with ❤️ for the open source community.
          </p>
          <p class="flex items-center gap-1">
            <span>Powered by</span>
            <span class="font-mono bg-gray-100 px-1.5 py-0.5 rounded text-indigo-700 font-semibold">
              Rust
            </span>
            <span>+</span>
            <span class="font-mono bg-gray-100 px-1.5 py-0.5 rounded text-emerald-700 font-semibold">
              Vue 3
            </span>
          </p>
        </div>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { LinkIcon } from '@/components/icons'

const currentYear = computed(() => new Date().getFullYear())
</script>

<style scoped>
/* 添加一些微妙的动画效果 */
footer {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
}

/* 响应式优化 */
@media (max-width: 768px) {
  .flex-col {
    text-align: center;
  }
}
</style>
