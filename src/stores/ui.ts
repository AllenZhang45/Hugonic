import { defineStore } from "pinia";
import { ref } from "vue";

export type ViewId = "content" | "dashboard" | "media" | "themes" | "analytics";
export type PreviewDevice = "desktop" | "mobile";

export const useUiStore = defineStore("ui", () => {
  const activeView = ref<ViewId>("content");
  const previewDevice = ref<PreviewDevice>("desktop");

  function setActiveView(view: ViewId) {
    activeView.value = view;
  }

  function togglePreviewDevice() {
    previewDevice.value = previewDevice.value === "desktop" ? "mobile" : "desktop";
  }

  return { activeView, previewDevice, setActiveView, togglePreviewDevice };
});
