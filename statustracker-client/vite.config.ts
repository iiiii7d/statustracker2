import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import topLevelAwait from "vite-plugin-top-level-await";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    topLevelAwait(),
  ],
  base: "/statustracker/",
  define: {
    '__APP_VERSION__': JSON.stringify(process.env.npm_package_version),
  }
})