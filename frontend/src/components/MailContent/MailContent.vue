<template>
  <div class="flex flex-col h-full bg-white">
    <div v-if="loading" class="flex items-center justify-center h-full">
      <div class="text-gray-500">Loading email...</div>
    </div>

    <div v-else-if="error" class="flex items-center justify-center h-full">
      <div class="text-red-500">{{ error }}</div>
    </div>

    <div v-else-if="email" class="flex flex-col h-full overflow-y-auto">
      <MailHeader :email="email" />

      <div class="flex items-center gap-2 mt-4">
        <button
          class="px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors flex items-center gap-1"
          @click="downloadRawEmail"
        >
          <ArrowDownTrayIcon class="h-4 w-4" />
          Download Raw
        </button>
        <button
          class="px-3 py-1.5 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
          @click="handleArchive"
        >
          {{ email.archived ? 'Unarchive' : 'Archive' }}
        </button>
      </div>

      <!-- Attachments -->
      <div
        v-if="email.attachments && email.attachments.length > 0"
        class="border-b border-gray-200 px-6 py-3 bg-gray-50"
      >
        <div class="flex items-center gap-2 mb-2">
          <PaperClipIcon class="h-4 w-4 text-gray-500" />
          <span class="text-sm font-medium text-gray-700">
            {{ email.attachments.length }}
            {{ email.attachments.length === 1 ? 'attachment' : 'attachments' }}
          </span>
        </div>
        <div class="flex flex-wrap gap-2">
          <a
            v-for="attachment in email.attachments"
            :key="attachment.id"
            :href="getAttachmentUrl(attachment.id)"
            class="flex items-center gap-2 px-3 py-2 text-sm text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 hover:border-gray-400 transition-colors"
          >
            <PaperClipIcon class="h-4 w-4 text-gray-500" />
            <span>{{ attachment.filename || 'Unnamed attachment' }}</span>
            <span class="text-xs text-gray-500"
              >({{ formatFileSize(attachment.size) }})</span
            >
          </a>
        </div>
      </div>

      <div class="mt-4">
        <Tabs v-model="viewMode" :tabs="emailTabs" />
      </div>

      <!-- Email Body -->
      <div
        class="flex-1 px-6 py-6"
        :class="{
          'flex flex-col':
            (viewMode === 'rendered' && email.rendered_body_html) ||
            (viewMode === 'html' && email.body_html) ||
            (viewMode === 'text' &&
              !email.body_text &&
              email.rendered_body_html),
        }"
      >
        <!-- Rendered HTML View -->
        <iframe
          v-if="viewMode === 'rendered' && email.rendered_body_html"
          :srcdoc="email.rendered_body_html"
          class="w-full border-0 flex-1 min-h-[400px]"
        ></iframe>

        <!-- Unrendered HTML View -->
        <CodeViewer
          v-else-if="viewMode === 'html' && email.body_html"
          :content="email.body_html"
        />

        <!-- Text Body -->
        <div
          v-else-if="viewMode === 'text' && email.body_text"
          class="max-w-none whitespace-pre-wrap"
        >
          {{ email.body_text }}
        </div>

        <!-- Raw Email View -->
        <CodeViewer v-else-if="viewMode === 'raw'" :content="rawEmailContent" />

        <!-- Fallback: Show text if rendered not available -->
        <div
          v-else-if="
            viewMode === 'rendered' &&
            !email.rendered_body_html &&
            email.body_text
          "
          class="max-w-none whitespace-pre-wrap"
        >
          {{ email.body_text }}
        </div>

        <!-- No content available -->
        <div
          v-else-if="
            !email.rendered_body_html && !email.body_html && !email.body_text
          "
          class="text-gray-500 italic"
        >
          No content available
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, onMounted, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'
  import {
    PaperClipIcon,
    ArrowDownTrayIcon,
    CodeBracketIcon,
  } from '@heroicons/vue/24/outline'
  import { apiClient } from '@/api/client'
  import type { EmailRecord } from '@/types/email'
  import Tabs from '@/components/shared/Tabs/Tabs.vue'
  import CodeViewer from '@/components/shared/CodeViewer/CodeViewer.vue'
  import MailHeader from './MailHeader.vue'

  const route = useRoute()
  const router = useRouter()
  const mailId = ref<string>(route.params.id as string)
  const email = ref<EmailRecord | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  const viewMode = ref<'rendered' | 'html' | 'text' | 'raw'>('rendered')
  const rawEmailContent = ref<string>('')

  const emailTabs = computed(() => {
    return [
      {
        id: 'rendered',
        label: 'Rendered',
        visible: !!email.value?.rendered_body_html,
      },
      {
        id: 'text',
        label: 'Text',
        visible: !!email.value?.body_text,
      },
      {
        id: 'html',
        label: 'HTML',
        visible: !!email.value?.body_html,
        icon: CodeBracketIcon,
      },
      {
        id: 'raw',
        label: 'Raw',
        visible: true,
        icon: CodeBracketIcon,
      },
    ]
  })

  const fetchEmail = async () => {
    if (!mailId.value) {
      error.value = 'No email ID provided'
      loading.value = false
      return
    }

    try {
      loading.value = true
      error.value = null
      email.value = await apiClient.getEmail(mailId.value)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch email'
    } finally {
      loading.value = false
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`
  }

  const getAttachmentUrl = (id: string) => {
    return apiClient.getAttachmentUrl(id)
  }

  const loadRawEmail = async () => {
    if (rawEmailContent.value) return // Already loaded
    if (!email.value) return

    try {
      rawEmailContent.value = await apiClient.getRawEmail(mailId.value)
    } catch (err) {
      console.error('Failed to load raw email:', err)
      rawEmailContent.value = 'Failed to load raw email content'
    }
  }

  watch(viewMode, (newValue: 'rendered' | 'html' | 'text' | 'raw') => {
    if (newValue === 'raw' && !rawEmailContent.value) {
      loadRawEmail()
    }
  })

  const downloadRawEmail = async () => {
    if (!email.value) return

    try {
      const rawData = await apiClient.getRawEmail(mailId.value)
      const blob = new Blob([rawData], { type: 'message/rfc822' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `email-${mailId.value}.eml`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    } catch (err) {
      console.error('Failed to download raw email:', err)
      error.value = 'Failed to download raw email'
    }
  }

  const handleArchive = async () => {
    if (!email.value) return

    try {
      const newArchivedState = !email.value.archived
      await apiClient.archive(mailId.value, newArchivedState)
      email.value.archived = newArchivedState

      // Navigate back to inbox if archiving
      if (newArchivedState) {
        const currentPath = router.currentRoute.value.path
        const basePath = currentPath.split('/').slice(0, 3).join('/')
        router.push(basePath)
      }
    } catch (err) {
      console.error('Failed to archive email:', err)
      error.value = 'Failed to archive email'
    }
  }

  // Set initial view mode based on available content
  watch(
    () => email.value,
    (newEmail: EmailRecord | null) => {
      if (newEmail) {
        // Prefer rendered HTML if available, otherwise text
        viewMode.value = newEmail.rendered_body_html ? 'rendered' : 'text'
      }
    },
    { immediate: true }
  )

  onMounted(() => {
    fetchEmail()
  })

  watch(
    () => route.params.id,
    newId => {
      mailId.value = newId as string
      fetchEmail()
    }
  )
</script>
