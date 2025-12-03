import { defineStore } from 'pinia'

type SearchState = {
  query: string
}

export const useSearchStore = defineStore('search', {
  state: (): SearchState => ({
    query: '',
  }),
  persist: true,
})
