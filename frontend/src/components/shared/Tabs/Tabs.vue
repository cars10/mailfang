<template>
  <div
    class="flex items-center gap-2"
    :class="{ 'border-b border-gray-200': showBorder }"
  >
    <template v-for="tab in tabs" :key="tab.id">
      <div v-if="tab.spacer" class="grow"></div>
      <button
        v-else
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 cursor-pointer flex items-center gap-2"
        :class="getTabClass(tab.id)"
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

  interface Tab {
    id: string
    label?: string
    icon?: Component
    spacer?: boolean
  }

  interface Props {
    modelValue: string
    tabs: Tab[]
    showBorder?: boolean
  }

  const props = withDefaults(defineProps<Props>(), {
    showBorder: true,
  })

  defineEmits<{
    'update:modelValue': [value: string]
  }>()

  const getTabClass = (tabId: string) => {
    return props.modelValue === tabId
      ? 'text-blue-600 border-blue-600'
      : 'text-gray-500 border-transparent hover:text-gray-700 hover:border-gray-300'
  }
</script>
