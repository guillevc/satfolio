import { mount } from "svelte";
import App from "./App.svelte";
import { initLocale } from "$lib/utils/locale";

(async () => {
  await initLocale();
  mount(App, { target: document.getElementById("app")! });
})();
