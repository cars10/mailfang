import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'
import svgLoader from 'vite-svg-loader'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss(), svgLoader()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    allowedHosts: ['frontend'],
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://backend:3000',
        changeOrigin: true,
      },
      '/ws': {
        target: 'http://backend:3000',
        ws: true,
        changeOrigin: true,
      },
    },
  },
})
