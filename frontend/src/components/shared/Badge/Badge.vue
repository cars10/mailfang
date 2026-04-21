<template>
  <button
    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-sm bg-app-gray-200 text-primary text-sm min-w-0 max-w-full cursor-pointer hover:bg-app-gray-300"
    :title="title"
    @click="$emit('click')"
  >
    <span class="truncate min-w-0 flex-1">{{ text }}</span>
    <span v-if="$slots.trailing" class="shrink-0">
      <slot name="trailing" />
    </span>
  </button>
</template>

<script setup lang="ts">
  defineProps<{
    text: string
    title?: string
  }>()

  defineEmits<{
    click: []
  }>()
</script>
