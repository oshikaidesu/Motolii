import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  plugins: [react()],
  build: {
    emptyOutDir: true,
    manifest: true,
    outDir: "generated-host",
    rollupOptions: {
      input: fileURLToPath(new URL("./host.html", import.meta.url)),
    },
  },
});
