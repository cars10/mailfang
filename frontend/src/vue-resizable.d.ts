declare module 'vue-resizable' {
  import { DefineComponent } from 'vue'

  export interface ResizableProps {
    width?: number | string
    height?: number | string
    minWidth?: number
    minHeight?: number
    maxWidth?: number
    maxHeight?: number
    active?: string[]
    lockAspectRatio?: boolean
    dragSelector?: string
    handles?: string[]
  }

  const VueResizable: DefineComponent<ResizableProps>
  export default VueResizable
}
