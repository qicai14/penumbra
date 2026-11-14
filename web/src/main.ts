import { mount } from "svelte";
import "../../design/tokens.css";
import App from "./App.svelte";

mount(App, { target: document.getElementById("app")! });
