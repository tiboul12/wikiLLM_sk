// Point d'entrée du front : monte l'application sur #app.
// Les modules du front sont documentés comme le reste du dépôt (JSDoc, balises wikik).
//
// @req EF-075
import { mount } from "svelte";
import App from "./App.svelte";
import "./styles.css";

mount(App, { target: document.getElementById("app")! });
