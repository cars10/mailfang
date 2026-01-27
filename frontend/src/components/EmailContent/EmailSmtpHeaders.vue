<template>
  <div class="p-4 grow">
    <table>
      <tbody>
        <template v-for="[key, values] in sortedHeaders" :key="key">
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
  import { computed } from 'vue'
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'

  const props = defineProps<{ email: EmailRecord }>()

  const sortedHeaders = computed(() => {
    return Object.entries(props.email.headers).sort(([a], [b]) =>
      a.toLowerCase().localeCompare(b.toLowerCase())
    )
  })
</script>
