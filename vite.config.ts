import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // Prevent Vite from obscuring Rust errors
  clearScreen: false,

  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // Tell Vite to ignore watching the src-tauri directory
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    // Tauri uses Chromium on Windows and Webkit on macOS/Linux
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      input: {
        settings: resolve(__dirname, "src/settings/index.html"),
        overlay: resolve(__dirname, "src/overlay/index.html"),
      },
    },
    outDir: "dist",
  },
});
