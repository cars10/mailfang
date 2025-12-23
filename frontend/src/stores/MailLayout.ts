import { defineStore } from 'pinia'

export const ScreenSize = {
  Mobile: 'mobile',
  Tablet: 'tablet',
  Desktop: 'desktop',
} as const
export type ScreenSize = (typeof ScreenSize)[keyof typeof ScreenSize]

export const DEFAULT_INBOX_WIDTH = 400
export const DEFAULT_SIDEBAR_WIDTH = 250
export const DEFAULT_RECIPIENTS_HEIGHT = 28 * 8

type MailLayoutState = {
  inboxWidth: number
  sidebarWidth: number
  mailContentZoom: number
  allowRemoteContent: boolean
  screenSize: ScreenSize
  recipientsCollapsed: boolean
  recipientsHeight: number
}

export const useMailLayoutStore = defineStore('mailLayout', {
  state: (): MailLayoutState => ({
    inboxWidth: DEFAULT_INBOX_WIDTH,
    sidebarWidth: DEFAULT_SIDEBAR_WIDTH,
    mailContentZoom: 1.0,
    allowRemoteContent: true,
    screenSize: ScreenSize.Desktop,
    recipientsCollapsed: false,
    recipientsHeight: DEFAULT_RECIPIENTS_HEIGHT,
  }),
  persist: true,
})
