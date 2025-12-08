import { defineStore } from 'pinia'

export enum ScreenSize {
  Mobile = 'mobile',
  Tablet = 'tablet',
  Desktop = 'desktop',
}

type MailLayoutState = {
  inboxWidth: number
  sidebarCollapsed: boolean
  mailContentZoom: number
  blockExternalRequests: boolean
  screenSize: ScreenSize | null
}

export const useMailLayoutStore = defineStore('mailLayout', {
  state: (): MailLayoutState => ({
    inboxWidth: 350,
    sidebarCollapsed: false,
    mailContentZoom: 1.0,
    blockExternalRequests: false,
    screenSize: null,
  }),
  persist: true,
})
