import { defineConfig } from "vite";
import path from "node:path";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  resolve: {
    alias: {
      ai_wasm: path.resolve(__dirname, "../ai_wasm/pkg"),
    },
  },
  server: {
    fs: {
      allow: [
        ".",
        "../"
      ],
    },
  },
  optimizeDeps: {
    exclude: ["ai_wasm"],
  },
});
