<template>
  <div class="flex flex-row items-center gap-2">
    <button
      id="toggle"
      type="button"
      class="cursor-pointer relative inline-flex items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
      :class="[sizeClasses.button, modelValue ? 'bg-primary' : 'bg-gray-300']"
      @click="toggle"
    >
      <span
        class="inline-block transform rounded-full bg-white transition-transform"
        :class="[
          sizeClasses.knob,
          modelValue ? sizeClasses.translateOn : sizeClasses.translateOff,
        ]"
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
  import { computed } from 'vue'

  const props = withDefaults(
    defineProps<{
      modelValue: boolean
      label?: string
      small?: boolean
    }>(),
    {
      label: '',
      small: false,
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: boolean]
  }>()

  const sizeClasses = computed(() =>
    props.small
      ? {
          button: 'h-5 w-9',
          knob: 'h-3.5 w-3.5',
          translateOn: 'translate-x-4.5',
          translateOff: 'translate-x-1',
        }
      : {
          button: 'h-6 w-11',
          knob: 'h-4 w-4',
          translateOn: 'translate-x-6',
          translateOff: 'translate-x-1',
        }
  )

  const toggle = () => {
    emit('update:modelValue', !props.modelValue)
  }
</script>
