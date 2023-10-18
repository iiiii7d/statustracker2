/* eslint-env node */
import * as esbuild from "esbuild";
import autoprefixer from "autoprefixer";
import postcssPresetEnv from "postcss-preset-env";
import { sassPlugin } from "esbuild-sass-plugin";
import * as fs from "fs";
import postcss from "postcss";
import sveltePlugin from "esbuild-svelte";
import sveltePreprocess from "svelte-preprocess";

const postcssPlugins = [autoprefixer(), postcssPresetEnv({ stage: 0 })];

const ctx = await esbuild.context({
  entryPoints: ["src/main.ts"],
  bundle: true,
  minify: true,
  sourcemap: true,
  outfile: "out/out.js",
  mainFields: ["svelte", "browser", "module", "main"],
  conditions: ["svelte", "browser"],
  publicPath:
    process.argv[2] == "prod"
      ? "https://iiiii7d.github.io/statustracker2"
      : undefined,
  plugins: [
    sveltePlugin({
      preprocess: sveltePreprocess({
        postcss: {
          plugins: postcssPlugins,
        },
      }),
    }),
    sassPlugin({
      async transform(source) {
        const { css } = await postcss(postcssPlugins).process(source, {
          from: undefined,
        });
        return css;
      },
    }),
  ],
  loader: {
    ".md": "text",
  },
  define: {
    __APP_VERSION__: `"${process.env.npm_package_version}"` ?? "???",
  },
});
if (!fs.existsSync("out")) fs.mkdirSync("out");
fs.copyFileSync("./index.html", "./out/index.html");

if (process.argv[2] == "prod") {
  await ctx.rebuild();
  process.exit();
}

await ctx.watch();

const { host, port } = await ctx.serve({
  servedir: "out",
});
console.log(`http://${host}:${port}`);
