<template>
  <div class="flex items-center gap-2 border-b border-gray-200">
    <button
      v-for="tab in visibleTabs"
      :key="tab.id"
      class="px-4 py-2 text-sm font-medium transition-colors border-b-2 cursor-pointer flex items-center gap-2"
      :class="
        modelValue === tab.id
          ? 'text-blue-600 border-blue-600'
          : 'text-gray-500 border-transparent hover:text-gray-700 hover:border-gray-300'
      "
      @click="$emit('update:modelValue', tab.id)"
    >
      <component :is="tab.icon" v-if="tab.icon" class="h-4 w-4" />
      {{ tab.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import type { Component } from 'vue'

  interface Tab {
    id: string
    label: string
    visible?: boolean
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

  const visibleTabs = computed(() => {
    return props.tabs.filter(tab => tab.visible !== false)
  })
</script>
