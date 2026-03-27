<template>
  <div
    class="flex items-center gap-2 rounded-sm border border-app-gray-300 px-2 focus-within:ring-1 focus-within:ring-primary focus-within:border-primary overflow-hidden"
  >
    <component :is="icon" v-if="icon" :class="iconClasses" />
    <div class="grow min-w-0">
      <input
        ref="inputEl"
        :value="modelValue"
        type="text"
        :placeholder="placeholder"
        class="w-full bg-transparent focus:outline-none"
        :class="inputClasses"
        @input="handleInput"
        @keydown.escape="handleEscape"
        @focus="handleFocus"
        @blur="handleBlur"
      />
    </div>
    <div v-if="hasRightSlot" class="shrink-0 flex items-center">
      <slot name="right" />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { computed, ref, useSlots } from 'vue'
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

  const slots = useSlots()
  const hasRightSlot = computed(() => Boolean(slots.right))

  const iconClasses = computed(() => {
    return `h-5 w-5 shrink-0 text-app-gray-500`
  })

  const inputEl = ref<HTMLInputElement | null>(null)

  const inputClasses = computed(() => {
    const verticalPadding = props.dense ? 'py-1' : 'py-2'
    return `${verticalPadding}`
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
