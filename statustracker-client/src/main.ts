import App from "./sections/App.svelte";
import "./scss/index.scss";
import { mount } from "svelte";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
