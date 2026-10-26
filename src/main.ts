import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "virtual:uno.css";
import "./styles/tokens.css";

createApp(App).use(createPinia()).mount("#app");
