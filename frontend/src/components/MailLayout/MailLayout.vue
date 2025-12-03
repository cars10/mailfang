<template>
  <div class="flex flex-row h-full">
    <MailSidebar :counts="counts" />

    <MailList
      :emails="emails"
      :loading="loading"
      :error="error"
      :has-next-page="hasNextPage"
      :loading-more="loadingMore"
      @load-more="loadMore"
    />

    <div class="grow">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, watch } from 'vue'
  import { useRoute } from 'vue-router'
  import { useWebSocket } from '@/composables/useWebSocket'
  import { apiClient } from '@/api/client'
  import { useSearchStore } from '@/stores/Search'
  import type { EmailListRecord, EmailCounts } from '@/types/email'
  import MailSidebar from './MailSidebar.vue'
  import MailList from './MailList.vue'

  const route = useRoute()
  const searchStore = useSearchStore()
  const emails = ref<EmailListRecord[]>([])
  const counts = ref<EmailCounts>({
    inbox: 0,
    unread: 0,
    with_attachments: 0,
    archive: 0,
  })
  const loading = ref(false)
  const loadingMore = ref(false)
  const error = ref<string | null>(null)
  const currentPage = ref(1)
  const hasNextPage = ref(false)
  let searchTimeout: ReturnType<typeof setTimeout> | null = null

  const getApiMethod = (path: string, page: number = 1, search?: string) => {
    if (path.startsWith('/mails/inbox')) {
      return apiClient.inbox(page, search)
    } else if (path.startsWith('/mails/unread')) {
      return apiClient.unread(page, search)
    } else if (path.startsWith('/mails/with-attachments')) {
      return apiClient.withAttachments(page, search)
    } else if (path.startsWith('/mails/archive')) {
      return apiClient.archived(page, search)
    }
    return apiClient.inbox(page, search)
  }

  const shouldShowEmail = (email: EmailListRecord, path: string): boolean => {
    if (path.startsWith('/mails/inbox')) {
      return true
    } else if (path.startsWith('/mails/unread')) {
      return !email.read
    } else if (path.startsWith('/mails/with-attachments')) {
      return email.has_attachments
    } else if (path.startsWith('/mails/archive')) {
      return true
    }
    return true
  }

  const fetchMails = async (reset: boolean = true) => {
    try {
      if (reset) {
        loading.value = true
        currentPage.value = 1
        emails.value = []
      } else {
        loadingMore.value = true
      }
      error.value = null
      const search = searchStore.query.trim() || undefined
      const response = await getApiMethod(route.path, currentPage.value, search)
      if (reset) {
        emails.value = response.emails
      } else {
        emails.value = [...emails.value, ...response.emails]
      }
      counts.value = response.counts
      currentPage.value = response.pagination.page
      hasNextPage.value =
        response.pagination.page < response.pagination.total_pages
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : 'Failed to fetch emails'
    } finally {
      loading.value = false
      loadingMore.value = false
    }
  }

  const handleNewEmail = (email: EmailListRecord, newCounts?: EmailCounts) => {
    if (newCounts) {
      counts.value = newCounts
    }

    if (searchStore.query.trim()) {
      return
    }

    if (!shouldShowEmail(email, route.path)) {
      return
    }

    const existingIds = new Set(emails.value.map(e => e.id))
    if (existingIds.has(email.id)) {
      return
    }

    emails.value = [email, ...emails.value]
  }

  const handleEmailUpdated = (
    email: EmailListRecord | null,
    newCounts?: EmailCounts
  ) => {
    if (newCounts) {
      counts.value = newCounts
    }

    if (!email) {
      return
    }

    if (searchStore.query.trim()) {
      return
    }

    const index = emails.value.findIndex(e => e.id === email.id)
    if (index !== -1) {
      if (shouldShowEmail(email, route.path)) {
        emails.value[index] = email
      } else {
        emails.value.splice(index, 1)
      }
    } else if (shouldShowEmail(email, route.path)) {
      emails.value = [email, ...emails.value]
    }
  }

  const loadMore = async () => {
    if (!loadingMore.value && hasNextPage.value) {
      currentPage.value += 1
      await fetchMails(false)
    }
  }

  useWebSocket(
    {
      onMessage: event => {
        try {
          const message = JSON.parse(event.data)
          if (message.event === 'new_email') {
            if (message.email) {
              handleNewEmail(message.email, message.counts)
            } else {
              if (message.counts) {
                counts.value = message.counts
              }
              fetchMails()
            }
          } else if (message.event === 'email_updated') {
            handleEmailUpdated(message.email, message.counts)
          }
        } catch {
          if (event.data === 'new_email' || event.data === 'email_updated') {
            fetchMails()
          }
        }
      },
    },
    {
      autoConnect: true,
    }
  )

  onMounted(() => {
    fetchMails()
  })

  watch(
    () => searchStore.query,
    () => {
      if (searchTimeout) {
        clearTimeout(searchTimeout)
      }
      searchTimeout = setTimeout(() => {
        fetchMails()
      }, 300)
    }
  )

  watch(
    () => route.path,
    newPath => {
      if (
        !newPath.match(
          /\/mails\/(inbox|unread|with-attachments|archive)\/[^/]+$/
        )
      ) {
        searchStore.query = ''
        fetchMails()
      }
    }
  )
</script>
