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
    rollupOptions: {
      output: {
        manualChunks: {
          // Core React - rarely changes, cache separately
          'vendor-react': ['react', 'react-dom'],
          // TanStack libraries - routing and data fetching
          'vendor-tanstack': ['@tanstack/react-query', '@tanstack/react-router'],
          // Charting library - only needed on dashboard pages
          'vendor-visx': [
            '@visx/axis',
            '@visx/group',
            '@visx/responsive',
            '@visx/scale',
            '@visx/shape',
          ],
          // Terminal emulator - only needed on session pages
          'vendor-xterm': ['@xterm/xterm', '@xterm/addon-fit', '@xterm/addon-web-links'],
        },
      },
    },
  },
})
