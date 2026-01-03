<template>
  <div class="h-full">
    <iframe :src="renderedUrl" class="border-0 bg-white h-full w-full"></iframe>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import { useRoute } from 'vue-router'
  import { useMailLayoutStore } from '@/stores/MailLayout'

  const route = useRoute()
  const emailId = route.params.id as string
  const mailLayoutStore = useMailLayoutStore()

  const renderedUrl = computed(() => {
    const allowRemote = mailLayoutStore.allowRemoteContent
    return `/api/emails/${emailId}/rendered?allow_remote_content=${allowRemote}`
  })
</script>
