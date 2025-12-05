<template>
  <div class="p-4 grow">
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
</template>

<script setup lang="ts">
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'

  defineProps<{ email: EmailRecord }>()
</script>
