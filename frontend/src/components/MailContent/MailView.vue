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

      <MailAttachments v-if="email.attachments.length > 0" :email="email" />

      <div class="mt-4">
        <Tabs v-model="viewMode" :tabs="emailTabs" />
      </div>

      <MailContent :email="email" :view-mode="viewMode" />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, onMounted, watch } from 'vue'
  import type { Component } from 'vue'
  import { useRoute } from 'vue-router'
  import { CodeBracketIcon } from '@heroicons/vue/24/outline'
  import { apiClient } from '@/api/client'
  import type { EmailRecord } from '@/types/email'
  import Tabs from '@/components/shared/Tabs/Tabs.vue'
  import MailHeader from './MailHeader.vue'
  import MailAttachments from './MailAttachments.vue'
  import MailContent from './MailContent.vue'

  const route = useRoute()
  const mailId = ref<string>(route.params.id as string)
  const email = ref<EmailRecord | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  const viewMode = ref<ViewMode>('rendered')

  export type ViewMode = 'rendered' | 'html' | 'text' | 'raw'
  type Tab = {
    id: ViewMode
    label: string
    icon?: Component
  }

  const emailTabs = computed(() => {
    const tabs: Tab[] = []

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

  watch(
    () => email.value,
    (newEmail: EmailRecord | null) => {
      if (newEmail) {
        viewMode.value = emailTabs.value[0]?.id || 'raw'
      }
    },
    { immediate: true }
  )

  onMounted(fetchEmail)

  watch(
    () => route.params.id,
    newId => {
      mailId.value = newId as string
      fetchEmail()
    }
  )
</script>
