<template>
  <div class="flex flex-col grow">
    <div
      v-if="viewMode === 'rendered' && email.rendered_body_html"
      class="flex flex-col grow max-w-full relative"
    >
      <ZoomControls v-model="mailLayoutStore.mailContentZoom" />
      <div class="h-full w-full overflow-auto">
        <iframe
          :srcdoc="email.rendered_body_html"
          class="border-0 h-full w-full max-w-full"
          :style="{ zoom: mailLayoutStore.mailContentZoom }"
        ></iframe>
      </div>
    </div>

    <div
      v-if="viewMode === 'text' && email.body_text"
      class="grow whitespace-pre-wrap p-4"
    >
      {{ email.body_text }}
    </div>

    <div v-if="viewMode === 'html' && email.body_html" class="p-4 grow">
      <CodeViewer :content="email.body_html" />
    </div>

    <div v-if="viewMode === 'raw'" class="p-4 grow">
      <CodeViewer :content="rawEmailContent || ''" />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, watch } from 'vue'
  import type { EmailRecord } from '@/types/email'
  import CodeViewer from '@/components/shared/CodeViewer/CodeViewer.vue'
  import { apiClient } from '@/api/client'
  import type { ViewMode } from './MailView.vue'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import ZoomControls from './ZoomControls.vue'

  const props = defineProps<{
    email: EmailRecord
    viewMode: ViewMode
  }>()

  const mailLayoutStore = useMailLayoutStore()
  const rawEmailContent = ref<string | null>(null)

  const loadRawEmail = async () => {
    if (rawEmailContent.value) return

    try {
      rawEmailContent.value = await apiClient.getRawEmail(props.email.id)
    } catch (err) {
      console.error('Failed to load raw email:', err)
      rawEmailContent.value = 'Failed to load raw email content'
    }
  }

  watch(
    () => props.viewMode,
    (newValue: ViewMode) => {
      if (newValue === 'raw' && !rawEmailContent.value) {
        loadRawEmail()
      }
    }
  )
</script>
