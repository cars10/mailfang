<template>
  <div
    class="inline-flex rounded-sm shadow-xs -space-x-px"
    :class="{ 'gap-1': modelValue === undefined }"
  >
    <button
      v-for="(option, index) in options"
      :key="option.value"
      class="border border-gray-300 flex items-center gap-2 px-2 py-1 cursor-pointer hover:bg-gray-100 hover:shadow-sm outline-primary focus:border-primary"
      :class="{
        'bg-gray-100': modelValue === option.value,
        'rounded-s': index === 0,
        'rounded-e': index === props.options.length - 1,
      }"
      :title="option.title"
      @click="selectOption(option)"
    >
      <component
        :is="option.icon"
        v-if="option.icon"
        class="h-5 w-5"
        :class="{ 'text-primary': modelValue === option.value }"
      />
      <span v-if="option.label">{{ option.label }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
  import type { Component } from 'vue'

  export interface ButtonGroupOption {
    value: string
    label?: string
    icon?: Component
    title?: string
    onClick?: () => void
  }

  const props = defineProps<{
    modelValue?: string | null
    options: ButtonGroupOption[]
  }>()

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()

  const selectOption = (option: ButtonGroupOption) => {
    option.onClick?.()
    if (props.modelValue !== undefined) {
      emit('update:modelValue', option.value)
    }
  }
</script>
