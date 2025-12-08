import { ref } from 'vue'
import { copyToClipboard } from '@/helpers/copy'

export function useCopy(getText: () => string) {
  const copied = ref(false)

  const handleCopy = async () => {
    await copyToClipboard(getText())
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  }

  return {
    copied,
    handleCopy,
  }
}
