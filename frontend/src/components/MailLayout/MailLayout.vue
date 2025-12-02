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
      hasNextPage.value = response.pagination.page < response.pagination.total_pages
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : 'Failed to fetch emails'
    } finally {
      loading.value = false
      loadingMore.value = false
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
        if (event.data === 'new_email' || event.data === 'email_updated') {
          fetchMails()
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
