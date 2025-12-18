import { defineStore } from 'pinia'

export const ScreenSize = {
  Mobile: 'mobile',
  Tablet: 'tablet',
  Desktop: 'desktop',
} as const
export type ScreenSize = (typeof ScreenSize)[keyof typeof ScreenSize]

export const DEFAULT_INBOX_WIDTH = 350
export const DEFAULT_SIDEBAR_WIDTH = 250

type MailLayoutState = {
  inboxWidth: number
  sidebarWidth: number
  sidebarCollapsed: boolean
  mailContentZoom: number
  allowRemoteContent: boolean
  screenSize: ScreenSize
  recipientsCollapsed: boolean
}

export const useMailLayoutStore = defineStore('mailLayout', {
  state: (): MailLayoutState => ({
    inboxWidth: DEFAULT_INBOX_WIDTH,
    sidebarWidth: DEFAULT_SIDEBAR_WIDTH,
    sidebarCollapsed: false,
    mailContentZoom: 1.0,
    allowRemoteContent: true,
    screenSize: ScreenSize.Desktop,
    recipientsCollapsed: false,
  }),
  persist: true,
})
