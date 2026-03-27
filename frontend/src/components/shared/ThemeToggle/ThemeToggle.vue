<template>
  <ButtonGroup
    :model-value="themeStore.preference"
    :options="themeOptions"
    @update:model-value="onThemeChange"
  />
</template>

<script setup lang="ts">
  import { SunIcon, MoonIcon, UserIcon } from '@heroicons/vue/24/outline'
  import ButtonGroup, {
    type ButtonGroupOption,
  } from '@/components/shared/ButtonGroup/ButtonGroup.vue'
  import { useThemeStore, ThemePreferences } from '@/stores/Theme'
  import { setAppThemeCss } from '@/helpers/theme'

  const themeStore = useThemeStore()

  const themeOptions: ButtonGroupOption[] = [
    {
      value: ThemePreferences.light,
      icon: SunIcon,
      title: 'Light theme',
    },
    {
      value: ThemePreferences.auto,
      icon: UserIcon,
      title: 'Auto theme',
    },
    {
      value: ThemePreferences.dark,
      icon: MoonIcon,
      title: 'Dark theme',
    },
  ]

  const changeTheme = (theme: ThemePreferences) => {
    themeStore.setPreference(theme)
    setAppThemeCss(themeStore.appTheme)
  }

  const onThemeChange = (value: string) => {
    changeTheme(value as ThemePreferences)
  }
</script>
