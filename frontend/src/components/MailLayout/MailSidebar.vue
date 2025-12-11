<template>
  <div class="p-4 bg-gray-100 border-r border-gray-300 shrink-0">
    <div class="flex flex-col gap-2">
      <router-link
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        :title="link.label"
        active-class="text-primary bg-gray-200"
        class="flex flex-row gap-4 items-center justify-between hover:bg-gray-200 px-2 py-1 rounded-sm text-[#222] h-[40px]"
      >
        <div class="flex flex-row gap-2 items-center">
          <component :is="link.icon" :class="['h-4']" />
          <div v-if="!mailLayoutStore.sidebarCollapsed">
            {{ link.label }}
          </div>
        </div>
        <span
          v-if="link.count !== undefined"
          class="text-sm text-gray-600 font-mono"
        >
          {{ link.count }}
        </span>
      </router-link>
    </div>

    <div
      :class="`flex mt-4 ${mailLayoutStore.sidebarCollapsed ? 'justify-center' : 'justify-end'}`"
    >
      <button
        class="hover:cursor-pointer bg-gray-200 shadow-md hover:text-primary p-1 rounded-xl z-10"
        @click="
          mailLayoutStore.sidebarCollapsed = !mailLayoutStore.sidebarCollapsed
        "
      >
        <ChevronDoubleRightIcon
          v-if="mailLayoutStore.sidebarCollapsed"
          class="h-4"
        />
        <ChevronDoubleLeftIcon v-else class="h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import {
    InboxIcon,
    EnvelopeIcon,
    PaperClipIcon,
    ChevronDoubleLeftIcon,
    ChevronDoubleRightIcon,
  } from '@heroicons/vue/24/outline'
  import type { EmailCounts } from '@/types/email'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const mailLayoutStore = useMailLayoutStore()

  const links = computed(() => [
    {
      to: '/mails/inbox',
      icon: InboxIcon,
      label: 'Inbox',
      count: props.counts.inbox,
    },
    {
      to: '/mails/unread',
      icon: EnvelopeIcon,
      label: 'Unread',
      count: props.counts.unread,
    },
    {
      to: '/mails/with-attachments',
      icon: PaperClipIcon,
      label: 'With attachments',
      count: props.counts.with_attachments,
    },
  ])
</script>
