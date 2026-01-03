<template>
  <ButtonGroup :options="buttonOptions" />
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import {
    MagnifyingGlassPlusIcon,
    MagnifyingGlassMinusIcon,
    MagnifyingGlassCircleIcon,
  } from '@heroicons/vue/24/outline'
  import ButtonGroup, {
    type ButtonGroupOption,
  } from '../shared/ButtonGroup/ButtonGroup.vue'

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

  const buttonOptions: ButtonGroupOption[] = [
    {
      value: 'zoom-in',
      icon: MagnifyingGlassPlusIcon,
      title: 'Zoom in',
      onClick: zoomIn,
    },
    {
      value: 'reset',
      icon: MagnifyingGlassCircleIcon,
      title: 'Reset zoom',
      onClick: resetZoom,
    },
    {
      value: 'zoom-out',
      icon: MagnifyingGlassMinusIcon,
      title: 'Zoom out',
      onClick: zoomOut,
    },
  ]
</script>
