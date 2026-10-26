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

  let healthTimer: ReturnType<typeof setInterval> | null = null;

  function stopHealthCheck() {
    if (healthTimer !== null) {
      clearInterval(healthTimer);
      healthTimer = null;
    }
  }

  function startHealthCheck() {
    stopHealthCheck();
    healthTimer = setInterval(() => {
      void checkAlive();
    }, 2000);
  }

  /** 探测 hugo 进程是否还活着；已退出时把状态切到错误，避免 UI 一直显示绿色。 */
  async function checkAlive(): Promise<boolean> {
    if (status.value !== "running") {
      return false;
    }

    try {
      const alive = await invoke<boolean>("preview_alive");
      if (!alive) {
        status.value = "error";
        error.value = "hugo server 已退出，预览不可用";
        stopHealthCheck();
      }
      return alive;
    } catch {
      return false;
    }
  }

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
      startHealthCheck();
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
    stopHealthCheck();
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
    if (!url.value) {
      return;
    }
    if (!(await checkAlive())) {
      return;
    }
    await openUrl(url.value);
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
