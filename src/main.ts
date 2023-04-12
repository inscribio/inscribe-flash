import { createApp } from "vue";
import { Icon } from "@iconify/vue";
import { createPinia } from "pinia";
import { createVfm } from "vue-final-modal";
import "vue-final-modal/style.css";
import Toast from "vue-toastification";
import "vue-toastification/dist/index.css";
import App from "./App.vue";
import "@/styles.css";

createApp(App)
  .component("Icon", Icon)
  .use(Toast, {
    hideProgressBar: true,
    newestOnTop: false,
    timeout: 6000,
    transition: "Vue-Toastification__fade",
    container: document.getElementById("app"),
  })
  .use(createPinia())
  .use(createVfm())
  .mount("#app");
