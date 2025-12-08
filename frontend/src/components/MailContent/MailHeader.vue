<template>
  <div class="border-b border-gray-200 bg-gray-50 px-6 py-4">
    <div class="flex items-start justify-between gap-2 mb-4">
      <h1 class="text-2xl text-gray-900">
        {{ email.subject || '(No Subject)' }}
      </h1>
      <div class="text-gray-600 text-nowrap flex flex-col">
        <div>
          {{ formatFullDate(email.date || email.created_at) }}
        </div>
        <div>
          {{ formatSize(email.size) }}
        </div>
      </div>
    </div>

    <div class="grid grid-cols-[auto_1fr] gap-x-2">
      <template v-if="email.from">
        <div class="flex items-center text-gray-500 select-text">From:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.from" />
        </div>
      </template>

      <template v-if="email.to.length > 0">
        <div class="flex items-center text-gray-500 select-text">To:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.to.join(', ')" />
        </div>
      </template>

      <template v-if="email.headers?.Cc">
        <div class="flex items-center text-gray-500 select-text">CC:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.headers?.Cc?.join(', ') || ''" />
        </div>
      </template>

      <template v-if="email.headers?.Bcc">
        <div class="flex items-center text-gray-500 select-text">BCC:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.headers?.Bcc?.join(', ') || ''" />
        </div>
      </template>

      <template v-if="email.headers?.['Reply-To']">
        <div class="flex items-center text-gray-500 select-text">Reply-To:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.headers?.['Reply-To']?.join(', ') || ''" />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'
  import prettyBytes from 'pretty-bytes'

  defineProps<{ email: EmailRecord }>()

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }

  const formatFullDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
    })
  }
</script>
