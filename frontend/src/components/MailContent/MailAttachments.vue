<template>
  <div class="flex flex-wrap gap-2 p-2 border-b border-gray-200">
    <a
      v-for="attachment in email.attachments"
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

  defineProps<{
    email: EmailRecord
  }>()

  const getAttachmentUrl = (id: string) => {
    return apiClient.getAttachmentUrl(id)
  }

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }
</script>
