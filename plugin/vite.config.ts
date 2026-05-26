import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { viteSingleFile } from "vite-plugin-singlefile";

export default defineConfig({
  plugins: [svelte(), viteSingleFile()],
  root: "./src/ui",
  build: {
    target: "es2015",
    cssCodeSplit: false,
    outDir: "../../dist",
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
    },
    emptyOutDir: true,
  },
});
