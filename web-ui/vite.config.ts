/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: './src/test/setup.ts',
  },
  server: {
    port: 5173,
    proxy: {
      // Proxy API requests to vibes daemon
      '/api': {
        target: 'http://localhost:7743',
        changeOrigin: true,
      },
      // Proxy WebSocket connections
      '/ws': {
        target: 'ws://localhost:7743',
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
})
