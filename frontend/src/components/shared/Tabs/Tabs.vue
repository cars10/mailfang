<template>
  <div class="flex items-center gap-2 py-2">
    <template v-for="tab in tabs" :key="tab.id">
      <button
        class="focus:outline-none focus:border-primary px-4 py-2 text-sm font-medium transition-colors border-b-2 cursor-pointer flex items-center gap-2"
        :class="tabClass(tab.id)"
        @click="$emit('update:modelValue', tab.id)"
      >
        <component :is="tab.icon" v-if="tab.icon" class="h-4 w-4" />
        {{ tab.label }}
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
  import type { Component } from 'vue'

  export interface Tab {
    id: string
    label: string
    icon?: Component
  }

  interface Props {
    modelValue: string
    tabs: Tab[]
  }

  const props = defineProps<Props>()

  defineEmits<{
    'update:modelValue': [value: string]
  }>()

  const tabClass = (tabId: string) => {
    return props.modelValue === tabId
      ? 'text-primary border-primary'
      : 'text-gray-500 border-transparent hover:text-gray-700 hover:border-gray-300'
  }
</script>
