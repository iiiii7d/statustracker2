import App from "./App.svelte";
import "./scss/index.scss";

const app = new App({
  target: document.getElementById("app")!!,
});

export default app;
