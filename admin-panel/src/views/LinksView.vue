<template>
  <div class="space-y-6">
    <!-- 页面标题 - 现代化设计 -->
    <div class="relative mb-6">
      <div class="absolute inset-0 bg-gradient-to-r from-indigo-500/5 via-blue-500/5 to-purple-500/5 dark:from-indigo-400/10 dark:via-blue-400/10 dark:to-purple-400/10 rounded-xl"></div>
      <div class="relative p-4">
        <h1 class="text-2xl font-bold bg-gradient-to-r from-gray-900 via-indigo-900 to-blue-900 dark:from-gray-100 dark:via-indigo-300 dark:to-blue-300 bg-clip-text text-transparent">
          {{ $t('links.title') }}
        </h1>
        <p class="text-gray-600 dark:text-gray-400 mt-1">{{ $t('links.description') }}</p>
      </div>
    </div>

    <!-- 筛选器 - 现代化设计 -->
    <div class="relative bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-xl shadow-lg border border-gray-200/50 dark:border-gray-700/50 transition-all duration-300 hover:shadow-xl hover:-translate-y-0.5">
      <div class="absolute inset-0 bg-gradient-to-br from-blue-50/30 via-indigo-50/20 to-purple-50/30 dark:from-blue-900/10 dark:via-indigo-900/10 dark:to-purple-900/10 rounded-xl"></div>
      <div
        class="relative p-4 border-b border-gray-200/50 dark:border-gray-700/50 cursor-pointer hover:bg-gradient-to-r hover:from-gray-50/50 hover:to-indigo-50/30 dark:hover:from-gray-700/30 dark:hover:to-indigo-900/20 transition-all duration-300 rounded-t-xl"
        @click="toggleFilter"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="p-2 bg-gradient-to-br from-blue-100 to-blue-50 dark:from-blue-900/50 dark:to-blue-800/30 text-blue-600 dark:text-blue-400 rounded-lg shadow-sm">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.207A1 1 0 013 6.5V4z"></path>
              </svg>
            </div>
            <div>
              <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{{ $t('links.filterTitle') }}</h2>
              <p class="text-sm text-gray-600 dark:text-gray-400 mt-0.5">{{ $t('links.filterDescription') }}</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span
              v-if="hasActiveFilters"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs font-bold bg-gradient-to-r from-blue-100 to-indigo-100 dark:from-blue-900/30 dark:to-indigo-900/30 text-blue-800 dark:text-blue-300 border border-blue-200/50 dark:border-blue-700/50 shadow-sm"
            >
              {{ $t('links.activeFilters', { count: activeFilterCount }) }}
            </span>
            <ChevronDownIcon
              :class="[
                'w-5 h-5 text-gray-500 dark:text-gray-400 transition-transform duration-300',
                { 'rotate-180': showFilter },
              ]"
            />
          </div>
        </div>
      </div>

      <!-- 筛选器内容 -->
      <div
        :class="[
          'overflow-hidden transition-all duration-300 ease-in-out',
          showFilter ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0',
        ]"
      >
        <div class="relative p-4">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div>
              <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">{{ $t('common.search') }}</label>
              <input
                v-model="filterForm.search"
                type="text"
                class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm hover:shadow-md bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400"
                :placeholder="$t('links.search.placeholder')"
              />
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">{{ $t('links.status') }}</label>
              <select
                v-model="filterForm.status"
                class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100"
              >
                <option value="">{{ $t('links.filterOptions.allLinks') }}</option>
                <option value="active">{{ $t('links.filterOptions.activeOnly') }}</option>
                <option value="expired">{{ $t('links.filterOptions.expiredOnly') }}</option>
                <option value="permanent">{{ $t('links.filterOptions.permanentLinks') }}</option>
                <option value="temporary">{{ $t('links.filterOptions.temporaryLinks') }}</option>
              </select>
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">{{ $t('links.createdAfter') }}</label>
              <input
                v-model="filterForm.created_after"
                type="date"
                class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100"
              />
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">{{ $t('links.createdBefore') }}</label>
              <input
                v-model="filterForm.created_before"
                type="date"
                class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100"
              />
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">{{ $t('links.pageSize') }}</label>
              <select
                v-model="filterForm.page_size"
                class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100"
              >
                <option :value="10">10 per page</option>
                <option :value="20">20 per page</option>
                <option :value="50">50 per page</option>
                <option :value="100">100 per page</option>
              </select>
            </div>
          </div>

          <div class="flex items-center justify-between pt-4 border-t-2 border-gray-200/50 dark:border-gray-700/50 mt-4">
            <button
              @click="resetFilters"
              class="px-3 py-1.5 text-xs font-bold text-gray-700 dark:text-gray-300 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 hover:from-gray-200 hover:to-gray-100 dark:hover:from-gray-600 dark:hover:to-gray-500 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md transform hover:scale-105 uppercase tracking-wide"
            >
              {{ $t('common.reset') }}
            </button>
            <button
              @click="applyFilters"
              class="px-4 py-2 bg-gradient-to-r from-indigo-500 to-indigo-600 dark:from-indigo-400 dark:to-indigo-500 text-white font-bold rounded-lg hover:from-indigo-600 hover:to-indigo-700 dark:hover:from-indigo-500 dark:hover:to-indigo-600 transition-all duration-300 shadow-md hover:shadow-lg transform hover:scale-105 uppercase tracking-wide text-sm"
            >
              {{ $t('common.apply') }} {{ $t('common.filter') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 创建/编辑链接表单 - 现代化设计 -->
    <div class="relative bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-xl shadow-lg border border-gray-200/50 dark:border-gray-700/50 transition-all duration-300 hover:shadow-xl hover:-translate-y-0.5">
      <!-- 表单头部 -->
      <div class="absolute inset-0 bg-gradient-to-br from-green-50/30 via-indigo-50/20 to-purple-50/30 dark:from-green-900/10 dark:via-indigo-900/10 dark:to-purple-900/10 rounded-xl"></div>
      <div
        class="relative p-4 border-b border-gray-200/50 dark:border-gray-700/50 cursor-pointer hover:bg-gradient-to-r hover:from-gray-50/50 hover:to-indigo-50/30 dark:hover:from-gray-700/30 dark:hover:to-indigo-900/20 transition-all duration-300 rounded-t-xl"
        @click="toggleForm"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div
              :class="[
                'p-2 rounded-lg flex items-center justify-center transition-all duration-300 shadow-sm',
                editingLink ? 'bg-gradient-to-br from-amber-100 to-orange-50 dark:from-amber-900/50 dark:to-orange-900/30 text-amber-700 dark:text-amber-400' : 'bg-gradient-to-br from-indigo-100 to-indigo-50 dark:from-indigo-900/50 dark:to-indigo-800/30 text-indigo-700 dark:text-indigo-400',
              ]"
            >
              <EditIcon
                v-if="editingLink"
                className="w-5 h-5"
              />
              <PlusIcon v-else className="w-5 h-5" />
            </div>
            <div>
              <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">
                {{ editingLink ? $t('links.editTitle') : $t('links.createTitle') }}
              </h2>
              <p class="text-sm text-gray-600 dark:text-gray-400 mt-0.5">
                {{ editingLink ? $t('links.editDescription') : $t('links.createDescription') }}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span
              v-if="editingLink"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs font-bold bg-gradient-to-r from-indigo-100 to-purple-100 dark:from-indigo-900/30 dark:to-purple-900/30 text-indigo-800 dark:text-indigo-300 border border-indigo-200/50 dark:border-indigo-700/50 shadow-sm"
            >
              {{ $t('links.editing', { code: editingLink.short_code }) }}
            </span>
            <ChevronDownIcon
              :class="[
                'w-5 h-5 text-gray-500 dark:text-gray-400 transition-transform duration-300',
                { 'rotate-180': showForm },
              ]"
            />
          </div>
        </div>
      </div>

      <!-- 表单内容 -->
      <div
        :class="[
          'overflow-hidden transition-all duration-300 ease-in-out',
          showForm ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0',
        ]"
      >
        <div class="relative p-4">
          <form @submit.prevent="handleSave" class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">
                  {{ $t('links.shortCodeOptional') }}
                </label>
                <input
                  v-model="formData.code"
                  type="text"
                  class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm hover:shadow-md bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400"
                  :placeholder="$t('links.shortCodePlaceholder')"
                />
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 font-medium">
                  {{ $t('links.shortCodeHelp') }}
                </p>
              </div>

              <div>
                <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">
                  {{ $t('links.expiresAtOptional') }}
                </label>
                <input
                  v-model="formData.expires_at"
                  type="datetime-local"
                  class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100"
                />
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 font-medium">{{ $t('links.expiresAtHelp') }}</p>
              </div>
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-900 dark:text-gray-100 mb-1 uppercase tracking-wide">{{ $t('links.targetUrlRequired') }}</label>
              <input
                v-model="formData.target"
                type="url"
                required
                class="w-full px-3 py-2 border-2 border-gray-200/50 dark:border-gray-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-400 dark:focus:border-indigo-500 transition-all duration-300 shadow-sm bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm text-sm text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400"
                :placeholder="$t('links.targetUrlPlaceholder')"
              />
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 font-medium">
                {{ $t('links.targetUrlHelp') }}
              </p>
            </div>

            <div class="flex items-center justify-between pt-4 border-t-2 border-gray-200/50 dark:border-gray-700/50">
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  @click="collapseForm"
                  class="px-3 py-1.5 text-xs font-bold text-gray-700 dark:text-gray-300 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 hover:from-gray-200 hover:to-gray-100 dark:hover:from-gray-600 dark:hover:to-gray-500 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md transform hover:scale-105 uppercase tracking-wide"
                >
                  {{ $t('common.cancel') }}
                </button>
                <button
                  v-if="editingLink"
                  type="button"
                  @click="cancelEdit"
                  class="px-3 py-1.5 text-xs font-bold text-gray-700 dark:text-gray-300 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 hover:from-gray-200 hover:to-gray-100 dark:hover:from-gray-600 dark:hover:to-gray-500 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md transform hover:scale-105 uppercase tracking-wide"
                >
                  {{ $t('links.clearEdit') }}
                </button>
              </div>

              <button
                type="submit"
                :disabled="loading || !formData.target"
                class="px-6 py-2 bg-gradient-to-r from-indigo-500 via-indigo-600 to-purple-600 dark:from-indigo-400 dark:via-indigo-500 dark:to-purple-500 text-white font-bold rounded-lg hover:from-indigo-600 hover:via-indigo-700 hover:to-purple-700 dark:hover:from-indigo-500 dark:hover:via-indigo-600 dark:hover:to-purple-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:ring-offset-2 dark:focus:ring-offset-gray-800 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-300 shadow-md hover:shadow-lg transform hover:scale-105 active:scale-95 uppercase tracking-wide text-sm"
              >
                <span v-if="loading" class="flex items-center">
                  <SpinnerIcon className="animate-spin -ml-1 mr-2 h-4 w-4" />
                  {{ editingLink ? $t('links.updating') : $t('links.creating') }}
                </span>
                <span v-else>
                  {{ editingLink ? $t('links.updateLink') : $t('links.createLink') }}
                </span>
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- 链接列表 - 现代化设计 -->
    <div class="relative bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-xl shadow-lg border border-gray-200/50 dark:border-gray-700/50 transition-all duration-300 hover:shadow-xl hover:-translate-y-0.5">
      <div class="absolute inset-0 bg-gradient-to-br from-gray-50/30 via-blue-50/20 to-indigo-50/30 dark:from-gray-800/30 dark:via-blue-900/10 dark:to-indigo-900/10 rounded-xl"></div>
      <div class="relative p-4 border-b border-gray-200/50 dark:border-gray-700/50">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="p-2 bg-gradient-to-br from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 rounded-lg shadow-sm">
              <LinkIcon className="w-5 h-5 text-gray-700 dark:text-gray-300" />
            </div>
            <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{{ $t('links.shortLinks') }}</h2>
          </div>
          <div class="flex items-center gap-4">
            <span class="text-xs font-bold text-gray-600 dark:text-gray-400 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 px-2 py-1 rounded border border-gray-200/50 dark:border-gray-600/50">
              {{ filteredLinks.length }}
              <span v-if="filteredLinks.length !== totalCount">of {{ totalCount }}</span>
              total
              <span v-if="hasActiveFilters">(filtered)</span>
            </span>
            <span class="text-xs font-bold text-gray-600 dark:text-gray-400 bg-gradient-to-r from-indigo-100 to-indigo-50 dark:from-indigo-900/30 dark:to-indigo-800/20 px-2 py-1 rounded border border-indigo-200/50 dark:border-indigo-700/50">
              Page {{ currentPage }} of {{ totalPages }}
            </span>
          </div>
        </div>
      </div>

      <div class="relative p-4">
        <div v-if="loading && links.length === 0" class="text-center py-8">
          <div class="animate-spin mx-auto mb-4">
            <SpinnerIcon className="h-8 w-8 text-indigo-500 dark:text-indigo-400" />
          </div>
          <p class="text-gray-500 dark:text-gray-400">{{ $t('common.loading') }}...</p>
        </div>

        <div v-else-if="error" class="text-center py-8">
          <div class="text-red-500 dark:text-red-400 mb-4">{{ error }}</div>
          <button
            @click="() => fetchLinks()"
            class="px-4 py-2 bg-indigo-500 dark:bg-indigo-400 text-white rounded-lg hover:bg-indigo-600 dark:hover:bg-indigo-500 transition-colors"
          >
            {{ $t('common.retry') }}
          </button>
        </div>

        <div v-else-if="filteredLinks.length === 0" class="text-center py-8">
          <div
            class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center"
          >
            <LinkIcon className="w-8 h-8 text-gray-400 dark:text-gray-500" />
          </div>
          <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            {{ hasActiveFilters ? $t('links.noMatchingLinks') : $t('links.noLinksYet') }}
          </h3>
          <p class="text-gray-700 dark:text-gray-400 mb-4">
            {{ hasActiveFilters ? $t('links.noMatchingLinksDesc') : $t('links.noLinksYetDesc') }}
          </p>
          <div class="flex items-center justify-center gap-2">
            <button
              v-if="hasActiveFilters"
              @click="resetFilters"
              class="inline-flex items-center px-4 py-2 bg-gray-500 dark:bg-gray-600 text-white rounded-lg hover:bg-gray-600 dark:hover:bg-gray-500 transition-colors"
            >
              {{ $t('links.clearFilters') }}
            </button>
            <button
              @click="showFormAndFocus"
              class="inline-flex items-center px-4 py-2 bg-indigo-500 dark:bg-indigo-400 text-white rounded-lg hover:bg-indigo-600 dark:hover:bg-indigo-500 transition-colors"
            >
              <PlusIcon className="w-4 h-4 mr-2" />
              {{ hasActiveFilters ? $t('links.createNewLink') : $t('links.createFirstLink') }}
            </button>
          </div>
        </div>

        <div v-else class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b-2 border-gray-200/70 dark:border-gray-700/70">
                <th class="text-left py-3 px-4 font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wide text-xs">{{ $t('links.table.code') }}</th>
                <th class="text-left py-3 px-4 font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wide text-xs">{{ $t('links.table.targetUrl') }}</th>
                <th class="text-left py-3 px-4 font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wide text-xs">{{ $t('links.table.status') }}</th>
                <th class="text-left py-3 px-4 font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wide text-xs">{{ $t('links.table.created') }}</th>
                <th class="text-left py-3 px-4 font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wide text-xs">{{ $t('links.table.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="link in filteredLinks"
                :key="link.short_code"
                class="border-b border-gray-100/70 dark:border-gray-700/70 hover:bg-gradient-to-r hover:from-gray-50/80 hover:to-indigo-50/40 dark:hover:from-gray-700/50 dark:hover:to-indigo-900/20 transition-all duration-300 group"
              >
                <td class="py-3 px-4">
                  <button
                    @click="copyShortLink(link.short_code)"
                    :class="[
                      'font-mono text-xs px-3 py-1.5 rounded-lg transition-all duration-300 border shadow-sm hover:shadow-md',
                      copiedLink === link.short_code
                        ? 'bg-gradient-to-r from-emerald-100 to-emerald-50 dark:from-emerald-900/30 dark:to-emerald-800/20 text-emerald-800 dark:text-emerald-300 border-emerald-300 dark:border-emerald-700 scale-105 shadow-md'
                        : 'bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 text-gray-800 dark:text-gray-200 border-gray-200 dark:border-gray-600 hover:from-indigo-100 hover:to-indigo-50 dark:hover:from-indigo-900/30 dark:hover:to-indigo-800/20 hover:text-indigo-800 dark:hover:text-indigo-300 hover:border-indigo-300 dark:hover:border-indigo-700 group-hover:scale-105'
                    ]"
                    :title="copiedLink === link.short_code ? $t('common.copied') : $t('links.copyLinkTitle')"
                  >
                    <div class="flex items-center gap-1.5">
                      <span class="font-bold">{{ link.short_code }}</span>
                      <CheckCircleIcon
                        v-if="copiedLink === link.short_code"
                        className="w-3 h-3 text-emerald-700 dark:text-emerald-400"
                      />
                      <CopyIcon
                        v-else
                        className="w-3 h-3 opacity-0 group-hover:opacity-100 transition-opacity"
                      />
                    </div>
                  </button>
                </td>
                <td class="py-3 px-4">
                  <a
                    :href="link.target_url"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-800 dark:hover:text-indigo-300 truncate block max-w-xs"
                  >
                    {{ link.target_url }}
                  </a>
                </td>
                <td class="py-3 px-4">
                  <div class="flex flex-col gap-1">
                    <span
                      v-if="link.expires_at"
                      :class="[
                        'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium w-fit',
                        isExpired(link.expires_at)
                          ? 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
                          : 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-800 dark:text-emerald-300',
                      ]"
                    >
                      {{ isExpired(link.expires_at) ? $t('links.expired') : $t('links.active') }}
                    </span>
                    <span
                      v-else
                      class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-indigo-100 dark:bg-indigo-900/30 text-indigo-800 dark:text-indigo-300 w-fit"
                    >
                      {{ $t('links.permanent') }}
                    </span>
                    <span
                      v-if="link.expires_at"
                      class="text-xs text-gray-500 dark:text-gray-400"
                    >
                      {{ isExpired(link.expires_at) ? $t('links.expired') : $t('analytics.expires') }}: {{ formatDate(link.expires_at) }}
                    </span>
                  </div>
                </td>
                <td class="py-3 px-4 text-sm text-gray-600 dark:text-gray-400">
                  {{ formatDate(link.created_at) }}
                </td>
                <td class="py-3 px-4">
                  <div class="flex items-center gap-1">
                    <button
                      @click="startEdit(link)"
                      class="p-2 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 rounded-lg transition-colors"
                      :title="$t('common.edit') + ' ' + $t('links.title')"
                    >
                      <EditIcon className="w-4 h-4" />
                    </button>

                    <button
                      @click="confirmDelete(link)"
                      class="p-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                      :title="$t('common.delete') + ' ' + $t('links.title')"
                    >
                      <DeleteIcon className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>

          <!-- 分页控件 -->
          <div v-if="totalPages > 1" class="flex items-center justify-between mt-6 pt-4 border-t-2 border-gray-200/70 dark:border-gray-700/70">
            <div class="flex items-center gap-2">
              <button
                @click="goToPreviousPage"
                :disabled="!hasPrev"
                class="px-4 py-2 text-sm font-bold text-gray-700 dark:text-gray-300 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 hover:from-gray-200 hover:to-gray-100 dark:hover:from-gray-600 dark:hover:to-gray-500 rounded-lg transition-all duration-300 shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ $t('common.previous') }}
              </button>
              <span class="text-sm text-gray-600 dark:text-gray-400">
                {{ $t('links.pagination.showing', {
                  from: (currentPage - 1) * pageSize + 1,
                  to: Math.min(currentPage * pageSize, totalCount),
                  total: totalCount
                }) }}
              </span>
            </div>

            <div class="flex items-center gap-1">
              <button
                v-for="page in visiblePages"
                :key="page"
                @click="goToSpecificPage(page)"
                :class="[
                  'px-4 py-2 text-sm font-bold rounded-lg transition-all duration-300',
                  page === currentPage
                    ? 'bg-indigo-500 dark:bg-indigo-400 text-white'
                    : 'text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'
                ]"
              >
                {{ page }}
              </button>
            </div>

            <div class="flex items-center gap-2">
              <button
                @click="goToNextPage"
                :disabled="!hasNext"
                class="px-4 py-2 text-sm font-bold text-gray-700 dark:text-gray-300 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 hover:from-gray-200 hover:to-gray-100 dark:hover:from-gray-600 dark:hover:to-gray-500 rounded-lg transition-all duration-300 shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ $t('common.next') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 删除确认模态框 -->
    <Transition
      name="modal-backdrop"
      enter-active-class="transition-opacity duration-300 ease-out"
      leave-active-class="transition-opacity duration-200 ease-in"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="showDeleteModal"
        class="fixed inset-0 bg-black/30 dark:bg-black/50 backdrop-blur-md flex items-center justify-center z-50 p-4"
        @click="handleBackdropClick"
      >
        <Transition
          name="modal-content"
          enter-active-class="transition-transform duration-300 ease-out"
          leave-active-class="transition-transform duration-200 ease-in"
          enter-from-class="scale-95"
          enter-to-class="scale-100"
          leave-from-class="scale-100"
          leave-to-class="scale-95"
        >
          <div
            v-if="showDeleteModal"
            class="relative bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm rounded-2xl shadow-2xl max-w-md w-full p-8 border border-gray-200/50 dark:border-gray-700/50"
          >
            <div class="absolute inset-0 bg-gradient-to-br from-white/60 to-red-50/30 dark:from-gray-800/60 dark:to-red-900/20 rounded-2xl"></div>
            <div class="relative text-center">
              <div class="w-16 h-16 mx-auto mb-6 bg-gradient-to-br from-red-100 to-red-50 dark:from-red-900/50 dark:to-red-800/30 rounded-full flex items-center justify-center shadow-lg">
                <DeleteIcon className="w-8 h-8 text-red-600 dark:text-red-400" />
              </div>

              <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-3">{{ $t('links.deleteLink') }}</h3>
              <p class="text-sm text-gray-700 dark:text-gray-300 mb-8 leading-relaxed">
                {{ $t('links.deleteConfirmation', { code: deletingLink?.short_code }) }}
              </p>

              <div class="flex justify-center space-x-4">
                <button
                  @click="closeDeleteModal"
                  class="px-6 py-3 text-sm font-bold text-gray-700 dark:text-gray-300 bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 hover:from-gray-200 hover:to-gray-100 dark:hover:from-gray-600 dark:hover:to-gray-500 rounded-xl transition-all duration-300 shadow-sm hover:shadow-md transform hover:scale-105 uppercase tracking-wide"
                >
                  {{ $t('common.cancel') }}
                </button>
                <button
                  @click="handleDelete"
                  :disabled="loading"
                  class="px-6 py-3 text-sm font-bold text-white bg-gradient-to-r from-red-500 to-red-600 dark:from-red-400 dark:to-red-500 hover:from-red-600 hover:to-red-700 dark:hover:from-red-500 dark:hover:to-red-600 rounded-xl transition-all duration-300 disabled:opacity-50 shadow-lg hover:shadow-xl transform hover:scale-105 active:scale-95 uppercase tracking-wide"
                >
                  {{ loading ? $t('links.deleting') : $t('common.delete') }}
                </button>
              </div>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>

    <!-- 复制成功提示 Toast -->
    <div
      v-if="showCopyToast"
      class="fixed top-6 right-6 z-50 bg-gradient-to-r from-emerald-500 to-emerald-600 dark:from-emerald-400 dark:to-emerald-500 text-white px-6 py-4 rounded-2xl shadow-2xl transform transition-all duration-300 ease-out border border-emerald-400/50 dark:border-emerald-500/50"
      :class="showCopyToast ? 'translate-x-0 opacity-100 scale-100' : 'translate-x-full opacity-0 scale-95'"
    >
      <div class="flex items-center gap-3">
        <CheckCircleIcon className="w-6 h-6" />
        <span class="font-bold text-lg">{{ $t('links.linkCopied') }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, watch, computed } from 'vue'
import { useLinksStore } from '@/stores/links'
import { storeToRefs } from 'pinia'
import { LinkIcon, PlusIcon, EditIcon, ChevronDownIcon, SpinnerIcon, CopyIcon, DeleteIcon, CheckCircleIcon } from '@/components/icons/index'
import type { SerializableShortLink, LinkPayload, GetLinksQuery } from '@/services/api'
import { useI18n } from 'vue-i18n'

const linksStore = useLinksStore()
const { links, loading, error, totalCount, currentPage, pageSize, hasNext, hasPrev } = storeToRefs(linksStore)
const { fetchLinks, createLink, updateLink, deleteLink, applyFilter, resetFilter, goToPage } = linksStore

const { t } = useI18n()

const showForm = ref(false)
const showFilter = ref(false)
const showDeleteModal = ref(false)
const editingLink = ref<SerializableShortLink | null>(null)
const deletingLink = ref<SerializableShortLink | null>(null)
const copiedLink = ref<string | null>(null)
const showCopyToast = ref(false)

const formData = reactive<LinkPayload>({
  code: '',
  target: '',
  expires_at: null,
})

const filterForm = reactive({
  search: '',
  status: '',
  page_size: 20,
  created_after: '',
  created_before: '',
})

// 计算属性
const totalPages = computed(() => Math.ceil(totalCount.value / pageSize.value))

const hasActiveFilters = computed(() => {
  return !!(filterForm.search || filterForm.status || filterForm.created_after || filterForm.created_before)
})

const activeFilterCount = computed(() => {
  let count = 0
  if (filterForm.search) count++
  if (filterForm.status) count++
  if (filterForm.created_after) count++
  if (filterForm.created_before) count++
  return count
})

// 计算属性 - 根据筛选条件过滤链接
const filteredLinks = computed(() => {
  let filtered = [...links.value]

  // 如果选择了永久或临时链接筛选
  if (filterForm.status === 'permanent') {
    filtered = filtered.filter(link => !link.expires_at)
  } else if (filterForm.status === 'temporary') {
    filtered = filtered.filter(link => !!link.expires_at)
  }

  return filtered
})

const visiblePages = computed(() => {
  const total = totalPages.value
  const current = currentPage.value
  const delta = 2

  let start = Math.max(1, current - delta)
  let end = Math.min(total, current + delta)

  if (end - start < 4) {
    if (start === 1) {
      end = Math.min(total, start + 4)
    } else if (end === total) {
      start = Math.max(1, end - 4)
    }
  }

  const pages = []
  for (let i = start; i <= end; i++) {
    pages.push(i)
  }
  return pages
})

// 筛选相关方法
function toggleFilter() {
  showFilter.value = !showFilter.value
}

function applyFilters() {
  const query: GetLinksQuery = {
    page: 1,
    page_size: filterForm.page_size,
  }

  if (filterForm.search) {
    query.search = filterForm.search
  }

  // 根据状态设置相应的布尔值
  if (filterForm.status === 'active') {
    query.only_active = true
  } else if (filterForm.status === 'expired') {
    query.only_expired = true
  }

  if (filterForm.created_after) {
    query.created_after = new Date(filterForm.created_after).toISOString()
  }

  if (filterForm.created_before) {
    // 设置为当天的结束时间
    const date = new Date(filterForm.created_before)
    date.setHours(23, 59, 59, 999)
    query.created_before = date.toISOString()
  }

  // 如果选择了永久或临时，我们需要在客户端进行额外筛选
  // 因为后端API目前不直接支持这种筛选
  applyFilter(query)
}

function resetFilters() {
  filterForm.search = ''
  filterForm.status = ''
  filterForm.page_size = 20
  filterForm.created_after = ''
  filterForm.created_before = ''
  resetFilter()
}

// 分页相关方法
function goToPreviousPage() {
  if (hasPrev.value) {
    goToPage(currentPage.value - 1)
  }
}

function goToNextPage() {
  if (hasNext.value) {
    goToPage(currentPage.value + 1)
  }
}

function goToSpecificPage(page: number) {
  goToPage(page)
}

// 监听筛选表单变化，实时应用筛选
watch([() => filterForm.search], () => {
  // 只对搜索框做防抖处理，其他筛选条件手动触发
  clearTimeout(filterTimeout)
  filterTimeout = setTimeout(() => {
    if (filterForm.search || hasOtherActiveFilters()) {
      applyFilters()
    }
  }, 500)
})

function hasOtherActiveFilters() {
  return !!(filterForm.status || filterForm.created_after || filterForm.created_before)
}

let filterTimeout: NodeJS.Timeout

// 监听编辑状态，自动展开表单
watch(editingLink, (newValue) => {
  if (newValue) {
    showForm.value = true
  }
})

function toggleForm() {
  showForm.value = !showForm.value
}

function collapseForm() {
  showForm.value = false
  if (!editingLink.value) {
    resetForm()
  }
}

function showFormAndFocus() {
  showForm.value = true
  // 等待DOM更新后聚焦到目标URL输入框
  setTimeout(() => {
    const targetInput = document.querySelector('input[type="url"]') as HTMLInputElement
    if (targetInput) {
      targetInput.focus()
    }
  }, 300)
}

function resetForm() {
  formData.code = ''
  formData.target = ''
  formData.expires_at = null
}

function startEdit(link: SerializableShortLink) {
  formData.code = link.short_code
  formData.target = link.target_url
  // 将 RFC3339 时间转换为 datetime-local 格式
  formData.expires_at = link.expires_at ? formatDateTimeLocal(link.expires_at) : null
  editingLink.value = { ...link }
  showForm.value = true

  // 滚动到表单顶部
  setTimeout(() => {
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }, 100)
}

function cancelEdit() {
  editingLink.value = null
  resetForm()
}

function confirmDelete(link: SerializableShortLink) {
  deletingLink.value = link
  showDeleteModal.value = true
}

function closeDeleteModal() {
  showDeleteModal.value = false
  deletingLink.value = null
}

function handleBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    closeDeleteModal()
  }
}

async function handleSave() {
  try {
    const payload: LinkPayload = {
      code: formData.code || undefined,
      target: formData.target,
      // 将 datetime-local 格式转换为 RFC3339
      expires_at: formData.expires_at ? formatToRFC3339(formData.expires_at) : null,
    }

    if (editingLink.value) {
      await updateLink(editingLink.value.short_code, payload)
      editingLink.value = null
    } else {
      await createLink(payload)
    }
    resetForm()
    showForm.value = false
  } catch (err) {
    console.error('Save failed:', err)
  }
}

async function handleDelete() {
  if (deletingLink.value) {
    try {
      await deleteLink(deletingLink.value.short_code)
      closeDeleteModal()
    } catch (err) {
      console.error('Delete failed:', err)
    }
  }
}

async function copyShortLink(code: string) {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || window.location.origin
  const shortUrl = `${baseUrl}/${code}`

  try {
    await navigator.clipboard.writeText(shortUrl)

    // 设置当前复制的链接
    copiedLink.value = code
    showCopyToast.value = true

    // 2秒后重置状态
    setTimeout(() => {
      copiedLink.value = null
    }, 2000)

    // 3秒后隐藏 Toast
    setTimeout(() => {
      showCopyToast.value = false
    }, 3000)

    console.log('Link copied to clipboard:', shortUrl)
  } catch (err) {
    console.error('Failed to copy link:', err)
    // 可以添加错误提示
  }
}

/**
 * 将 RFC3339 时间格式转换为 datetime-local 输入框格式
 * @param rfc3339Time RFC3339 格式的时间字符串
 * @returns datetime-local 格式的时间字符串
 */
function formatDateTimeLocal(rfc3339Time: string): string {
  const date = new Date(rfc3339Time)
  // 获取本地时间偏移
  const offset = date.getTimezoneOffset()
  const localDate = new Date(date.getTime() - offset * 60 * 1000)
  // 转换为 YYYY-MM-DDTHH:mm 格式
  return localDate.toISOString().slice(0, 16)
}

/**
 * 将 datetime-local 格式转换为 RFC3339 时间格式
 * @param datetimeLocal datetime-local 格式的时间字符串
 * @returns RFC3339 格式的时间字符串
 */
function formatToRFC3339(datetimeLocal: string): string {
  const date = new Date(datetimeLocal)
  return date.toISOString()
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function isExpired(expiresAt: string) {
  return new Date(expiresAt) < new Date()
}

onMounted(() => {
  fetchLinks()
})
</script>

<style scoped>
/* 现代化样式增强 */
.max-h-0 {
  max-height: 0;
}

.max-h-96 {
  max-height: 24rem;
}

/* 更流畅的模态框动画 */
.modal-backdrop-enter-active {
  transition: opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.modal-backdrop-leave-active {
  transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.modal-content-enter-active {
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.modal-content-leave-active {
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
