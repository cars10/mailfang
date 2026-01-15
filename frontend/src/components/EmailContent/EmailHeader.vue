<template>
  <div class="px-6 py-4">
    <div class="flex justify-between gap-4 mb-2">
      <h1 class="text-3xl text-gray-900 font-weight-semibold mb-4">
        {{ email.subject || '(No Subject)' }}
      </h1>

      <div class="flex items-start gap-3">
        <div class="text-gray-500 flex align-center text-nowrap py-2">
          {{ formatDate(email.created_at) }}
        </div>
        <DropdownMenu :items="menuItems" />
      </div>
    </div>

    <div class="grid grid-cols-[max-content_1fr] gap-x-2">
      <template v-if="displayFrom">
        <div class="text-gray-500 select-text text-right text-nowrap py-1">
          From
        </div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="displayFrom" />
        </div>
      </template>

      <template v-if="displayTo.length > 0">
        <div class="text-gray-500 select-text text-right text-nowrap py-1">
          To
        </div>
        <div class="w-fit">
          <div
            v-for="recipient in displayTo"
            :key="recipient"
            class="text-gray-600 select-text w-fit"
          >
            <CopyBadge :text="recipient" />
          </div>
        </div>
      </template>

      <template v-if="displayCc.length > 0">
        <div class="text-right text-gray-500 select-text text-nowrap py-1">
          CC
        </div>
        <div class="w-fit">
          <div
            v-for="cc in displayCc"
            :key="cc"
            class="text-gray-600 select-text w-fit"
          >
            <CopyBadge :text="cc" />
          </div>
        </div>
      </template>

      <template v-if="displayBcc.length > 0">
        <div class="text-right text-gray-500 select-text text-nowrap py-1">
          BCC
        </div>
        <div class="w-fit">
          <div
            v-for="bcc in displayBcc"
            :key="bcc"
            class="text-gray-600 select-text w-fit"
          >
            <CopyBadge :text="bcc" />
          </div>
        </div>
      </template>

      <template v-if="displayReplyTo.length > 0">
        <div class="text-right text-gray-500 select-text text-nowrap py-1">
          Reply-To
        </div>
        <div class="w-fit">
          <div
            v-for="replyTo in displayReplyTo"
            :key="replyTo"
            class="text-gray-600 select-text w-fit"
          >
            <CopyBadge :text="replyTo" />
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed } from 'vue'
  import { useRouter } from 'vue-router'
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'
  import DropdownMenu from '@/components/shared/DropdownMenu/DropdownMenu.vue'
  import type { DropdownMenuItem } from '@/components/shared/DropdownMenu/DropdownMenu.vue'
  import { apiClient } from '@/api/client'
  import { TrashIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/outline'
  import { parseAndDecodeHeaderValues } from '@/utils/emailAddress'

  const props = defineProps<{ email: EmailRecord }>()

  const router = useRouter()
  const loadingDelete = ref(false)
  const loadingDownload = ref(false)

  const handleDelete = async () => {
    if (loadingDelete.value) return

    if (!confirm('Are you sure you want to delete this email?')) {
      return
    }

    try {
      loadingDelete.value = true
      await apiClient.deleteEmail(props.email.id)
      // Navigate back to inbox after deletion
      router.push('/emails/inbox')
    } catch (err) {
      console.error('Failed to delete email:', err)
    } finally {
      loadingDelete.value = false
    }
  }

  const handleDownload = async () => {
    if (loadingDownload.value) return

    try {
      loadingDownload.value = true
      const rawContent = await apiClient.getRawEmail(props.email.id)

      // Create a blob and download it
      const blob = new Blob([rawContent], { type: 'message/rfc822' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `email-${props.email.id}.eml`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    } catch (err) {
      console.error('Failed to download email:', err)
    } finally {
      loadingDownload.value = false
    }
  }

  const menuItems = computed<DropdownMenuItem[]>(() => [
    {
      id: 'download',
      label: 'Download',
      icon: ArrowDownTrayIcon,
      disabled: loadingDownload.value,
      onClick: handleDownload,
    },
    {
      id: 'delete',
      label: 'Delete',
      icon: TrashIcon,
      disabled: loadingDelete.value,
      onClick: handleDelete,
    },
  ])

  const formatDate = (dateString: string) => {
    const today = new Date()
    const mailDate = new Date(dateString)

    const isToday =
      mailDate.getDate() === today.getDate() &&
      mailDate.getMonth() === today.getMonth() &&
      mailDate.getFullYear() === today.getFullYear()

    return isToday
      ? mailDate.toLocaleTimeString()
      : mailDate.toLocaleDateString()
  }

  const displayFrom = computed(() => {
    return parseAndDecodeHeaderValues(props.email.headers?.From)?.[0]
  })

  const displayTo = computed(() => {
    return parseAndDecodeHeaderValues(props.email.headers?.To)
  })

  const displayCc = computed(() => {
    return parseAndDecodeHeaderValues(props.email.headers?.Cc)
  })

  const displayBcc = computed(() => {
    return parseAndDecodeHeaderValues(props.email.headers?.Bcc)
  })

  const displayReplyTo = computed(() => {
    return parseAndDecodeHeaderValues(props.email.headers?.['Reply-To'])
  })
</script>
