import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { initNav } from "./lib/nav.svelte";

initNav();

const target = document.getElementById("app");
if (!target) throw new Error("#app mount point missing");

export default mount(App, { target });
