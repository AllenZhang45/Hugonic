import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

export type PreviewStatus = "idle" | "starting" | "running" | "error";
export type PreviewDevice = "desktop" | "mobile";

export interface PreviewInfo {
  url: string;
  port: number;
}

export const usePreviewStore = defineStore("preview", () => {
  const status = ref<PreviewStatus>("idle");
  const url = ref<string | null>(null);
  const port = ref<number | null>(null);
  const error = ref<string | null>(null);
  const device = ref<PreviewDevice>("desktop");
  const reloadToken = ref(0);

  const running = computed(() => status.value === "running");

  async function start() {
    if (status.value === "starting" || status.value === "running") {
      return;
    }

    status.value = "starting";
    error.value = null;

    try {
      const info = await invoke<PreviewInfo>("start_preview");
      url.value = info.url;
      port.value = info.port;
      status.value = "running";
      reloadToken.value += 1;
    } catch (cause) {
      status.value = "error";
      url.value = null;
      port.value = null;
      error.value = String(cause);
    }
  }

  async function stop() {
    try {
      await invoke("stop_preview");
    } catch {
      // 后端可能已经没有进程，忽略即可。
    }
    reset();
  }

  function reset() {
    status.value = "idle";
    url.value = null;
    port.value = null;
    error.value = null;
  }

  function toggleDevice() {
    device.value = device.value === "desktop" ? "mobile" : "desktop";
  }

  function reload() {
    reloadToken.value += 1;
  }

  async function openExternal() {
    if (url.value) {
      await openUrl(url.value);
    }
  }

  return {
    status,
    url,
    port,
    error,
    device,
    reloadToken,
    running,
    start,
    stop,
    reset,
    toggleDevice,
    reload,
    openExternal,
  };
});
