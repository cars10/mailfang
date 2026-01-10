<template>
  <div ref="dropdownRef" class="relative">
    <button type="button" class="btn btn--icon" @click="toggleMenu">
      <EllipsisVerticalIcon class="h-5 w-5 text-gray-600" />
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 mt-1 rounded-sm shadow-lg bg-white border border-gray-300 z-50"
      role="menu"
      aria-orientation="vertical"
    >
      <div class="py-1">
        <button
          v-for="item in items"
          :key="item.id"
          type="button"
          class="w-full flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-700 cursor-pointer hover:bg-gray-50 hover:text-gray-900 focus:outline-none focus:bg-gray-50 focus:text-gray-900 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-white disabled:hover:text-gray-700"
          role="menuitem"
          :disabled="item.disabled"
          @click="handleItemClick(item)"
        >
          <component :is="item.icon" v-if="item.icon" class="h-4 w-4" />
          <span>{{ item.label }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, onUnmounted } from 'vue'
  import { EllipsisVerticalIcon } from '@heroicons/vue/24/outline'
  import type { Component } from 'vue'

  export interface DropdownMenuItem {
    id: string
    label: string
    icon?: Component
    disabled?: boolean
    onClick: () => void
  }

  defineProps<{
    items: DropdownMenuItem[]
  }>()

  const isOpen = ref(false)
  const dropdownRef = ref<HTMLElement | null>(null)

  const toggleMenu = () => {
    isOpen.value = !isOpen.value
  }

  const handleItemClick = (item: DropdownMenuItem) => {
    if (item.disabled) return
    item.onClick()
    isOpen.value = false
  }

  const handleClickOutside = (event: MouseEvent) => {
    if (
      dropdownRef.value &&
      !dropdownRef.value.contains(event.target as Node)
    ) {
      isOpen.value = false
    }
  }

  onMounted(() => {
    document.addEventListener('click', handleClickOutside)
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
  })
</script>
