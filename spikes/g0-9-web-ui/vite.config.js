import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const publicId = "virtual:g0-9-hmr-probe";
const resolvedId = `\0${publicId}`;

function hmrProbe() {
  let revision = 0;

  return {
    name: "g0-9-hmr-probe",
    resolveId(id) {
      return id === publicId ? resolvedId : undefined;
    },
    load(id) {
      return id === resolvedId ? `export const revision = ${revision};` : undefined;
    },
    configureServer(server) {
      server.middlewares.use("/__g0_9_hmr_tick", async (_request, response) => {
        revision += 1;
        const module = server.moduleGraph.getModuleById(resolvedId);
        if (!module) {
          response.statusCode = 409;
          response.end("probe module is not loaded");
          return;
        }
        server.moduleGraph.invalidateModule(module);
        await server.reloadModule(module);
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({ revision }));
      });
    },
  };
}

export default defineConfig({
  plugins: [react(), hmrProbe()],
  server: {
    host: "127.0.0.1",
    port: 4179,
    strictPort: true,
  },
});
