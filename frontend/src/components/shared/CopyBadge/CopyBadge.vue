<template>
  <div
    title="Click to copy"
    class="group hover:bg-gray-100 py-1 px-2 rounded-xl cursor-pointer flex items-center gap-1 grow-0"
    @click="handleCopy"
  >
    {{ text }}
    <ClipboardIcon
      v-if="!copied"
      class="h-4 w-4 min-w-4 opacity-0 group-hover:opacity-100 transition-opacity text-indigo-700"
    />
    <CheckIcon
      v-else
      class="h-4 w-4 min-w-4 text-green-600 transition-opacity"
    />
  </div>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
  import { copyToClipboard } from '@/helpers/copy'
  import { ClipboardIcon, CheckIcon } from '@heroicons/vue/24/outline'

  const props = defineProps<{
    text: string
  }>()

  const copied = ref(false)

  const handleCopy = async () => {
    await copyToClipboard(props.text)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  }
</script>
