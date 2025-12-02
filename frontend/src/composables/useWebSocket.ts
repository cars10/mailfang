import { onUnmounted } from 'vue'

export interface WebSocketCallbacks {
  onMessage?: (event: MessageEvent) => void
  onOpen?: () => void
  onError?: (error: Event) => void
  onClose?: () => void
}

export interface UseWebSocketOptions {
  url?: string
  reconnectInterval?: number
  autoConnect?: boolean
}

export function useWebSocket(
  callbacks: WebSocketCallbacks = {},
  options: UseWebSocketOptions = {}
) {
  const { url, reconnectInterval = 3000, autoConnect = true } = options

  let ws: WebSocket | null = null
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null

  const getWebSocketUrl = (): string => {
    if (url) {
      return url
    }
    // Use current origin for WebSocket connection (works with Vite proxy in dev and direct in prod)
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${protocol}//${window.location.host}/ws`
  }

  const connect = () => {
    if (ws?.readyState === WebSocket.OPEN) {
      return
    }

    const wsUrl = getWebSocketUrl()
    ws = new WebSocket(wsUrl)

    ws.onopen = () => {
      console.log('WebSocket connected')
      callbacks.onOpen?.()
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout)
        reconnectTimeout = null
      }
    }

    ws.onmessage = event => {
      callbacks.onMessage?.(event)
    }

    ws.onerror = error => {
      console.error('WebSocket error:', error)
      callbacks.onError?.(error)
    }

    ws.onclose = () => {
      console.log('WebSocket disconnected')
      callbacks.onClose?.()
      ws = null

      // Attempt to reconnect after interval
      if (reconnectInterval > 0) {
        reconnectTimeout = setTimeout(() => {
          connect()
        }, reconnectInterval)
      }
    }
  }

  const disconnect = () => {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }
    if (ws) {
      ws.close()
      ws = null
    }
  }

  const send = (data: string | ArrayBuffer | Blob) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(data)
    } else {
      console.warn('WebSocket is not connected')
    }
  }

  const checkAndReconnect = () => {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      console.log('WebSocket not connected, attempting to reconnect...')
      connect()
    }
  }

  const handleVisibilityChange = () => {
    if (document.visibilityState === 'visible') {
      checkAndReconnect()
    }
  }

  if (autoConnect) {
    connect()
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', handleVisibilityChange)
  }

  onUnmounted(() => {
    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
    disconnect()
  })

  return {
    connect,
    disconnect,
    send,
    isConnected: () => ws?.readyState === WebSocket.OPEN,
  }
}
