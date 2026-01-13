<template>
  <div class="p-4 shadow-md">
    <div class="flex flex-row justify-between gap-4">
      <EmailAttachments v-if="email.attachments.length > 0" :email="email" />

      <div class="text-gray-500 flex align-center text-nowrap">
        {{ formatSize(email.size) }} ({{ email.size }} bytes)
      </div>
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
