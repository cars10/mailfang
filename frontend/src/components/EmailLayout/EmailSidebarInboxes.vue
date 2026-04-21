<template>
  <div class="pt-2 flex flex-col flex-1">
    <div class="flex flex-row items-center justify-between gap-2 h-6 mt-6 mb-2">
      <div
        v-if="!searchFocused"
        class="flex items-center gap-2 min-w-0 mx-1 py-2 flex-1"
      >
        <span class="shrink-0">Inboxes</span>
        <div
          v-if="searchTerm && !sidebarCollapsed"
          class="min-w-0 flex-1 flex justify-end"
        >
          <Badge
            class="min-w-0 max-w-full"
            :text="searchTerm"
            :title="`Clear filter: ${searchTerm}`"
            @click="clearSearch"
          >
            <template #trailing>
              <XMarkIcon class="h-3 w-3" />
            </template>
          </Badge>
        </div>
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
        @blur="commitSearch"
        @keydown.enter="commitSearch"
        @keydown.escape="searchTerm = ''"
      />
    </div>

    <VirtualList
      class="p-1"
      :items="filteredRecipients"
      :row-height="30"
      :item-key="itemKey"
    >
      <template #default="{ item }">
        <router-link
          :to="`/emails/inbox/${encodeURIComponent(item.recipient)}`"
          :title="item.recipient"
          active-class="text-primary bg-app-gray-200"
          class="flex flex-row gap-1 items-center justify-between hover:bg-app-gray-200 px-2 py-1 rounded-sm text-sm h-[26px] mb-1"
        >
          <span class="truncate">{{ item.recipient }}</span>
          <span class="text-xs text-app-gray-600 font-mono">
            {{ item.count }}
          </span>
        </router-link>
      </template>
      <template #empty>
        <div v-if="searchTerm" class="px-2 py-1 text-xs text-app-gray-600">
          No inboxes found.
        </div>
      </template>
    </VirtualList>
  </div>
</template>

<script setup lang="ts">
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import { MagnifyingGlassIcon, XMarkIcon } from '@heroicons/vue/24/outline'
  import TextInput from '@/components/shared/TextInput/TextInput.vue'
  import Badge from '@/components/shared/Badge/Badge.vue'
  import VirtualList from '@/components/shared/VirtualList/VirtualList.vue'
  import type { EmailCounts } from '@/types/email'
  import { debounce } from '@/utils/debounce'
  import { computed, nextTick, ref, watch } from 'vue'

  interface Props {
    counts: EmailCounts
  }
  type Recipient = EmailCounts['recipients'][number]

  const props = defineProps<Props>()

  const mailLayoutStore = useMailLayoutStore()
  const searchFocused = ref(false)

  const searchTerm = ref('')
  const filteredRecipients = ref<Recipient[]>([])
  const searchInput = ref<InstanceType<typeof TextInput> | null>(null)

  const updateFilteredRecipients = () => {
    const term = searchTerm.value.trim().toLowerCase()
    filteredRecipients.value = term
      ? props.counts.recipients.filter(recipient =>
          recipient.recipient.toLowerCase().includes(term)
        )
      : props.counts.recipients
  }

  const updateFilteredRecipientsDebounced = debounce(
    updateFilteredRecipients,
    50
  )

  watch(
    [searchTerm, () => props.counts.recipients],
    () => {
      updateFilteredRecipientsDebounced()
    },
    { immediate: true }
  )

  const focusSearchInput = () => {
    searchFocused.value = true
    nextTick(() => {
      searchInput.value?.focus()
    })
  }

  const commitSearch = () => {
    searchTerm.value = searchTerm.value.trim()
    searchFocused.value = false
  }

  const clearSearch = () => {
    searchTerm.value = ''
    updateFilteredRecipients()
  }

  const itemKey = (item: Recipient) => item.recipient

  const sidebarCollapsed = computed(() => {
    return mailLayoutStore.sidebarWidth < 140
  })
</script>
