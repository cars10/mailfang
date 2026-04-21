<template>
  <div
    ref="container"
    class="overflow-y-auto min-h-0 flex-1"
    @scroll.passive="handleScroll"
  >
    <template v-if="items.length > 0">
      <div :style="{ height: `${topSpacerHeight}px` }" />

      <template
        v-for="(item, visibleIndex) in visibleItems"
        :key="props.itemKey(item, startIndex + visibleIndex)"
      >
        <slot :item="item" :index="startIndex + visibleIndex" />
      </template>

      <div :style="{ height: `${bottomSpacerHeight}px` }" />
    </template>

    <slot v-else name="empty" />
  </div>
</template>

<script setup lang="ts" generic="T">
  import {
    computed,
    nextTick,
    onBeforeUnmount,
    onMounted,
    ref,
    watch,
  } from 'vue'

  type KeyType = string | number
  type ItemKeyGetter = (item: T, index: number) => KeyType

  interface Props {
    items: T[]
    rowHeight: number
    itemKey?: ItemKeyGetter
    overscanRows?: number
    minHeight?: number
    resetOnItemsChange?: boolean
  }

  const props = withDefaults(defineProps<Props>(), {
    itemKey: (_item: T, index: number) => index,
    overscanRows: 6,
    minHeight: 160,
    resetOnItemsChange: true,
  })
  defineSlots<{
    default(props: { item: T; index: number }): unknown
    empty?: () => unknown
  }>()

  const container = ref<HTMLElement | null>(null)
  const scrollTop = ref(0)
  const containerHeight = ref(0)

  const measureContainerHeight = () => {
    containerHeight.value = container.value?.clientHeight ?? 0
  }

  const visibleRows = computed(() => {
    return Math.max(1, Math.ceil(containerHeight.value / props.rowHeight))
  })

  const startIndex = computed(() => {
    const start =
      Math.floor(scrollTop.value / props.rowHeight) - props.overscanRows
    return Math.max(0, start)
  })

  const endIndex = computed(() => {
    const end = startIndex.value + visibleRows.value + props.overscanRows * 2
    return Math.min(props.items.length, end)
  })

  const visibleItems = computed(() => {
    return props.items.slice(startIndex.value, endIndex.value)
  })

  const topSpacerHeight = computed(() => {
    return startIndex.value * props.rowHeight
  })

  const bottomSpacerHeight = computed(() => {
    const totalHeight = props.items.length * props.rowHeight
    const renderedHeight = (endIndex.value - startIndex.value) * props.rowHeight
    return Math.max(0, totalHeight - topSpacerHeight.value - renderedHeight)
  })

  const handleScroll = () => {
    scrollTop.value = container.value?.scrollTop ?? 0
  }

  const resetScroll = () => {
    scrollTop.value = 0
    if (container.value) {
      container.value.scrollTop = 0
    }
  }

  watch(
    () => props.items,
    () => {
      if (props.resetOnItemsChange) {
        resetScroll()
      }
      measureContainerHeight()
    }
  )

  watch(
    () => [props.rowHeight, props.minHeight],
    () => {
      measureContainerHeight()
    }
  )

  let resizeObserver: ResizeObserver | null = null

  const handleWindowResize = () => {
    measureContainerHeight()
  }

  onMounted(() => {
    nextTick(() => {
      measureContainerHeight()
    })

    if (container.value) {
      resizeObserver = new ResizeObserver(() => {
        measureContainerHeight()
      })
      resizeObserver.observe(container.value)
    }

    window.addEventListener('resize', handleWindowResize)
  })

  onBeforeUnmount(() => {
    resizeObserver?.disconnect()
    window.removeEventListener('resize', handleWindowResize)
  })
</script>
