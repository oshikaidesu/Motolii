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
      input: {
        host: fileURLToPath(new URL("./host.html", import.meta.url)),
        inspector: fileURLToPath(new URL("./inspector.html", import.meta.url)),
        stageHeader: fileURLToPath(new URL("./stage-header.html", import.meta.url)),
        stageTransport: fileURLToPath(new URL("./stage-transport.html", import.meta.url)),
        timelineTools: fileURLToPath(new URL("./timeline-tools.html", import.meta.url)),
      },
    },
  },
});
