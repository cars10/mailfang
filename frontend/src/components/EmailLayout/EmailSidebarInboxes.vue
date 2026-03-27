<template>
  <div class="mt-2">
    <div class="flex flex-row items-center justify-between gap-2 h-6 mt-6 mb-2">
      <div v-if="!searchFocused" class="truncate min-w-0 mx-1 py-2">
        Inboxes
      </div>
      <button
        v-if="!searchFocused && !sidebarCollapsed"
        class="btn btn--icon btn--small"
        @click="focusSearchInput"
      >
        <MagnifyingGlassIcon class="h-4 w-4" />
      </button>
      <TextInput
        v-if="searchFocused"
        ref="searchInput"
        v-model="searchTerm"
        class="w-full"
        dense
        placeholder="Search..."
        @blur="searchFocused = false"
        @keydown.escape="searchTerm = ''"
      />
    </div>

    <div class="flex flex-col gap-1 overflow-y-auto max-h-full p-1">
      <router-link
        v-for="recipient in filteredRecipients"
        :key="recipient.recipient"
        :to="`/emails/inbox/${encodeURIComponent(recipient.recipient)}`"
        :title="recipient.recipient"
        active-class="text-primary bg-app-gray-200"
        class="flex flex-row gap-1 items-center justify-between hover:bg-app-gray-200 px-2 py-1 rounded-sm text-sm"
      >
        <span class="truncate">{{ recipient.recipient }}</span>
        <span class="text-xs text-app-gray-600 font-mono">
          {{ recipient.count }}
        </span>
      </router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import { MagnifyingGlassIcon } from '@heroicons/vue/24/outline'
  import TextInput from '@/components/shared/TextInput/TextInput.vue'
  import type { EmailCounts } from '@/types/email'
  import { computed, nextTick, ref } from 'vue'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const mailLayoutStore = useMailLayoutStore()
  const searchFocused = ref(false)

  const searchTerm = ref('')
  const searchInput = ref<InstanceType<typeof TextInput> | null>(null)

  const focusSearchInput = () => {
    searchFocused.value = true
    nextTick(() => {
      searchInput.value?.focus()
    })
  }

  const sidebarCollapsed = computed(() => {
    return mailLayoutStore.sidebarWidth < 140
  })

  const filteredRecipients = computed(() => {
    if (!searchTerm.value.trim()) {
      return props.counts.recipients
    }
    const term = searchTerm.value.toLowerCase()
    return props.counts.recipients.filter(recipient =>
      recipient.recipient.toLowerCase().includes(term)
    )
  })
</script>
