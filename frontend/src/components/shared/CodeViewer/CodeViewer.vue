<template>
  <div>
    <div class="flex items-center mb-3">
      <button class="btn" @click="copyToClipboard(content)">
        <ClipboardIcon class="h-4 w-4" />
        Copy
      </button>
    </div>
    <div class="bg-gray-900 text-gray-100 p-4 rounded-sm overflow-auto">
      <pre class="text-xs font-mono whitespace-pre-wrap wrap-break-word">{{
        content
      }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ClipboardIcon } from '@heroicons/vue/24/outline'
  import { copyToClipboard } from '@/helpers/copy'

  interface Props {
    content: string
  }

  defineProps<Props>()
</script>

<style scoped>
  .wrap-break-word {
    word-break: break-word;
    overflow-wrap: break-word;
  }
</style>
