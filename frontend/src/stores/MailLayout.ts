import { defineStore } from 'pinia'

type MailLayoutState = {
  inboxWidth: number
  sidebarCollapsed: boolean
}

export const useMailLayoutStore = defineStore('mailLayout', {
  state: (): MailLayoutState => ({
    inboxWidth: 350,
    sidebarCollapsed: false,
  }),
  persist: true,
})
