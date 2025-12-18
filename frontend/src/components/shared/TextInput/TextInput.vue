<template>
  <div class="relative flex items-center">
    <component :is="icon" v-if="icon" :class="iconClasses" />
    <input
      :value="modelValue"
      type="text"
      :placeholder="placeholder"
      class="rounded-sm border border-gray-300 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary overflow-hidden"
      :class="inputClasses"
      :style="inputStyles"
      @input="handleInput"
      @keydown.escape="handleEscape"
      @focus="handleFocus"
      @blur="handleBlur"
    />
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import type { Component } from 'vue'

  const props = withDefaults(
    defineProps<{
      modelValue: string
      placeholder?: string
      icon?: Component
      expandable?: boolean
      expandedWidth?: string
    }>(),
    {
      placeholder: undefined,
      icon: undefined,
      expandable: false,
      expandedWidth: '200px',
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: string]
    focus: [event: FocusEvent]
    blur: [event: FocusEvent]
  }>()

  const iconClasses = computed(() => {
    const base = 'absolute pointer-events-none z-10'
    if (props.expandable) {
      return `${base} left-2 h-4 w-4 text-gray-500`
    }
    return `${base} left-3 h-5 w-5 text-gray-400`
  })

  const inputClasses = computed(() => {
    if (props.expandable) {
      const baseClasses =
        'w-8 text-transparent focus:w-[var(--expanded-width)] focus:text-inherit placeholder:opacity-0 focus:placeholder:opacity-100'
      if (props.icon) {
        return `${baseClasses} pl-8 focus:pr-2 py-1 text-sm`
      }
      return `${baseClasses} pl-2 pr-2 py-1 text-sm`
    }
    if (props.icon) {
      return 'pl-10 pr-2 py-2 w-full'
    }
    return 'px-2 py-2 w-full'
  })

  const inputStyles = computed(() => {
    if (!props.expandable) {
      return {}
    }
    return {
      '--expanded-width': props.expandedWidth,
    }
  })

  const handleInput = (event: Event) => {
    const target = event.target as HTMLInputElement
    emit('update:modelValue', target.value.trim())
  }

  const handleEscape = () => {
    emit('update:modelValue', '')
  }

  const handleFocus = (event: FocusEvent) => {
    emit('focus', event)
  }

  const handleBlur = (event: FocusEvent) => {
    emit('blur', event)
  }
</script>
