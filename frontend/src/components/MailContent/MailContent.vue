<template>
  <div class="flex flex-col grow">
    <div
      v-if="viewMode === 'rendered' && email.body_html"
      class="flex flex-col grow max-w-full"
    >
      <div class="flex flex-row border-b border-gray-200">
        <ZoomControls v-model="mailLayoutStore.mailContentZoom" />
        <SecurityControls />
      </div>

      <div
        v-if="loadingRendered"
        class="flex items-center justify-center h-full"
      >
        <div class="text-gray-500">Loading email content...</div>
      </div>
      <div v-else class="h-full w-full overflow-auto">
        <iframe
          :key="renderedUrl"
          :src="renderedUrl"
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
      <CodeViewer :content="email.body_html" language="xml" />
    </div>

    <div v-if="viewMode === 'raw'" class="p-4 grow">
      <div v-if="loadingRaw" class="text-gray-500">Loading raw email...</div>
      <CodeViewer v-else :content="rawContent" />
    </div>

    <MailHeaders v-if="viewMode === 'headers'" :email="email" />
  </div>
</template>

<script setup lang="ts">
  import { ref, watch, computed } from 'vue'
  import type { EmailRecord } from '@/types/email'
  import CodeViewer from '@/components/shared/CodeViewer/CodeViewer.vue'
  import { apiClient } from '@/api/client'
  import type { ViewMode } from './MailView.vue'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import ZoomControls from './ZoomControls.vue'
  import SecurityControls from './SecurityControls.vue'
  import MailHeaders from './MailHeaders.vue'

  const props = defineProps<{
    email: EmailRecord
    viewMode: ViewMode
  }>()

  const mailLayoutStore = useMailLayoutStore()
  const rawContent = ref<string>('')
  const loadingRaw = ref(false)
  const loadingRendered = ref(false)

  // Computed property for iframe URL that updates when blocking state changes
  const renderedUrl = computed(() => {
    if (!props.email.body_html) return ''
    const blockExternal = mailLayoutStore.blockExternalRequests
    return `/api/emails/${props.email.id}/rendered?block_external_requests=${blockExternal}`
  })

  const loadRawEmail = async () => {
    loadingRaw.value = true
    try {
      rawContent.value = await apiClient.getRawEmail(props.email.id)
    } catch (err) {
      console.error('Failed to load raw email:', err)
      rawContent.value = 'Failed to load raw email content'
    } finally {
      loadingRaw.value = false
    }
  }

  // Watch viewMode to fetch content when switching tabs
  watch(
    () => props.viewMode,
    (newValue: ViewMode) => {
      if (newValue === 'raw') {
        loadRawEmail()
      } else if (newValue === 'rendered' && props.email.body_html) {
        // Set loading state briefly when switching to rendered tab
        loadingRendered.value = true
        // The iframe will load the content, so we can clear loading after a short delay
        setTimeout(() => {
          loadingRendered.value = false
        }, 100)
      }
    },
    { immediate: true }
  )

  // Watch blockExternalRequests to trigger iframe reload
  watch(
    () => mailLayoutStore.blockExternalRequests,
    () => {
      // The iframe src will change, triggering a reload
      // Set loading state briefly
      if (props.viewMode === 'rendered' && props.email.body_html) {
        loadingRendered.value = true
        setTimeout(() => {
          loadingRendered.value = false
        }, 100)
      }
    }
  )
</script>
