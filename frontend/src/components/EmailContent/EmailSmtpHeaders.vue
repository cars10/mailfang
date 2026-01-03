<template>
  <div class="p-4 grow">
    <table>
      <tbody>
        <template v-for="(values, key) in email.headers" :key="key">
          <tr>
            <td class="text-gray-700 select-text px-2 text-nowrap">
              {{ key }}:
            </td>
            <td class="text-gray-600 select-text w-fit break-all px-2">
              <CopyBadge :text="values.join(', ')" />
            </td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'

  defineProps<{ email: EmailRecord }>()
</script>
