import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  build: {
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('@codemirror/') || id.includes('@lezer/')) {
            return 'codemirror';
          }
          if (id.includes('/react/') || id.includes('/react-dom/') || id.includes('dompurify')) {
            return 'vendor';
          }
        },
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
});
