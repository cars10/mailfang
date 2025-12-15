<template>
  <ArrowPathIcon
    class="animate-spin text-primary"
    :class="size ? `h-${size} w-${size}` : 'h-4 w-4'"
  />
</template>

<script setup lang="ts">
  import { ArrowPathIcon } from '@heroicons/vue/24/outline'

  defineProps<{
    size?: string | number
  }>()
</script>
