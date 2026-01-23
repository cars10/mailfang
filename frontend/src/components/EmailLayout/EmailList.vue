<template>
  <div class="shadow-lg z-20 border-r border-gray-200">
    <vue-resizable
      :active="['r']"
      :min-width="350"
      :max-width="800"
      :width="mailLayoutStore.inboxWidth || DEFAULT_INBOX_WIDTH"
      class="flex flex-col"
      @dblclick="handleDoubleClick"
      @resize:end="handleResizeEnd"
    >
      <div class="p-4">
        <div class="relative">
          <TextInput
            v-model="searchStore.query"
            placeholder="Search"
            :icon="MagnifyingGlassIcon"
          />
          <button
            type="button"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-500 hover:text-gray-700 focus:outline-none underline cursor-pointer"
            @click="showHelpModal = true"
          >
            help
          </button>
        </div>
      </div>

      <div
        ref="emailListContainer"
        tabindex="0"
        class="overflow-y-auto px-4 flex flex-col gap-2 focus:outline-none"
        @keydown="handleKeyDown"
      >
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
            :class="`border-l-4 ${route.params.id === mail.id ? 'border-l-primary bg-gray-50' : 'border-l-transparent'} p-2 gap-4 hover:cursor-pointer hover:bg-gray-100 w-full rounded-sm`"
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
                  class="flex items-center gap-1 text-sm text-gray-500 text-nowrap"
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
                <div class="truncate">
                  {{
                    parseAndDecodeHeaderValues(mail.to_header || []).join(', ')
                  }}
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

    <SearchHelpModal v-model:is-open="showHelpModal" />
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, nextTick, watch } from 'vue'
  import { useIntersectionObserver } from '@vueuse/core'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import { useSearchStore } from '@/stores/Search'
  import { PaperClipIcon, MagnifyingGlassIcon } from '@heroicons/vue/24/outline'
  import VueResizable from 'vue-resizable'
  import { useRouter, useRoute } from 'vue-router'
  import type { EmailListRecord } from '@/types/email'
  import TextInput from '@/components/shared/TextInput/TextInput.vue'
  import Spinner from '@/components/shared/Spinner/Spinner.vue'
  import SearchHelpModal from './SearchHelpModal.vue'
  import { DEFAULT_INBOX_WIDTH } from '@/stores/MailLayout'
  import { parseAndDecodeHeaderValues } from '@/utils/emailAddress'

  interface Props {
    emails: EmailListRecord[]
    loading: boolean
  }

  const props = defineProps<Props>()
  const emit = defineEmits<{ 'load-more': [] }>()

  const searchStore = useSearchStore()
  const showHelpModal = ref(false)

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

  const openMailByIndex = (index: number) => {
    if (index < 0 || index >= props.emails.length) return
    openMail(props.emails[index]!.id)
    nextTick(() => emailListContainer.value?.focus())
  }

  const emailListContainer = ref<HTMLElement | null>(null)
  const currentEmailIndex = computed(() => {
    const currentId = route.params.id as string | undefined
    if (!currentId) return -1
    return props.emails.findIndex(email => email.id === currentId)
  })

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      openMailByIndex(currentEmailIndex.value + 1)
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      openMailByIndex(currentEmailIndex.value - 1)
    }
  }

  watch(
    () => route.params.id,
    () => nextTick(() => emailListContainer.value?.focus())
  )

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
