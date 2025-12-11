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
  })
  const loading = ref(false)
  const loadingMore = ref(false)
  const error = ref<string | null>(null)
  const currentPage = ref(1)
  const hasNextPage = ref(false)
  let searchTimeout: ReturnType<typeof setTimeout> | null = null

  const fetchCounts = async () => {
    try {
      counts.value = await apiClient.getSidebar()
    } catch (err) {
      console.error('Failed to fetch counts:', err)
    }
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
      const response = await apiClient.inbox(currentPage.value, search)
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

  const handleNewMail = (email: EmailListRecord) => {
    fetchCounts()

    if (searchStore.query.trim()) {
      return
    }

    const existingIds = new Set(emails.value.map(e => e.id))
    if (existingIds.has(email.id)) {
      return
    }

    emails.value = [email, ...emails.value]
  }

  const handleEmailRead = (email: EmailListRecord) => {
    fetchCounts()

    if (searchStore.query.trim()) {
      return
    }

    const index = emails.value.findIndex(e => e.id === email.id)
    if (index !== -1) {
      emails.value[index] = email
    } else {
      emails.value = [email, ...emails.value]
    }
  }

  const handleEmailDeleted = (emailId: string) => {
    fetchCounts()

    if (searchStore.query.trim()) {
      return
    }

    const index = emails.value.findIndex(e => e.id === emailId)
    if (index !== -1) {
      emails.value.splice(index, 1)
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
          console.log(message)
          if (message.event === 'new_mail' && message.email) {
            handleNewMail(message.email)
          } else if (message.event === 'email_read' && message.email) {
            handleEmailRead(message.email)
          } else if (message.event === 'email_deleted' && message.email_id) {
            handleEmailDeleted(message.email_id)
          }
        } catch (err) {
          console.error('Failed to parse websocket message:', err)
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
      if (!newPath.match(/\/mails\/inbox\/[^/]+$/)) {
        searchStore.query = ''
        fetchMails()
      }
    }
  )
</script>
