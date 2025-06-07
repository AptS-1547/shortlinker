<template>
  <div class="space-y-6">
    <!-- 页面标题 -->
    <div class="mb-4">
      <h1 class="text-2xl font-bold text-gray-900">Link Management</h1>
      <p class="text-gray-600">Create, edit, and manage your short links.</p>
    </div>

    <!-- 创建/编辑链接表单 - 可折叠 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200">
      <div
        class="p-4 border-b border-gray-200 cursor-pointer hover:bg-gray-50 transition-colors"
        @click="toggleForm"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div
              :class="[
                'p-2 rounded-lg flex items-center justify-center transition-colors',
                editingLink ? 'bg-indigo-100 text-indigo-600' : 'bg-blue-100 text-blue-600',
              ]"
            >
              <svg
                v-if="editingLink"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                />
              </svg>
              <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                />
              </svg>
            </div>
            <div>
              <h2 class="text-lg font-semibold text-gray-900">
                {{ editingLink ? 'Edit Link' : 'Create New Link' }}
              </h2>
              <p class="text-sm text-gray-600 mt-1">
                {{
                  editingLink
                    ? 'Update the details of your short link'
                    : 'Add a new short link to your collection'
                }}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span
              v-if="editingLink"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
            >
              Editing: {{ editingLink.short_code }}
            </span>
            <svg
              :class="[
                'w-5 h-5 text-gray-400 transition-transform duration-200',
                { 'rotate-180': showForm },
              ]"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 9l-7 7-7-7"
              />
            </svg>
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
                <label class="block text-sm font-medium text-gray-700 mb-1">
                  Short Code
                  <span class="text-gray-400 font-normal">(optional)</span>
                </label>
                <input
                  v-model="formData.code"
                  type="text"
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                  placeholder="Leave empty for auto-generation"
                />
                <p class="text-xs text-gray-500 mt-1">
                  Custom short code (letters, numbers, hyphens, underscores)
                </p>
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">
                  Expires At
                  <span class="text-gray-400 font-normal">(optional)</span>
                </label>
                <input
                  v-model="formData.expires_at"
                  type="datetime-local"
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                />
                <p class="text-xs text-gray-500 mt-1">Leave empty for permanent links</p>
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1"> Target URL * </label>
              <input
                v-model="formData.target"
                type="url"
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                placeholder="https://example.com/your-long-url"
              />
              <p class="text-xs text-gray-500 mt-1">
                The destination URL where users will be redirected
              </p>
            </div>

            <div class="flex items-center justify-between pt-3 border-t border-gray-200">
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  @click="collapseForm"
                  class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  v-if="editingLink"
                  type="button"
                  @click="cancelEdit"
                  class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                >
                  Clear Edit
                </button>
              </div>

              <button
                type="submit"
                :disabled="loading || !formData.target"
                class="px-5 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-lg hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 transform hover:scale-105 active:scale-95"
              >
                <span v-if="loading" class="flex items-center">
                  <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                    <circle
                      class="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      stroke-width="4"
                    ></circle>
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  {{ editingLink ? 'Updating...' : 'Creating...' }}
                </span>
                <span v-else>
                  {{ editingLink ? 'Update Link' : 'Create Link' }}
                </span>
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- 链接列表 -->
    <div class="bg-white rounded-xl shadow-sm border border-gray-200">
      <div class="p-4 border-b border-gray-200">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="p-2 bg-gray-100 rounded-lg">
              <svg
                class="w-5 h-5 text-gray-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
                />
              </svg>
            </div>
            <h2 class="text-xl font-bold text-gray-900">Short Links</h2>
          </div>
          <span class="text-sm text-gray-500">{{ links.length }} total</span>
        </div>
      </div>

      <div class="p-4">
        <div v-if="loading && links.length === 0" class="text-center py-8">
          <div
            class="animate-spin rounded-full h-8 w-8 border-4 border-blue-600 border-t-transparent mx-auto mb-4"
          ></div>
          <p class="text-gray-500">Loading links...</p>
        </div>

        <div v-else-if="error" class="text-center py-8">
          <div class="text-red-500 mb-4">{{ error }}</div>
          <button
            @click="fetchLinks"
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Retry
          </button>
        </div>

        <div v-else-if="links.length === 0" class="text-center py-8">
          <div
            class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center"
          >
            <LinkIcon class="w-8 h-8 text-gray-400" />
          </div>
          <h3 class="text-lg font-medium text-gray-900 mb-2">No links yet</h3>
          <p class="text-gray-600 mb-4">Create your first short link using the form above.</p>
          <button
            @click="showFormAndFocus"
            class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 6v6m0 0v6m0-6h6m-6 0H6"
              />
            </svg>
            Create Your First Link
          </button>
        </div>

        <div v-else class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b border-gray-200">
                <th class="text-left py-3 px-4 font-semibold text-gray-700">Code</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-700">Target URL</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-700">Status</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-700">Created</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-700">Actions</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="link in links"
                :key="link.short_code"
                class="border-b border-gray-100 hover:bg-gray-50 transition-colors"
              >
                <td class="py-4 px-4">
                  <span class="font-mono text-sm bg-gray-100 px-2 py-1 rounded">
                    {{ link.short_code }}
                  </span>
                </td>
                <td class="py-4 px-4">
                  <a
                    :href="link.target_url"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-blue-600 hover:text-blue-800 truncate block max-w-xs"
                  >
                    {{ link.target_url }}
                  </a>
                </td>
                <td class="py-4 px-4">
                  <span
                    v-if="link.expires_at"
                    :class="[
                      'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
                      isExpired(link.expires_at)
                        ? 'bg-red-100 text-red-800'
                        : 'bg-green-100 text-green-800',
                    ]"
                  >
                    {{ isExpired(link.expires_at) ? 'Expired' : 'Active' }}
                  </span>
                  <span
                    v-else
                    class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
                  >
                    Permanent
                  </span>
                </td>
                <td class="py-4 px-4 text-sm text-gray-600">
                  {{ formatDate(link.created_at) }}
                </td>
                <td class="py-4 px-4">
                  <div class="flex items-center gap-2">
                    <button
                      @click="copyShortLink(link.short_code)"
                      class="p-2 text-gray-600 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                      title="Copy short link"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                        />
                      </svg>
                    </button>

                    <button
                      @click="startEdit(link)"
                      class="p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                      title="Edit link"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                        />
                      </svg>
                    </button>

                    <button
                      @click="confirmDelete(link)"
                      class="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                      title="Delete link"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                        />
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div
      v-if="showDeleteModal"
      class="fixed inset-0 bg-black/20 backdrop-blur-md flex items-center justify-center z-50 p-4"
      @click="handleBackdropClick"
    >
      <div
        class="bg-white/95 backdrop-blur-xl border border-white/20 rounded-xl shadow-2xl max-w-md w-full p-5 transform transition-all duration-300"
      >
        <div class="text-center">
          <div
            class="w-10 h-10 mx-auto mb-3 bg-red-100 rounded-full flex items-center justify-center"
          >
            <svg class="w-5 h-5 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
              />
            </svg>
          </div>

          <h3 class="text-lg font-semibold text-gray-900 mb-2">Delete Link</h3>
          <p class="text-sm text-gray-600 mb-4">
            Are you sure you want to delete the link
            <span class="font-mono font-medium">{{ deletingLink?.short_code }}</span
            >? This action cannot be undone.
          </p>

          <div class="flex justify-center space-x-3">
            <button
              @click="closeDeleteModal"
              class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              @click="handleDelete"
              :disabled="loading"
              class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {{ loading ? 'Deleting...' : 'Delete' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, watch } from 'vue'
import { useLinksStore } from '@/stores/links'
import { storeToRefs } from 'pinia'
import { LinkIcon } from '@heroicons/vue/24/outline'
import type { SerializableShortLink, LinkPayload } from '@/services/api'

const linksStore = useLinksStore()
const { links, loading, error } = storeToRefs(linksStore)
const { fetchLinks, createLink, updateLink, deleteLink } = linksStore

const showForm = ref(false)
const showDeleteModal = ref(false)
const editingLink = ref<SerializableShortLink | null>(null)
const deletingLink = ref<SerializableShortLink | null>(null)

const formData = reactive<LinkPayload>({
  code: '',
  target: '',
  expires_at: null,
})

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
    // TODO: 添加成功提示
    console.log('Link copied to clipboard')
  } catch (err) {
    console.error('Failed to copy link:', err)
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
</style>
