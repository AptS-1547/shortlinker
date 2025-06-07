<template>
  <div class="space-y-6">
    <!-- 页面标题 -->
    <div class="mb-4">
      <h1 class="text-2xl font-bold text-gray-900">{{ $t('links.title') }}</h1>
      <p class="text-gray-700">{{ $t('links.description') }}</p>
    </div>

    <!-- 筛选器 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <div
        class="p-4 border-b border-gray-200 cursor-pointer hover:bg-gray-50 transition-colors"
        @click="toggleFilter"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="p-2 bg-blue-100 text-blue-600 rounded-lg">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.207A1 1 0 013 6.5V4z"></path>
              </svg>
            </div>
            <div>
              <h2 class="text-lg font-semibold text-gray-900">{{ $t('links.filterTitle') }}</h2>
              <p class="text-sm text-gray-700 mt-1">{{ $t('links.filterDescription') }}</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span
              v-if="hasActiveFilters"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-700"
            >
              {{ $t('links.activeFilters', { count: activeFilterCount }) }}
            </span>
            <ChevronDownIcon
              :class="[
                'w-5 h-5 text-gray-500 transition-transform duration-200',
                { 'rotate-180': showFilter },
              ]"
            />
          </div>
        </div>
      </div>

      <div
        :class="[
          'overflow-hidden transition-all duration-300 ease-in-out',
          showFilter ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0',
        ]"
      >
        <div class="p-4">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-900 mb-1">{{ $t('common.search') }}</label>
              <input
                v-model="filterForm.search"
                type="text"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
                :placeholder="$t('links.search.placeholder')"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-900 mb-1">{{ $t('links.status') }}</label>
              <select
                v-model="filterForm.status"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
              >
                <option value="">{{ $t('links.filterOptions.allLinks') }}</option>
                <option value="active">{{ $t('links.filterOptions.activeOnly') }}</option>
                <option value="expired">{{ $t('links.filterOptions.expiredOnly') }}</option>
                <option value="permanent">{{ $t('links.filterOptions.permanentLinks') }}</option>
                <option value="temporary">{{ $t('links.filterOptions.temporaryLinks') }}</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-900 mb-1">{{ $t('links.createdAfter') }}</label>
              <input
                v-model="filterForm.created_after"
                type="date"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-900 mb-1">{{ $t('links.createdBefore') }}</label>
              <input
                v-model="filterForm.created_before"
                type="date"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-900 mb-1">{{ $t('links.pageSize') }}</label>
              <select
                v-model="filterForm.page_size"
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
              >
                <option :value="10">10 per page</option>
                <option :value="20">20 per page</option>
                <option :value="50">50 per page</option>
                <option :value="100">100 per page</option>
              </select>
            </div>
          </div>

          <div class="flex items-center justify-between pt-4 border-t border-gray-200 mt-4">
            <div class="flex items-center gap-2">
              <button
                @click="resetFilters"
                class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
              >
                {{ $t('common.reset') }}
              </button>
            </div>
            <button
              @click="applyFilters"
              class="px-4 py-2 bg-indigo-500 text-white font-medium rounded-lg hover:bg-indigo-600 transition-colors"
            >
              {{ $t('common.apply') }} {{ $t('common.filter') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 创建/编辑链接表单 - 可折叠 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <div
        class="p-4 border-b border-gray-200 cursor-pointer hover:bg-gray-50 transition-colors"
        @click="toggleForm"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div
              :class="[
                'p-2 rounded-lg flex items-center justify-center transition-colors',
                editingLink ? 'bg-amber-100 text-amber-600' : 'bg-indigo-100 text-indigo-600',
              ]"
            >
              <EditIcon
                v-if="editingLink"
                className="w-5 h-5"
              />
              <PlusIcon v-else className="w-5 h-5" />
            </div>
            <div>
              <h2 class="text-lg font-semibold text-gray-900">
                {{ editingLink ? $t('links.editTitle') : $t('links.createTitle') }}
              </h2>
              <p class="text-sm text-gray-700 mt-1">
                {{ editingLink ? $t('links.editDescription') : $t('links.createDescription') }}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span
              v-if="editingLink"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-indigo-100 text-indigo-700"
            >
              {{ $t('links.editing', { code: editingLink.short_code }) }}
            </span>
            <ChevronDownIcon
              :class="[
                'w-5 h-5 text-gray-500 transition-transform duration-200',
                { 'rotate-180': showForm },
              ]"
            />
          </div>
        </div>
      </div>

      <!-- 可折叠的表单内容 -->
      <div
        :class="[
          'overflow-hidden transition-all duration-300 ease-in-out',
          showForm ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0',
        ]"
      >
        <div class="p-4">
          <form @submit.prevent="handleSave" class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-900 mb-1">
                  {{ $t('links.shortCodeOptional') }}
                </label>
                <input
                  v-model="formData.code"
                  type="text"
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
                  :placeholder="$t('links.shortCodePlaceholder')"
                />
                <p class="text-xs text-gray-500 mt-1">
                  {{ $t('links.shortCodeHelp') }}
                </p>
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-900 mb-1">
                  {{ $t('links.expiresAtOptional') }}
                </label>
                <input
                  v-model="formData.expires_at"
                  type="datetime-local"
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
                />
                <p class="text-xs text-gray-500 mt-1">{{ $t('links.expiresAtHelp') }}</p>
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-900 mb-1">{{ $t('links.targetUrlRequired') }}</label>
              <input
                v-model="formData.target"
                type="url"
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
                :placeholder="$t('links.targetUrlPlaceholder')"
              />
              <p class="text-xs text-gray-500 mt-1">
                {{ $t('links.targetUrlHelp') }}
              </p>
            </div>

            <div class="flex items-center justify-between pt-3 border-t border-gray-200">
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  @click="collapseForm"
                  class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                >
                  {{ $t('common.cancel') }}
                </button>
                <button
                  v-if="editingLink"
                  type="button"
                  @click="cancelEdit"
                  class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                >
                  {{ $t('links.clearEdit') }}
                </button>
              </div>

              <button
                type="submit"
                :disabled="loading || !formData.target"
                class="px-5 py-2 bg-gradient-to-r from-indigo-500 to-indigo-600 text-white font-medium rounded-lg hover:from-indigo-600 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 transform hover:scale-105 active:scale-95"
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

    <!-- 链接列表 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200 transition-all duration-200 hover:shadow-md hover:-translate-y-0.5">
      <div class="p-4 border-b border-gray-200">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="p-2 bg-gray-100 rounded-lg">
              <LinkIcon className="w-5 h-5 text-gray-600" />
            </div>
            <h2 class="text-xl font-bold text-gray-900">{{ $t('links.shortLinks') }}</h2>
          </div>
          <div class="flex items-center gap-4">
            <span class="text-sm text-gray-500">
              {{ filteredLinks.length }}
              <span v-if="filteredLinks.length !== totalCount">of {{ totalCount }}</span>
              total
              <span v-if="hasActiveFilters">(filtered)</span>
            </span>
            <span class="text-sm text-gray-500">
              Page {{ currentPage }} of {{ totalPages }}
            </span>
          </div>
        </div>
      </div>

      <div class="p-4">
        <div v-if="loading && links.length === 0" class="text-center py-8">
          <div class="animate-spin mx-auto mb-4">
            <SpinnerIcon className="h-8 w-8 text-indigo-500" />
          </div>
          <p class="text-gray-500">{{ $t('common.loading') }}...</p>
        </div>

        <div v-else-if="error" class="text-center py-8">
          <div class="text-red-500 mb-4">{{ error }}</div>
          <button
            @click="() => fetchLinks()"
            class="px-4 py-2 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 transition-colors"
          >
            {{ $t('common.retry') }}
          </button>
        </div>

        <div v-else-if="filteredLinks.length === 0" class="text-center py-8">
          <div
            class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center"
          >
            <LinkIcon className="w-8 h-8 text-gray-400" />
          </div>
          <h3 class="text-lg font-medium text-gray-900 mb-2">
            {{ hasActiveFilters ? $t('links.noMatchingLinks') : $t('links.noLinksYet') }}
          </h3>
          <p class="text-gray-700 mb-4">
            {{ hasActiveFilters ? $t('links.noMatchingLinksDesc') : $t('links.noLinksYetDesc') }}
          </p>
          <div class="flex items-center justify-center gap-2">
            <button
              v-if="hasActiveFilters"
              @click="resetFilters"
              class="inline-flex items-center px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors"
            >
              {{ $t('links.clearFilters') }}
            </button>
            <button
              @click="showFormAndFocus"
              class="inline-flex items-center px-4 py-2 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 transition-colors"
            >
              <PlusIcon className="w-4 h-4 mr-2" />
              {{ hasActiveFilters ? $t('links.createNewLink') : $t('links.createFirstLink') }}
            </button>
          </div>
        </div>

        <div v-else class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b border-gray-200">
                <th class="text-left py-3 px-4 font-semibold text-gray-900">{{ $t('links.table.code') }}</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-900">{{ $t('links.table.targetUrl') }}</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-900">{{ $t('links.table.status') }}</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-900">{{ $t('links.table.created') }}</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-900">{{ $t('links.table.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="link in filteredLinks"
                :key="link.short_code"
                class="border-b border-gray-100 hover:bg-gray-50 transition-colors group"
              >
                <td class="py-4 px-4">
                  <button
                    @click="copyShortLink(link.short_code)"
                    :class="[
                      'font-mono text-sm px-3 py-2 rounded-lg transition-all duration-200 border-2',
                      copiedLink === link.short_code
                        ? 'bg-emerald-100 text-emerald-700 border-emerald-300 scale-105'
                        : 'bg-gray-100 text-gray-800 border-gray-200 hover:bg-indigo-100 hover:text-indigo-700 hover:border-indigo-300 group-hover:scale-105'
                    ]"
                    :title="copiedLink === link.short_code ? $t('common.copied') : $t('links.copyLinkTitle')"
                  >
                    <div class="flex items-center gap-2">
                      <span>{{ link.short_code }}</span>
                      <CheckCircleIcon
                        v-if="copiedLink === link.short_code"
                        className="w-3 h-3 text-emerald-600"
                      />
                      <CopyIcon
                        v-else
                        className="w-3 h-3 opacity-0 group-hover:opacity-100 transition-opacity"
                      />
                    </div>
                  </button>
                </td>
                <td class="py-4 px-4">
                  <a
                    :href="link.target_url"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-indigo-600 hover:text-indigo-800 truncate block max-w-xs"
                  >
                    {{ link.target_url }}
                  </a>
                </td>
                <td class="py-4 px-4">
                  <div class="flex flex-col gap-1">
                    <span
                      v-if="link.expires_at"
                      :class="[
                        'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium w-fit',
                        isExpired(link.expires_at)
                          ? 'bg-red-100 text-red-800'
                          : 'bg-emerald-100 text-emerald-800',
                      ]"
                    >
                      {{ isExpired(link.expires_at) ? $t('links.expired') : $t('links.active') }}
                    </span>
                    <span
                      v-else
                      class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800 w-fit"
                    >
                      {{ $t('links.permanent') }}
                    </span>
                    <span
                      v-if="link.expires_at"
                      class="text-xs text-gray-500"
                    >
                      {{ isExpired(link.expires_at) ? $t('links.expired') : $t('analytics.expires') }}: {{ formatDate(link.expires_at) }}
                    </span>
                  </div>
                </td>
                <td class="py-4 px-4 text-sm text-gray-600">
                  {{ formatDate(link.created_at) }}
                </td>
                <td class="py-4 px-4">
                  <div class="flex items-center gap-1">
                    <button
                      @click="startEdit(link)"
                      class="p-2 text-indigo-600 hover:bg-indigo-50 rounded-lg transition-colors"
                      :title="$t('common.edit') + ' ' + $t('links.title')"
                    >
                      <EditIcon className="w-4 h-4" />
                    </button>

                    <button
                      @click="confirmDelete(link)"
                      class="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
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
          <div v-if="totalPages > 1" class="flex items-center justify-between mt-6 pt-4 border-t border-gray-200">
            <div class="flex items-center gap-2">
              <button
                @click="goToPreviousPage"
                :disabled="!hasPrev"
                class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ $t('common.previous') }}
              </button>
              <span class="text-sm text-gray-600">
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
                  'px-3 py-2 text-sm font-medium rounded-lg transition-colors',
                  page === currentPage
                    ? 'bg-indigo-500 text-white'
                    : 'text-gray-700 bg-gray-100 hover:bg-gray-200'
                ]"
              >
                {{ page }}
              </button>
            </div>

            <div class="flex items-center gap-2">
              <button
                @click="goToNextPage"
                :disabled="!hasNext"
                class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ $t('common.next') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
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
        class="fixed inset-0 bg-black/20 backdrop-blur-md flex items-center justify-center z-50 p-4"
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
            class="bg-white rounded-xl shadow-2xl max-w-md w-full p-5"
          >
            <div class="text-center">
              <div
                class="w-10 h-10 mx-auto mb-3 bg-red-100 rounded-full flex items-center justify-center"
              >
                <DeleteIcon className="w-5 h-5 text-red-600" />
              </div>

              <h3 class="text-lg font-semibold text-gray-900 mb-2">{{ $t('links.deleteLink') }}</h3>
              <p class="text-sm text-gray-700 mb-4">
                {{ $t('links.deleteConfirmation', { code: deletingLink?.short_code }) }}
              </p>

              <div class="flex justify-center space-x-3">
                <button
                  @click="closeDeleteModal"
                  class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                >
                  {{ $t('common.cancel') }}
                </button>
                <button
                  @click="handleDelete"
                  :disabled="loading"
                  class="px-4 py-2 text-sm font-medium text-white bg-red-500 hover:bg-red-600 rounded-lg transition-colors disabled:opacity-50"
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
      class="fixed top-4 right-4 z-50 bg-emerald-500 text-white px-4 py-3 rounded-lg shadow-xl transform transition-all duration-300 ease-out"
      :class="showCopyToast ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'"
    >
      <div class="flex items-center gap-2">
        <CheckCircleIcon className="w-5 h-5" />
        <span class="font-medium">{{ $t('links.linkCopied') }}</span>
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
/* 确保表单动画平滑 */
.max-h-0 {
  max-height: 0;
}

.max-h-96 {
  max-height: 24rem;
}

/* 模态框背景动画 */
.modal-backdrop-enter-active {
  transition: opacity 0.3s ease-out;
}

.modal-backdrop-leave-active {
  transition: opacity 0.2s ease-in;
}

.modal-backdrop-enter-from,
.modal-backdrop-leave-to {
  opacity: 0;
}

.modal-backdrop-enter-to,
.modal-backdrop-leave-from {
  opacity: 1;
}

/* 模态框内容动画 */
.modal-content-enter-active {
  transition: transform 0.3s ease-out;
}

.modal-content-leave-active {
  transition: transform 0.2s ease-in;
}

.modal-content-enter-from,
.modal-content-leave-to {
  transform: scale(0.95);
}

.modal-content-enter-to,
.modal-content-leave-from {
  transform: scale(1);
}
</style>
