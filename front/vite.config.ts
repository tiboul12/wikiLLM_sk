// Front du wiki : compilé dans dist/, embarqué par le binaire wikik (rust-embed).
// En développement, `npm run dev` relaie /api vers `wikik serve` lancé sur le port 8484.
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  build: { outDir: "dist", emptyOutDir: true, chunkSizeWarningLimit: 4000 },
  server: { proxy: { "/api": "http://127.0.0.1:8484" } },
});
