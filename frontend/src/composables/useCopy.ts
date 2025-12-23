import { ref } from 'vue'
import { copyToClipboard } from '@/helpers/copy'

export function useCopy(getText: () => string, timeout: number = 2000) {
  const copied = ref(false)

  const handleCopy = async () => {
    await copyToClipboard(getText())
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, timeout)
  }

  return {
    copied,
    handleCopy,
  }
}
