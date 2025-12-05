<template>
  <div
    class="p-2 bg-gray-200 absolute top-2 right-2 shadow-md rounded-sm flex gap-1 opacity-50 hover:opacity-100"
  >
    <button
      class="btn btn--icon"
      :title="
        blockExternalRequests
          ? 'Allow external requests'
          : 'Block external requests'
      "
      @click="toggleBlocking"
    >
      <LockClosedIcon
        v-if="blockExternalRequests"
        class="w-4 h-4 text-green-600"
      />
      <LockOpenIcon v-else class="w-4 h-4 text-orange-600" />
    </button>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import { LockClosedIcon, LockOpenIcon } from '@heroicons/vue/24/outline'
  import { useMailLayoutStore } from '@/stores/MailLayout'

  const mailLayoutStore = useMailLayoutStore()

  const blockExternalRequests = computed({
    get: () => mailLayoutStore.blockExternalRequests,
    set: (value: boolean) => {
      mailLayoutStore.blockExternalRequests = value
    },
  })

  const toggleBlocking = () => {
    blockExternalRequests.value = !blockExternalRequests.value
  }
</script>
