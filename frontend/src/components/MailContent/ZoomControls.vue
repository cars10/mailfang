<template>
  <div
    class="p-2 bg-gray-200 absolute top-2 left-2 shadow-md rounded-sm flex gap-1 opacity-50 hover:opacity-100"
  >
    <button class="btn btn--icon" title="Zoom in" @click="zoomIn">
      <PlusIcon class="w-4 h-4" />
    </button>
    <button class="btn btn--icon" title="Reset zoom" @click="resetZoom">
      <ArrowsPointingOutIcon class="w-4 h-4" />
    </button>
    <button class="btn btn--icon" title="Zoom out" @click="zoomOut">
      <MinusIcon class="w-4 h-4" />
    </button>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import {
    PlusIcon,
    MinusIcon,
    ArrowsPointingOutIcon,
  } from '@heroicons/vue/24/outline'

  const props = defineProps<{
    modelValue: number
  }>()

  const emit = defineEmits<{
    'update:modelValue': [value: number]
  }>()

  const zoom = computed({
    get: () => props.modelValue,
    set: (value: number) => emit('update:modelValue', value),
  })

  const zoomIn = () => {
    zoom.value = Math.min(zoom.value + 0.1, 3.0)
  }

  const zoomOut = () => {
    zoom.value = Math.max(zoom.value - 0.1, 0.1)
  }

  const resetZoom = () => {
    zoom.value = 1.0
  }
</script>
