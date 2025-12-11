import { defineStore } from 'pinia'

export const ScreenSize = {
  Mobile: 'mobile',
  Tablet: 'tablet',
  Desktop: 'desktop',
} as const
export type ScreenSize = (typeof ScreenSize)[keyof typeof ScreenSize]

type MailLayoutState = {
  inboxWidth: number
  sidebarCollapsed: boolean
  mailContentZoom: number
  allowRemoteContent: boolean
  screenSize: ScreenSize | null
}

export const useMailLayoutStore = defineStore('mailLayout', {
  state: (): MailLayoutState => ({
    inboxWidth: 350,
    sidebarCollapsed: false,
    mailContentZoom: 1.0,
    allowRemoteContent: true,
    screenSize: null,
  }),
  persist: true,
})
