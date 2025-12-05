import { defineStore } from 'pinia'

type MailLayoutState = {
  inboxWidth: number
  sidebarCollapsed: boolean
  mailContentZoom: number
  blockExternalRequests: boolean
}

export const useMailLayoutStore = defineStore('mailLayout', {
  state: (): MailLayoutState => ({
    inboxWidth: 350,
    sidebarCollapsed: false,
    mailContentZoom: 1.0,
    blockExternalRequests: true,
  }),
  persist: true,
})
