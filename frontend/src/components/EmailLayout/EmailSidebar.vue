<template>
  <div class="bg-gray-100 border-r border-gray-300 shrink-0">
    <vue-resizable
      :active="['r']"
      :min-width="64"
      :max-width="600"
      :width="mailLayoutStore.sidebarWidth || DEFAULT_SIDEBAR_WIDTH"
      @dblclick="handleDoubleClick"
      @resize:move="handleResizeEnd"
    >
      <div class="p-4 h-full overflow-y-auto">
        <h1 class="text-2xl font-bold mb-4 text-center">{{ headerText }}</h1>

        <div class="flex flex-col gap-2">
          <router-link
            :to="'/emails/inbox'"
            title="All emails"
            active-class="text-primary bg-gray-200"
            class="h-8 flex flex-row gap-4 items-center justify-between hover:bg-gray-200 px-2 py-1 rounded-sm text-[#222] overflow-hidden min-w-4"
          >
            <div class="flex flex-row gap-2 items-center min-w-4">
              <InboxIcon class="h-4 w-4 shrink-0" />
              <div v-if="!sidebarCollapsed" class="truncate">All emails</div>
            </div>
            <span class="text-sm text-gray-600 font-mono">
              {{ props.counts.inbox }}
            </span>
          </router-link>
        </div>

        <EmailSidebarInboxes :counts="props.counts" />

        <div class="flex flex-col gap-2 mt-4">
          <button
            class="btn flex flex-row items-center gap-2"
            :class="{ 'justify-center': sidebarCollapsed }"
            :disabled="loadingDeleteAll"
            @click="handleDeleteAll"
          >
            <TrashIcon class="h-4" />
            <div v-if="!sidebarCollapsed">Clear</div>
          </button>
        </div>
      </div>
    </vue-resizable>
  </div>
</template>

<script setup lang="ts">
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import { InboxIcon, TrashIcon } from '@heroicons/vue/24/outline'
  import type { EmailCounts } from '@/types/email'
  import { apiClient } from '@/api/client'
  import VueResizable from 'vue-resizable'
  import { DEFAULT_SIDEBAR_WIDTH } from '@/stores/MailLayout'
  import EmailSidebarInboxes from './EmailSidebarInboxes.vue'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const router = useRouter()
  const mailLayoutStore = useMailLayoutStore()
  const loadingDeleteAll = ref(false)

  const sidebarCollapsed = computed(() => {
    return mailLayoutStore.sidebarWidth < 140
  })

  const headerText = computed(() =>
    sidebarCollapsed.value ? 'MF' : 'MailFang'
  )

  const handleDoubleClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    if (target.classList.contains('resizable-r')) {
      mailLayoutStore.sidebarWidth = DEFAULT_SIDEBAR_WIDTH
    }
  }

  const handleResizeEnd = ({ width }: { width: number }) => {
    mailLayoutStore.sidebarWidth = width
  }

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
