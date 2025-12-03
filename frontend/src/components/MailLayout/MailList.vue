<template>
  <div class="bg-white border-r border-gray-300">
    <vue-resizable
      :active="['r']"
      :min-width="350"
      :max-width="800"
      :width="mailLayoutStore.inboxWidth || 350"
      @resize:end="handleResizeEnd"
    >
      <div class="p-4 border-b border-gray-300">
        <input
          v-model="searchStore.query"
          type="text"
          placeholder="Search"
          class="w-full p-2 rounded-md border border-gray-300"
          @keydown.escape="searchStore.query = ''"
        />
      </div>

      <div class="overflow-y-auto h-full">
        <div v-if="loading" class="flex items-center justify-center p-8">
          <div class="text-gray-500">Loading emails...</div>
        </div>

        <div v-else-if="error" class="flex items-center justify-center p-8">
          <div class="text-red-500">{{ error }}</div>
        </div>

        <div
          v-else-if="emails.length === 0"
          class="flex items-center justify-center p-8"
        >
          <div class="text-gray-500">No emails found</div>
        </div>

        <template v-else>
          <div
            v-for="mail in emails"
            :key="mail.id"
            :class="`border-b border-b-gray-300 border-l-4 ${route.params.id === mail.id ? 'border-l-indigo-500 bg-gray-50' : 'border-l-transparent'} p-4 gap-4 hover:cursor-pointer hover:bg-gray-100 w-full`"
            @click="openMail(mail.id)"
          >
            <div class="flex flex-col grow">
              <div class="flex flex-row justify-between gap-4 items-center">
                <div
                  :class="['line-clamp-2', { 'font-bold': !mail.read }]"
                  :title="mail.subject || '(No Subject)'"
                >
                  {{ mail.subject || '(No Subject)' }}
                </div>
                <div
                  class="flex items-center gap-1 text-sm text-gray-500 text-nowrap font-mono"
                >
                  <PaperClipIcon
                    v-if="mail.has_attachments"
                    class="h-3 text-gray-500"
                  />
                  {{ formatDate(mail.date || mail.created_at) }}
                </div>
              </div>
              <div class="grid grid-cols-[auto_1fr] gap-x-2">
                <div class="text-gray-500">From:</div>
                <div class="truncate" :title="mail.from">
                  {{ mail.from }}
                </div>
                <div class="text-gray-500">To:</div>
                <div class="truncate" :title="mail.to.join(', ')">
                  {{ mail.to.join(', ') }}
                </div>
              </div>
            </div>
          </div>

          <div
            v-if="hasNextPage"
            class="flex items-center justify-center p-4 border-t border-gray-300"
          >
            <button
              v-if="!loadingMore"
              class="text-indigo-600 hover:text-indigo-800 hover:underline"
              @click="$emit('load-more')"
            >
              Load more
            </button>
            <div v-else class="text-gray-500">Loading more...</div>
          </div>
        </template>
      </div>
    </vue-resizable>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, onUnmounted } from 'vue'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import { useSearchStore } from '@/stores/Search'
  import { PaperClipIcon } from '@heroicons/vue/24/outline'
  import VueResizable from 'vue-resizable'
  import { useRouter, useRoute } from 'vue-router'
  import type { EmailListRecord } from '@/types/email'

  interface Props {
    emails: EmailListRecord[]
    loading: boolean
    error: string | null
    hasNextPage: boolean
    loadingMore: boolean
  }

  defineProps<Props>()
  defineEmits<{
    'load-more': []
  }>()

  const searchStore = useSearchStore()

  const windowHeight = ref(window.innerHeight)

  const updateWindowHeight = () => {
    windowHeight.value = window.innerHeight
  }

  onMounted(() => {
    window.addEventListener('resize', updateWindowHeight)
    updateWindowHeight()
  })

  onUnmounted(() => {
    window.removeEventListener('resize', updateWindowHeight)
  })

  const formatDate = (dateString: string) => {
    const today = new Date()
    const mailDate = new Date(dateString)

    // Check if same day
    const isToday =
      mailDate.getDate() === today.getDate() &&
      mailDate.getMonth() === today.getMonth() &&
      mailDate.getFullYear() === today.getFullYear()

    if (isToday) {
      // Show time only (HH:MM format)
      return mailDate.toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      })
    } else {
      // Show date & month (e.g., "Jan 15")
      return mailDate.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
      })
    }
  }

  const route = useRoute()
  const router = useRouter()

  const mailLayoutStore = useMailLayoutStore()
  const handleResizeEnd = ({ width }: { width: number }) => {
    mailLayoutStore.inboxWidth = width
  }

  const openMail = (id: string) => {
    const currentPath = router.currentRoute.value.path
    // Extract the base path (e.g., /mails/inbox from /mails/inbox or /mails/inbox/1)
    const basePath = currentPath.split('/').slice(0, 3).join('/')
    router.push(`${basePath}/${id}`)
  }
</script>
