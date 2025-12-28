<template>
  <div class="bg-white border-r border-gray-300">
    <vue-resizable
      :active="['r']"
      :min-width="350"
      :max-width="800"
      :width="mailLayoutStore.inboxWidth || DEFAULT_INBOX_WIDTH"
      class="flex flex-col"
      @dblclick="handleDoubleClick"
      @resize:end="handleResizeEnd"
    >
      <div class="p-4 border-b border-gray-300">
        <TextInput
          v-model="searchStore.query"
          placeholder="Search"
          :icon="MagnifyingGlassIcon"
        />
      </div>

      <div class="overflow-y-auto">
        <div
          v-if="loading && emails.length === 0"
          class="flex items-center justify-center p-8"
        >
          <Spinner size="6" />
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
            :class="`border-b border-b-gray-300 border-l-4 ${route.params.id === mail.id ? 'border-l-primary bg-gray-50' : 'border-l-transparent'} p-4 gap-4 hover:cursor-pointer hover:bg-gray-100 w-full`"
            @click="openMail(mail.id)"
          >
            <div class="flex flex-col grow">
              <div class="flex flex-row justify-between gap-4 items-start">
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
                    class="h-4 text-gray-500"
                  />
                  {{ formatDate(mail.created_at) }}
                </div>
              </div>
              <div class="flex flex-row gap-2">
                <div class="text-gray-500">To:</div>
                <div class="truncate" :title="mail.to.join(', ')">
                  {{ mail.to.join(', ') }}
                </div>
              </div>
            </div>
          </div>
          <div
            v-if="emails.length > 0"
            ref="loadMoreSentinel"
            class="flex items-center justify-center p-6"
          >
            <Spinner v-if="loading" size="6" />
          </div>
        </template>
      </div>
    </vue-resizable>
  </div>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
  import { useIntersectionObserver } from '@vueuse/core'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import { useSearchStore } from '@/stores/Search'
  import { PaperClipIcon, MagnifyingGlassIcon } from '@heroicons/vue/24/outline'
  import VueResizable from 'vue-resizable'
  import { useRouter, useRoute } from 'vue-router'
  import type { EmailListRecord } from '@/types/email'
  import TextInput from '@/components/shared/TextInput/TextInput.vue'
  import Spinner from '@/components/shared/Spinner/Spinner.vue'
  import { DEFAULT_INBOX_WIDTH } from '@/stores/MailLayout'

  interface Props {
    emails: EmailListRecord[]
    loading: boolean
  }

  defineProps<Props>()
  const emit = defineEmits<{ 'load-more': [] }>()

  const searchStore = useSearchStore()

  const formatDate = (dateString: string) => {
    const today = new Date()
    const mailDate = new Date(dateString)

    const isToday =
      mailDate.getDate() === today.getDate() &&
      mailDate.getMonth() === today.getMonth() &&
      mailDate.getFullYear() === today.getFullYear()

    if (isToday) {
      return mailDate.toLocaleTimeString()
    } else {
      return mailDate.toLocaleDateString()
    }
  }

  const route = useRoute()
  const router = useRouter()

  const mailLayoutStore = useMailLayoutStore()
  const handleDoubleClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    if (target.classList.contains('resizable-r')) {
      mailLayoutStore.inboxWidth = DEFAULT_INBOX_WIDTH
    }
  }

  const handleResizeEnd = ({ width }: { width: number }) => {
    mailLayoutStore.inboxWidth = width
  }

  const openMail = (id: string) => {
    const recipient = route.params.recipient as string | undefined
    if (recipient) {
      router.push(`/emails/inbox/${encodeURIComponent(recipient)}/email/${id}`)
    } else {
      router.push(`/emails/inbox/email/${id}`)
    }
  }

  const loadMoreSentinel = ref<HTMLElement | null>(null)

  useIntersectionObserver(
    loadMoreSentinel,
    ([entry]: IntersectionObserverEntry[]) => {
      if (!entry?.isIntersecting) return

      emit('load-more')
    },
    { root: loadMoreSentinel.value?.parentElement, threshold: 0.5 }
  )
</script>
