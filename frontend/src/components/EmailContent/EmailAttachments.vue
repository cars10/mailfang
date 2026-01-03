<template>
  <div
    v-if="attachments.length > 0"
    class="flex flex-wrap gap-2 p-2 border-b border-gray-200"
  >
    <div class="text-sm text-gray-600 flex items-center gap-1">
      Attachments:
    </div>
    <a
      v-for="attachment in attachments"
      :key="attachment.id"
      :href="getAttachmentUrl(attachment.id)"
      target="_blank"
      class="btn btn--small"
    >
      <PaperClipIcon class="h-4 w-4 text-gray-600" />
      <span>{{ attachment.filename || 'unnamed' }}</span>
      <span class="text-xs text-gray-600"
        >({{ formatSize(attachment.size) }})</span
      >
    </a>
  </div>

  <div
    v-if="inlineAttachments.length > 0"
    class="flex flex-wrap gap-2 p-2 border-b border-gray-200"
  >
    <div
      class="text-sm text-gray-600 flex items-center gap-1 cursor-help"
      title="Inline attachments are usually images displayed inside the email content and not typically intended to be downloaded."
    >
      Inline attachments:
    </div>
    <a
      v-for="attachment in inlineAttachments"
      :key="attachment.id"
      :href="getAttachmentUrl(attachment.id)"
      target="_blank"
      class="btn btn--small"
    >
      <PaperClipIcon class="h-4 w-4 text-gray-600" />
      <span>{{ attachment.filename || 'unnamed' }}</span>
      <span class="text-xs text-gray-600"
        >({{ formatSize(attachment.size) }})</span
      >
    </a>
  </div>
</template>

<script setup lang="ts">
  import type { EmailRecord } from '@/types/email'
  import prettyBytes from 'pretty-bytes'
  import { apiClient } from '@/api/client'
  import { PaperClipIcon } from '@heroicons/vue/24/outline'
  import { computed } from 'vue'

  const props = defineProps<{
    email: EmailRecord
  }>()

  const inlineAttachments = computed(() => {
    return props.email.attachments.filter(
      attachment => attachment.disposition === 'inline'
    )
  })

  const attachments = computed(() => {
    return props.email.attachments.filter(
      attachment => attachment.disposition !== 'inline'
    )
  })

  const getAttachmentUrl = (id: string) => {
    return apiClient.getAttachmentUrl(id)
  }

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }
</script>
