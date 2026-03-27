<template>
  <div class="relative flex items-center">
    <component :is="icon" v-if="icon" :class="iconClasses" />
    <input
      ref="inputEl"
      :value="modelValue"
      type="text"
      :placeholder="placeholder"
      class="rounded-sm border border-app-gray-300 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary overflow-hidden"
      :class="inputClasses"
      @input="handleInput"
      @keydown.escape="handleEscape"
      @focus="handleFocus"
      @blur="handleBlur"
    />
  </div>
</template>

<script setup lang="ts">
  import { computed, ref } from 'vue'
  import type { Component } from 'vue'

  const props = withDefaults(
    defineProps<{
      modelValue: string
      placeholder?: string
      icon?: Component
      dense?: boolean
    }>(),
    {
      placeholder: undefined,
      icon: undefined,
      dense: false,
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: string]
    focus: [event: FocusEvent]
    blur: [event: FocusEvent]
  }>()

  const iconClasses = computed(() => {
    return `absolute pointer-events-none z-10 left-3 h-5 w-5 text-app-gray-500`
  })

  const inputEl = ref<HTMLInputElement | null>(null)

  const inputClasses = computed(() => {
    if (props.icon) {
      return 'pl-10 pr-2 py-2 w-full'
    }
    return props.dense ? 'px-2 py-1 w-full' : 'px-2 py-2 w-full'
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

  const focus = () => {
    inputEl.value?.focus()
  }

  defineExpose({
    focus,
  })
</script>
