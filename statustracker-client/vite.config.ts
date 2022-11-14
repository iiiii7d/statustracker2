import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import autoprefixer from "autoprefixer";
import postcssScss from "postcss-scss";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  base: "/statustracker2/",
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version),
  },
  css: {
    postcss: {
      syntax: postcssScss,
      plugins: [autoprefixer({})],
    },
  },
});
