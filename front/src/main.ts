// Point d'entrée du front : monte l'application sur #app.
import { mount } from "svelte";
import App from "./App.svelte";
import "./styles.css";

mount(App, { target: document.getElementById("app")! });
