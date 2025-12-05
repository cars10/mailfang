<template>
  <div
    title="Click to copy"
    class="group hover:bg-gray-100 py-1 px-2 rounded-xl cursor-pointer flex items-center gap-1 grow-0"
    @click="handleCopy"
  >
    {{ text }}
    <ClipboardIcon
      v-if="!copied"
      class="h-4 w-4 min-w-4 opacity-0 group-hover:opacity-100 transition-opacity text-primary"
    />
    <CheckIcon
      v-else
      class="h-4 w-4 min-w-4 text-green-600 transition-opacity"
    />
  </div>
</template>

<script setup lang="ts">
  import { ClipboardIcon, CheckIcon } from '@heroicons/vue/24/outline'
  import { useCopy } from '@/composables/useCopy'

  const props = defineProps<{
    text: string
  }>()

  const { copied, handleCopy } = useCopy(() => props.text)
</script>
