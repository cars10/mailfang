<template>
  <div class="border-b border-gray-200 bg-gray-50 px-6 py-4">
    <div class="flex items-start justify-between gap-2 mb-2">
      <h1 class="text-2xl text-gray-900">
        {{ email.subject || '(No Subject)' }}
      </h1>

      <div class="flex flex-row gap-1">
        <button
          class="btn btn--icon"
          :title="email.archived ? 'Unarchive' : 'Archive'"
          :disabled="loadingArchive"
          @click="handleArchive"
        >
          <ArchiveBoxIcon class="h-5" />
        </button>

        <button
          class="btn btn--icon"
          title="Delete"
          :disabled="loadingDelete"
          @click="handleDelete"
        >
          <TrashIcon class="h-5" />
        </button>

        <button
          class="btn btn--icon"
          title="Download"
          :disabled="loadingDownload"
          @click="handleDownload"
        >
          <ArrowDownTrayIcon class="h-5" />
        </button>
      </div>
    </div>

    <div class="text-gray-600 text-nowrap flex flex-row justify-between">
      <div>
        {{ formatFullDate(email.created_at) }}
      </div>
      <div>
        {{ formatSize(email.size) }}
      </div>
    </div>

    <div class="grid grid-cols-[auto_1fr] gap-x-2">
      <template v-if="email.from">
        <div class="flex items-center text-gray-500 select-text">From:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.from" />
        </div>
      </template>

      <template v-if="email.to.length > 0">
        <div class="flex items-center text-gray-500 select-text">To:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.to.join(', ')" />
        </div>
      </template>

      <template v-if="email.headers?.Cc">
        <div class="flex items-center text-gray-500 select-text">CC:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.headers?.Cc?.join(', ') || ''" />
        </div>
      </template>

      <template v-if="email.headers?.Bcc">
        <div class="flex items-center text-gray-500 select-text">BCC:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.headers?.Bcc?.join(', ') || ''" />
        </div>
      </template>

      <template v-if="email.headers?.['Reply-To']">
        <div class="flex items-center text-gray-500 select-text">Reply-To:</div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.headers?.['Reply-To']?.join(', ') || ''" />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
  import { useRouter } from 'vue-router'
  import type { EmailRecord } from '@/types/email'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'
  import prettyBytes from 'pretty-bytes'
  import { apiClient } from '@/api/client'
  import {
    ArchiveBoxIcon,
    TrashIcon,
    ArrowDownTrayIcon,
  } from '@heroicons/vue/24/outline'

  const props = defineProps<{ email: EmailRecord }>()
  const emit = defineEmits<{
    update: []
  }>()

  const router = useRouter()
  const loadingArchive = ref(false)
  const loadingDelete = ref(false)
  const loadingDownload = ref(false)

  const handleArchive = async () => {
    if (loadingArchive.value) return

    try {
      loadingArchive.value = true
      await apiClient.archive(props.email.id, !props.email.archived)
      emit('update')
    } catch (err) {
      console.error('Failed to archive email:', err)
    } finally {
      loadingArchive.value = false
    }
  }

  const handleDelete = async () => {
    if (loadingDelete.value) return

    if (!confirm('Are you sure you want to delete this email?')) {
      return
    }

    try {
      loadingDelete.value = true
      await apiClient.deleteEmail(props.email.id)
      // Navigate back to inbox after deletion
      router.push('/mails/inbox')
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

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }

  const formatFullDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleString('en-US', {
      month: 'numeric',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
    })
  }
</script>
