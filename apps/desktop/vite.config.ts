import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  build: {
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks: {
          codemirror: [
            "@codemirror/lang-markdown",
            "@codemirror/lang-rust",
            "@codemirror/state",
            "@codemirror/theme-one-dark",
            "@codemirror/view",
            "@lezer/common",
            "@lezer/highlight",
            "@lezer/lr",
            "@lezer/markdown",
            "@lezer/rust",
          ],
          vendor: ["react", "react-dom", "dompurify"],
        },
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
});
