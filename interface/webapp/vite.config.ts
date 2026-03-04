import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/lifelog.LifelogServerService': {
        target: 'https://100.118.206.104:7182',
        changeOrigin: true,
        secure: false,
        ws: true,
      },
      '/media': {
        target: 'http://100.118.206.104:7183',
        changeOrigin: true,
        secure: false,
      }
    }
  }
})
