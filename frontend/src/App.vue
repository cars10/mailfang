<template>
  <RouterView />
</template>

<script setup lang="ts">
  import { onMounted } from 'vue'
  import { useThemeStore } from '@/stores/Theme'
  import { setAppThemeCss, setupThemeListener } from '@/helpers/theme'

  const themeStore = useThemeStore()

  onMounted(() => {
    setAppThemeCss(themeStore.appTheme)
    setupThemeListener()
  })
</script>
