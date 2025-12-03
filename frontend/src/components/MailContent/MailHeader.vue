<template>
  <div class="border-b border-gray-200 bg-gray-50 px-6 py-4">
    <div class="flex items-start justify-between gap-2 mb-4">
      <h1 class="text-2xl text-gray-900">
        {{ email.subject || '(No Subject)' }}
      </h1>
      <div class="text-gray-600 text-nowrap font-mono">
        {{ formatFullDate(email.date || email.created_at) }}
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

    <div class="mt-4">
      <button
        class="flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900 transition-colors cursor-pointer"
        @click="showHeaders = !showHeaders"
      >
        <ChevronDownIcon
          :class="['h-4 w-4 transition-transform', showHeaders && 'rotate-180']"
        />
        <span>{{ showHeaders ? 'Hide' : 'Show' }} all headers</span>
      </button>

      <div
        v-if="showHeaders && email.headers"
        class="mt-2 p-3 bg-white border border-gray-200 rounded-lg text-sm"
      >
        <div class="grid grid-cols-[auto_1fr] gap-x-2">
          <template v-for="(values, key) in email.headers" :key="key">
            <div class="flex items-center text-gray-700 select-text">
              {{ key }}:
            </div>
            <div class="text-gray-600 select-text w-fit">
              <CopyBadge
                v-if="values && values.length > 0"
                :text="values.join(', ')"
              />
              <span v-else class="text-gray-400">—</span>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'
  import { ChevronDownIcon } from '@heroicons/vue/24/outline'

  defineProps<{ email: EmailRecord }>()

  const showHeaders = ref(false)

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
