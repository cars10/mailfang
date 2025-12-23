<template>
  <div class="mt-2">
    <div
      class="flex w-full flex-row items-center justify-between gap-2 rounded-sm h-6"
    >
      <button
        class="flex flex-row shrink grow items-center gap-2 hover:bg-gray-200 rounded-sm px-2 py-1 cursor-pointer overflow-hidden min-w-4"
        type="button"
        @click="
          mailLayoutStore.recipientsCollapsed =
            !mailLayoutStore.recipientsCollapsed
        "
      >
        <ChevronRightIcon
          v-if="mailLayoutStore.recipientsCollapsed"
          class="h-4 shrink-0"
        />
        <ChevronDownIcon v-else class="h-4 shrink-0" />

        <div v-if="!searchFocused" class="truncate min-w-0">Inboxes</div>
      </button>
      <div v-if="!sidebarCollapsed" class="ml-auto">
        <TextInput
          v-model="searchTerm"
          placeholder="Search inboxes..."
          :icon="MagnifyingGlassIcon"
          :expandable="true"
          expanded-width="100%"
          @focus="searchFocused = true"
          @blur="searchFocused = false"
          @keydown.escape="searchTerm = ''"
        />
      </div>
    </div>

    <vue-resizable
      v-if="!mailLayoutStore.recipientsCollapsed"
      :active="['b']"
      :min-height="20"
      :height="mailLayoutStore.recipientsHeight"
      :style="{ width: '100%' }"
      class="mt-1 pb-1 overflow-hidden border-b border-gray-300 w-full"
      @dblclick="handleDoubleClick"
      @resize:end="handleRecipientsResizeEnd"
    >
      <div class="flex flex-col gap-1 overflow-y-auto ml-1">
        <router-link
          v-for="recipient in filteredRecipients"
          :key="recipient.recipient"
          :to="`/emails/inbox/${encodeURIComponent(recipient.recipient)}`"
          :title="recipient.recipient"
          active-class="text-primary bg-gray-200"
          class="flex flex-row gap-1 items-center justify-between hover:bg-gray-200 px-2 py-1 rounded-sm text-sm text-[#222]"
        >
          <span class="truncate">{{ recipient.recipient }}</span>
          <span class="text-xs text-gray-600 font-mono">
            {{ recipient.count }}
          </span>
        </router-link>
      </div>
    </vue-resizable>
  </div>
</template>

<script setup lang="ts">
  import {
    DEFAULT_RECIPIENTS_HEIGHT,
    useMailLayoutStore,
  } from '@/stores/MailLayout'
  import {
    ChevronRightIcon,
    ChevronDownIcon,
    MagnifyingGlassIcon,
  } from '@heroicons/vue/24/outline'
  import TextInput from '@/components/shared/TextInput/TextInput.vue'
  import VueResizable from 'vue-resizable'
  import type { EmailCounts } from '@/types/email'
  import { computed, ref } from 'vue'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const mailLayoutStore = useMailLayoutStore()
  const searchFocused = ref(false)

  const searchTerm = ref('')

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

  const handleRecipientsResizeEnd = ({ height }: { height: number }) => {
    mailLayoutStore.recipientsHeight = height
  }

  const handleDoubleClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    if (target.classList.contains('resizable-b')) {
      mailLayoutStore.recipientsHeight = DEFAULT_RECIPIENTS_HEIGHT
    }
  }
</script>
