// Dev-only: serves dev-mock/database-views-harness.html, which renders the database views (board, calendar,
// timeline, list, gallery, chart) with hand-made data in a plain browser. Not used by the real app or its build.
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";

const p = (f: string) => fileURLToPath(new URL(f, import.meta.url));

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@tauri-apps/api/core": p("./dev-mock/views-core.ts"),
      "@tauri-apps/api/event": p("./dev-mock/event.ts"),
      "@tauri-apps/api/window": p("./dev-mock/window.ts"),
      "@tauri-apps/api/webviewWindow": p("./dev-mock/webviewWindow.ts"),
      "@": p("./src"),
    },
  },
  server: { port: 1423, strictPort: true, host: "127.0.0.1" },
});
