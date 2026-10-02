import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// No dev proxy on purpose: the app talks to the API cross-origin (VITE_API_BASE_URL),
// exactly like a Tauri webview will, so CORS problems show up in dev.
export default defineConfig({
  plugins: [react()],
  server: { port: 5173, strictPort: true },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    css: { modules: { classNameStrategy: "non-scoped" } },
  },
});
