<template>
  <div class="flex flex-row h-full">
    <MailSidebar :counts="counts" />

    <MailList
      :emails="emails"
      :loading="loading"
      :error="error"
      @load-more="fetchNextPage"
    />

    <div class="grow">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, watch } from 'vue'
  import { useWebSocket } from '@/composables/useWebSocket'
  import { apiClient } from '@/api/client'
  import { useSearchStore } from '@/stores/Search'
  import type { EmailListRecord, EmailCounts } from '@/types/email'
  import MailSidebar from './MailSidebar.vue'
  import MailList from './MailList.vue'
  import { useRoute } from 'vue-router'

  const searchStore = useSearchStore()
  const route = useRoute()
  const emails = ref<EmailListRecord[]>([])
  const counts = ref<EmailCounts>({
    inbox: 0,
    recipients: [],
  })
  const loading = ref(false)
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

  const fetchPage = async (page: number, append: boolean) => {
    if (loading.value) return

    loading.value = true
    error.value = null
    const search = searchStore.query
    const recipient = route.params.recipient as string | undefined

    try {
      const response = recipient
        ? await apiClient.inboxByRecipient(recipient, page, search)
        : await apiClient.inbox(page, search)
      emails.value = append
        ? [...emails.value, ...response.emails]
        : response.emails
      counts.value = response.counts
      currentPage.value = response.pagination.page
      hasNextPage.value =
        response.pagination.page < response.pagination.total_pages
    } catch (err) {
      error.value =
        err instanceof Error ? err.message : 'Failed to fetch emails'
    } finally {
      loading.value = false
    }
  }

  const fetchInitialMails = async () => {
    await fetchPage(1, false)
  }

  const fetchNextPage = async () => {
    if (!hasNextPage.value) return

    const nextPage = currentPage.value + 1
    await fetchPage(nextPage, true)
  }

  const handleNewMail = (email: EmailListRecord, recipients?: string[]) => {
    fetchCounts()

    if (searchStore.query) return

    const existingIds = new Set(emails.value.map(e => e.id))
    if (existingIds.has(email.id)) {
      return
    }

    const currentRecipient = route.params.recipient as string | undefined

    if (!currentRecipient) {
      emails.value = [email, ...emails.value]
      return
    }

    if (recipients && recipients.includes(currentRecipient)) {
      emails.value = [email, ...emails.value]
    }
  }

  const handleEmailRead = (email: EmailListRecord) => {
    const index = emails.value.findIndex(e => e.id === email.id)
    if (index !== -1) emails.value[index] = email
  }

  const handleEmailDeleted = (emailId: string) => {
    fetchCounts()

    const index = emails.value.findIndex(e => e.id === emailId)
    if (index !== -1) emails.value.splice(index, 1)
  }

  useWebSocket(
    {
      onMessage: event => {
        try {
          const message = JSON.parse(event.data)
          if (message.event === 'new_mail' && message.email) {
            handleNewMail(message.email, message.recipients)
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
    fetchInitialMails()
  })

  watch(
    () => searchStore.query,
    () => {
      if (searchTimeout) {
        clearTimeout(searchTimeout)
      }
      searchTimeout = setTimeout(() => {
        fetchInitialMails()
      }, 300)
    }
  )

  watch(
    () => route.params.recipient,
    () => {
      fetchInitialMails()
    }
  )
</script>
