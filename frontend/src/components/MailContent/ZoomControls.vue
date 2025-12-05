<template>
  <div class="flex flex-row p-2 gap-2">
    <button class="btn btn--icon" title="Zoom in" @click="zoomIn">
      <MagnifyingGlassPlusIcon class="w-5 h-5" />
    </button>
    <button class="btn btn--icon" title="Reset zoom" @click="resetZoom">
      <MagnifyingGlassCircleIcon class="w-5 h-5" />
    </button>
    <button class="btn btn--icon" title="Zoom out" @click="zoomOut">
      <MagnifyingGlassMinusIcon class="w-5 h-5" />
    </button>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import {
    MagnifyingGlassPlusIcon,
    MagnifyingGlassMinusIcon,
    MagnifyingGlassCircleIcon,
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
