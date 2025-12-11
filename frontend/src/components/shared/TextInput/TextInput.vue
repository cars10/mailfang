<template>
  <div class="relative flex items-center">
    <component
      :is="icon"
      v-if="icon"
      class="absolute left-3 h-5 w-5 text-gray-400 pointer-events-none"
    />
    <input
      :value="modelValue"
      type="text"
      :placeholder="placeholder"
      class="w-full rounded-sm border border-gray-300 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary"
      :class="icon ? 'pl-10 pr-2 py-2' : 'px-2 py-2'"
      @input="handleInput"
      @keydown.escape="handleEscape"
    />
  </div>
</template>

<script setup lang="ts">
  import type { Component } from 'vue'

  defineProps<{
    modelValue: string
    placeholder?: string
    icon?: Component
  }>()

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()

  const handleInput = (event: Event) => {
    const target = event.target as HTMLInputElement
    emit('update:modelValue', target.value)
  }

  const handleEscape = () => {
    emit('update:modelValue', '')
  }
</script>
