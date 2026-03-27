<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background-color: rgba(0, 0, 0, 0.5)"
      @click.self="close"
    >
      <div
        class="bg-app-white rounded-sm shadow-lg max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto"
      >
        <div class="p-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-xl font-semibold text-app-gray-900">{{ title }}</h2>
            <button
              type="button"
              class="text-app-gray-500 cursor-pointer hover:text-app-gray-600 focus:outline-none"
              @click="close"
            >
              <XMarkIcon class="h-6 w-6" />
            </button>
          </div>
          <div class="text-app-gray-700">
            <slot />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
  import { XMarkIcon } from '@heroicons/vue/24/outline'

  defineProps<{
    isOpen: boolean
    title: string
  }>()

  const emit = defineEmits<{ 'update:isOpen': [value: boolean] }>()

  const close = () => {
    emit('update:isOpen', false)
  }
</script>
