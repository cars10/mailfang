<template>
  <div class="p-4 border-t border-gray-300">
    <EmailAttachments
      v-if="email.attachments.length > 0"
      :email="email"
      class="mb-2"
    />

    <div class="text-gray-500 flex align-center text-nowrap">
      Total size: {{ formatSize(email.size) }} ({{ email.size }} bytes)
    </div>
  </div>
</template>

<script setup lang="ts">
  import type { EmailRecord } from '@/types/email'
  import EmailAttachments from './EmailAttachments.vue'
  import prettyBytes from 'pretty-bytes'

  defineProps<{ email: EmailRecord }>()

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }
</script>
