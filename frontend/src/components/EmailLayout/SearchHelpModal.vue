<template>
  <Modal v-model:is-open="modalOpen" title="Search Help">
    <div class="space-y-6">
      <div>
        <h3 class="text-lg font-semibold text-gray-900 mb-3">
          Default Search
        </h3>
        <p class="text-gray-600 mb-2">
          The following fields are searched by default:
        </p>
        <ul class="list-disc list-inside space-y-1 text-gray-700 ml-4">
          <li>Email subject</li>
          <li>Email from address</li>
          <li>Email body text</li>
          <li>Recipient email addresses</li>
          <li>Header values (From, To, Cc, Bcc)</li>
        </ul>
      </div>

      <div>
        <h3 class="text-lg font-semibold text-gray-900 mb-3">
          Field-Specific Filters
        </h3>
        <p class="text-gray-600 mb-3">
          Use <code class="bg-gray-100 px-1 rounded text-sm">field:value</code> to search specific fields:
        </p>
        <ul class="list-disc list-inside space-y-1.5 text-gray-700 ml-4">
          <li><code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">subject:</code> - Email subject</li>
          <li><code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">from:</code> - Sender email and From headers</li>
          <li><code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">to:</code> or <code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">recipient:</code> - Recipient emails and To/Cc/Bcc headers</li>
          <li><code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">text:</code> - Email body text</li>
          <li><code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">html:</code> - Email HTML body</li>
          <li><code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">attachment:</code> - Attachment filenames</li>
        </ul>
      </div>

      <div>
        <h3 class="text-lg font-semibold text-gray-900 mb-3">Examples</h3>
        <ul class="space-y-2.5 text-gray-700">
          <li>
            <code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">paypal</code>
            <span class="ml-2">Searches all default fields</span>
          </li>
          <li>
            <code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">subject:invoice payment</code>
            <span class="ml-2">Subject contains "invoice" AND any field contains "payment"</span>
          </li>
          <li>
            <code class="bg-gray-100 px-1.5 py-0.5 rounded text-sm font-mono">from:paypal recipient:john</code>
            <span class="ml-2">From contains "paypal" AND recipient contains "john"</span>
          </li>
        </ul>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import Modal from '@/components/shared/Modal/Modal.vue'

  const props = defineProps<{
    isOpen: boolean
  }>()

  const emit = defineEmits<{ 'update:isOpen': [value: boolean] }>()

  const modalOpen = computed({
    get: () => props.isOpen,
    set: (value: boolean) => emit('update:isOpen', value),
  })
</script>
