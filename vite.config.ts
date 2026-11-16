import { defineConfig } from "vite";

// Minimal Vite config for Open-Muse.
// The production window loads https://muse.ai directly (see tauri.conf.json),
// so this local frontend is only a small placeholder shown in `tauri dev`.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2021",
    outDir: "dist",
  },
});
