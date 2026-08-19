import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Port 1420 is Tauri's expected dev-server port (see src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    // WebKitGTK is the floor (ARCHITECTURE §2): keep output conservative.
    target: ["es2022", "safari15"],
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
