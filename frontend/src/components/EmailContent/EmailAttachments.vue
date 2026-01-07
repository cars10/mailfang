<template>
  <div>
    <div v-if="attachments.length > 0" class="flex flex-wrap gap-2">
      <a
        v-for="attachment in attachments"
        :key="attachment.id"
        :href="getAttachmentUrl(attachment.id)"
        target="_blank"
        class="btn btn--small"
      >
        <component
          :is="getAttachmentIcon(attachment.content_type)"
          class="h-4 w-4 text-gray-600"
        />
        <span>{{ attachment.filename || 'unnamed' }}</span>
        <span class="text-xs text-gray-600"
          >({{ formatSize(attachment.size) }})</span
        >
      </a>
    </div>

    <div v-if="inlineAttachments.length > 0" class="flex flex-wrap gap-2 mb-2">
      <div
        class="text-gray-500 flex items-center cursor-help"
        title="Inline attachments are usually images displayed inside the email content and not typically intended to be downloaded."
      >
        Inline attachments:
      </div>
      <a
        v-for="attachment in inlineAttachments"
        :key="attachment.id"
        :href="getAttachmentUrl(attachment.id)"
        target="_blank"
        class="btn btn--small"
      >
        <component
          :is="getAttachmentIcon(attachment.content_type)"
          class="h-4 w-4 text-gray-600"
        />
        <span>{{ attachment.filename || 'unnamed' }}</span>
        <span class="text-xs text-gray-600"
          >({{ formatSize(attachment.size) }})</span
        >
      </a>
    </div>
  </div>
</template>

<script setup lang="ts">
  import type { EmailRecord } from '@/types/email'
  import type { Component } from 'vue'
  import prettyBytes from 'pretty-bytes'
  import { apiClient } from '@/api/client'
  import {
    PaperClipIcon,
    PhotoIcon,
    DocumentIcon,
    VideoCameraIcon,
    MusicalNoteIcon,
    CodeBracketIcon,
    ArchiveBoxIcon,
  } from '@heroicons/vue/24/outline'
  import { computed } from 'vue'

  const props = defineProps<{
    email: EmailRecord
  }>()

  const inlineAttachments = computed(() => {
    return props.email.attachments.filter(
      attachment => attachment.disposition === 'inline'
    )
  })

  const attachments = computed(() => {
    return props.email.attachments.filter(
      attachment => attachment.disposition !== 'inline'
    )
  })

  const getAttachmentUrl = (id: string) => {
    return apiClient.getAttachmentUrl(id)
  }

  const formatSize = (size: number) => {
    return prettyBytes(size)
  }

  const getAttachmentIcon = (contentType: string | null): Component => {
    if (!contentType) return PaperClipIcon

    const mimeType = contentType?.toLowerCase().split(';')?.[0]?.trim() ?? ''
    if (mimeType.length === 0) return PaperClipIcon

    if (mimeType.startsWith('image/')) return PhotoIcon
    if (mimeType.startsWith('video/')) return VideoCameraIcon
    if (mimeType.startsWith('audio/')) return MusicalNoteIcon

    if (
      mimeType === 'application/zip' ||
      mimeType === 'application/x-zip-compressed' ||
      mimeType === 'application/gzip' ||
      mimeType === 'application/x-gzip' ||
      mimeType === 'application/x-tar' ||
      mimeType === 'application/x-rar-compressed' ||
      mimeType === 'application/x-7z-compressed' ||
      (mimeType.startsWith('application/x-') && mimeType.includes('archive'))
    ) {
      return ArchiveBoxIcon
    }

    if (
      mimeType === 'application/pdf' ||
      mimeType === 'application/msword' ||
      mimeType ===
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document' ||
      mimeType === 'application/vnd.ms-excel' ||
      mimeType ===
        'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' ||
      mimeType === 'application/vnd.ms-powerpoint' ||
      mimeType ===
        'application/vnd.openxmlformats-officedocument.presentationml.presentation' ||
      mimeType.startsWith('application/vnd.')
    ) {
      return DocumentIcon
    }

    if (
      mimeType.startsWith('text/') ||
      mimeType === 'application/json' ||
      mimeType === 'application/xml' ||
      mimeType === 'application/javascript' ||
      mimeType === 'application/typescript'
    ) {
      return CodeBracketIcon
    }

    return PaperClipIcon
  }
</script>
