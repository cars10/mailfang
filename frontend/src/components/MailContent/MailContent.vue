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

      <MailAttachments :email="email" />

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
  import { useRoute } from 'vue-router'
  import { CodeBracketIcon } from '@heroicons/vue/24/outline'
  import { apiClient } from '@/api/client'
  import type { EmailRecord } from '@/types/email'
  import Tabs from '@/components/shared/Tabs/Tabs.vue'
  import CodeViewer from '@/components/shared/CodeViewer/CodeViewer.vue'
  import MailHeader from './MailHeader.vue'
  import MailAttachments from './MailAttachments.vue'

  const route = useRoute()
  const mailId = ref<string>(route.params.id as string)
  const email = ref<EmailRecord | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  const viewMode = ref<'rendered' | 'html' | 'text' | 'raw'>('rendered')
  const rawEmailContent = ref<string>('')

  const emailTabs = computed(() => {
    const tabs = []

    if (email.value?.rendered_body_html) {
      tabs.push({ id: 'rendered', label: 'Rendered' })
    }

    if (email.value?.body_text) {
      tabs.push({ id: 'text', label: 'Text' })
    }

    if (email.value?.body_html) {
      tabs.push({ id: 'html', label: 'HTML', icon: CodeBracketIcon })
    }

    tabs.push({ id: 'raw', label: 'Raw', icon: CodeBracketIcon })

    return tabs
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
