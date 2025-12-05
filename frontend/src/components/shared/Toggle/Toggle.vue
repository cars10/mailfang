<template>
  <div class="flex flex-row items-center gap-2">
    <button
      id="toggle"
      type="button"
      class="cursor-pointer relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
      :class="modelValue ? 'bg-primary' : 'bg-gray-300'"
      @click="toggle"
    >
      <span
        class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
        :class="modelValue ? 'translate-x-6' : 'translate-x-1'"
      />
    </button>
    <label
      v-if="label"
      class="cursor-pointer text-sm text-gray-600"
      for="toggle"
    >
      {{ label }}
    </label>
  </div>
</template>

<script setup lang="ts">
  const props = defineProps<{
    modelValue: boolean
    label?: string
  }>()

  const emit = defineEmits<{
    'update:modelValue': [value: boolean]
  }>()

  const toggle = () => {
    emit('update:modelValue', !props.modelValue)
  }
</script>
