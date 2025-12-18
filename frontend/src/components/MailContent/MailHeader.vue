<template>
  <div class="border-b border-gray-200 bg-gray-50 px-6 py-4">
    <div class="flex items-start justify-between gap-2 mb-2">
      <h1 class="text-2xl text-gray-900">
        {{ email.subject || '(No Subject)' }}
      </h1>

      <div class="flex flex-row gap-1">
        <button
          class="btn btn--icon"
          title="Download"
          :disabled="loadingDownload"
          @click="handleDownload"
        >
          <ArrowDownTrayIcon class="h-5" />
        </button>

        <button
          class="btn btn--icon"
          title="Delete"
          :disabled="loadingDelete"
          @click="handleDelete"
        >
          <TrashIcon class="h-5" />
        </button>
      </div>
    </div>

    <div class="flex flex-row gap-2 justify-between">
      <div>
        <div class="grid grid-cols-[auto_1fr] gap-x-2">
          <template v-if="email.from">
            <div
              class="flex items-center text-gray-500 select-text text-nowrap"
            >
              From:
            </div>
            <div class="text-gray-600 select-text w-fit">
              <CopyBadge :text="email.from" />
            </div>
          </template>

          <template v-if="email.to.length > 0">
            <div
              class="flex items-center text-gray-500 select-text text-nowrap"
            >
              To:
            </div>
            <div class="text-gray-600 select-text w-fit">
              <CopyBadge :text="email.to.join(', ')" />
            </div>
          </template>

          <template v-if="email.headers?.Cc">
            <div
              class="flex items-center text-gray-500 select-text text-nowrap"
            >
              CC:
            </div>
            <div class="text-gray-600 select-text w-fit">
              <CopyBadge :text="email.headers?.Cc?.join(', ') || ''" />
            </div>
          </template>

          <template v-if="email.headers?.Bcc">
            <div
              class="flex items-center text-gray-500 select-text text-nowrap"
            >
              BCC:
            </div>
            <div class="text-gray-600 select-text w-fit">
              <CopyBadge :text="email.headers?.Bcc?.join(', ') || ''" />
            </div>
          </template>

          <template v-if="email.headers?.['Reply-To']">
            <div
              class="flex items-center text-gray-500 select-text text-nowrap"
            >
              Reply-To:
            </div>
            <div class="text-gray-600 select-text w-fit">
              <CopyBadge
                :text="email.headers?.['Reply-To']?.join(', ') || ''"
              />
            </div>
          </template>
        </div>
      </div>

      <div>
        <div class="grid grid-cols-[auto_1fr] gap-x-2">
          <div class="flex items-center text-gray-500 select-text">Date:</div>
          <div class="text-gray-600 select-text w-fit">
            <CopyBadge :text="formatFullDate(email.date || email.created_at)" />
          </div>

          <div class="flex items-center text-gray-500 select-text">Size:</div>
          <div class="text-gray-600 select-text w-fit">
            <CopyBadge :text="formatSize(email.size)" />
          </div>
        </div>
      </div>
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
  import { TrashIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/outline'

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

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }

  const formatFullDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleString()
  }
</script>
