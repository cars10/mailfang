<template>
  <div class="p-4 bg-gray-100 border-r border-gray-300 shrink-0">
    <h1 class="text-2xl font-bold mb-4 text-center">{{ headerText }}</h1>

    <div class="flex flex-col gap-2">
      <router-link
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        :title="link.label"
        active-class="text-primary bg-gray-200"
        class="flex flex-row gap-4 items-center justify-between hover:bg-gray-200 px-2 py-1 rounded-sm text-[#222]"
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

    <div class="mt-2">
      <button
        class="flex w-full flex-row items-center justify-between px-2 py-1 rounded-sm text-[#222] hover:bg-gray-200 cursor-pointer"
        type="button"
        @click="
          mailLayoutStore.recipientsCollapsed =
            !mailLayoutStore.recipientsCollapsed
        "
      >
        <div class="flex flex-row items-center gap-2">
          <component
            :is="
              mailLayoutStore.recipientsCollapsed
                ? ChevronRightIcon
                : ChevronDownIcon
            "
            class="h-4"
          />
          <div v-if="!mailLayoutStore.sidebarCollapsed">Inboxes</div>
        </div>
      </button>

      <div
        v-if="
          !mailLayoutStore.recipientsCollapsed &&
          !mailLayoutStore.sidebarCollapsed
        "
        class="mt-1 flex flex-col gap-1 ml-4 pl-2 border-l border-gray-300"
      >
        <router-link
          v-for="recipient in props.counts.recipients"
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
    </div>

    <div class="flex flex-col gap-2 mt-4">
      <button class="btn" :disabled="loadingDeleteAll" @click="handleDeleteAll">
        <TrashIcon class="h-4" />
        <div v-if="!mailLayoutStore.sidebarCollapsed">Delete all emails</div>
      </button>
    </div>

    <div class="flex mt-4 justify-end">
      <button
        class="hover:cursor-pointer bg-gray-200 shadow-md hover:text-primary focus:outline-primary p-1 rounded-xl z-10"
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
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import {
    InboxIcon,
    ChevronDoubleLeftIcon,
    ChevronDoubleRightIcon,
    TrashIcon,
    ChevronDownIcon,
    ChevronRightIcon,
  } from '@heroicons/vue/24/outline'
  import type { EmailCounts } from '@/types/email'
  import { apiClient } from '@/api/client'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const router = useRouter()
  const mailLayoutStore = useMailLayoutStore()
  const loadingDeleteAll = ref(false)

  const headerText = computed(() =>
    mailLayoutStore.sidebarCollapsed ? 'MF' : 'MailFang'
  )
  const links = computed(() => [
    {
      to: '/emails/inbox',
      icon: InboxIcon,
      label: 'All emails',
      count: props.counts.inbox,
    },
  ])

  const handleDeleteAll = async () => {
    if (loadingDeleteAll.value) return

    if (
      !confirm(
        'Are you sure you want to delete ALL emails? This action cannot be undone.'
      )
    ) {
      return
    }

    try {
      loadingDeleteAll.value = true
      await apiClient.deleteAll()
      router.push('/emails/inbox')
      window.location.reload()
    } catch (err) {
      console.error('Failed to delete all emails:', err)
    } finally {
      loadingDeleteAll.value = false
    }
  }
</script>
