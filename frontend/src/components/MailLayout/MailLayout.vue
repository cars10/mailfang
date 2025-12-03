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
    // Note: EmailListRecord doesn't have archived field, so we assume non-archived for inbox/unread/with-attachments
    // and archived emails won't appear in those views anyway
    if (path.startsWith('/mails/inbox')) {
      return true // All non-archived emails show in inbox
    } else if (path.startsWith('/mails/unread')) {
      return !email.read
    } else if (path.startsWith('/mails/with-attachments')) {
      return email.has_attachments
    } else if (path.startsWith('/mails/archive')) {
      return true // All archived emails show in archive
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
    // Update counts if provided
    if (newCounts) {
      counts.value = newCounts
    }

    // Don't add if we have a search query active
    if (searchStore.query.trim()) {
      return
    }

    // Check if email should be shown in current view
    if (!shouldShowEmail(email, route.path)) {
      // Email doesn't match current view, but counts are already updated
      return
    }

    // Check for duplicates
    const existingIds = new Set(emails.value.map(e => e.id))
    if (existingIds.has(email.id)) {
      return
    }

    // Prepend the new email to the list
    emails.value = [email, ...emails.value]
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
              // New email with data - prepend it if it matches current view
              handleNewEmail(message.email, message.counts)
            } else {
              // Email updated but no email data, reload to reflect changes
              if (message.counts) {
                counts.value = message.counts
              }
              fetchMails()
            }
          }
        } catch {
          // If parsing fails, treat as old format string message
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
      // Only refetch if we're on a different mail list route (not when opening a specific mail)
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
