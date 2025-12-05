<template>
  <div>
    <div class="flex items-center mb-3">
      <button class="btn" @click="copyToClipboard(content)">
        <ClipboardIcon class="h-4 w-4" />
        Copy
      </button>
    </div>
    <div class="p-4 rounded-sm overflow-auto">
      <pre
        class="text-xs font-mono whitespace-pre-wrap"
        v-html="highlightedContent"
      ></pre>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import { ClipboardIcon } from '@heroicons/vue/24/outline'
  import { copyToClipboard } from '@/helpers/copy'
  import hljs from 'highlight.js/lib/core'
  import xml from 'highlight.js/lib/languages/xml'
  import plaintext from 'highlight.js/lib/languages/plaintext'
  import 'highlight.js/styles/github.css'

  hljs.registerLanguage('xml', xml)
  hljs.registerLanguage('plaintext', plaintext)
  interface Props {
    content: string
    language?: string
  }

  const props = withDefaults(defineProps<Props>(), {
    language: 'plaintext',
  })

  const highlightedContent = computed(() => {
    if (!props.content) return ''

    try {
      const result = hljs.highlight(props.content, {
        language: props.language || 'plaintext',
        ignoreIllegals: true,
      })
      return result.value
    } catch {
      return hljs.highlight(props.content, {
        language: 'plaintext',
        ignoreIllegals: true,
      }).value
    }
  })
</script>
