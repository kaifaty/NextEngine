import { fileURLToPath, URL } from "node:url";

import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [vue()],
  base: "/",
  build: {
    // AudioWorklet modules must remain same-origin files. Inlining them as a
    // data: URL would require a broader CSP and behaves inconsistently across
    // browsers.
    assetsInlineLimit: 0,
    emptyOutDir: true,
    outDir: fileURLToPath(
      new URL("../src/nextengine_speech_timeline/dashboard_static", import.meta.url),
    ),
    sourcemap: false,
  },
});
