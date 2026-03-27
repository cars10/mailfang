import { AppThemes, ThemePreferences, useThemeStore } from '@/stores/Theme'

export const setAppThemeCss = (theme: AppThemes) => {
  const el = document.documentElement

  if (theme === AppThemes.dark) {
    el.setAttribute('data-theme', 'dark')
  } else {
    el.removeAttribute('data-theme')
  }
}

export const setupThemeListener = () => {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: light)')
  const themeStore = useThemeStore()

  mediaQuery.addEventListener('change', e => {
    if (themeStore.preference !== ThemePreferences.auto) return

    const appTheme = e.matches ? AppThemes.light : AppThemes.dark
    themeStore.appTheme = appTheme

    setAppThemeCss(appTheme)
  })
}
