import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface BuildLog {
  line: string;
  stream: "stdout" | "stderr";
}

export interface BuildSummary {
  pages: number | null;
  staticFiles: number | null;
  processedImages: number | null;
  durationMs: number | null;
}

export interface BuildDone {
  code: number | null;
  durationMs: number;
  cancelled: boolean;
  summary: BuildSummary;
}

const MAX_LOGS = 5000;

export const useBuildStore = defineStore("build", () => {
  const running = ref(false);
  const logs = ref<BuildLog[]>([]);
  const summary = ref<BuildSummary | null>(null);
  const durationMs = ref<number | null>(null);
  const exitCode = ref<number | null>(null);
  const cancelled = ref(false);
  const error = ref<string | null>(null);
  const modalOpen = ref(false);

  let listening = false;

  async function ensureListeners() {
    if (listening) {
      return;
    }
    listening = true;

    const unsubscribeLog = await listen<BuildLog>("build://log", (event) => {
      logs.value.push(event.payload);
      if (logs.value.length > MAX_LOGS) {
        logs.value.splice(0, logs.value.length - MAX_LOGS);
      }
    });
    const unsubscribeDone = await listen<BuildDone>("build://done", (event) => {
      const payload = event.payload;
      running.value = false;
      summary.value = payload.summary;
      durationMs.value = payload.durationMs;
      exitCode.value = payload.code;
      cancelled.value = payload.cancelled;
    });

    unlisteners.push(unsubscribeLog, unsubscribeDone);
  }

  const unlisteners: UnlistenFn[] = [];

  async function start() {
    if (running.value) {
      return;
    }

    await ensureListeners();

    logs.value = [];
    summary.value = null;
    durationMs.value = null;
    exitCode.value = null;
    cancelled.value = false;
    error.value = null;
    modalOpen.value = true;
    running.value = true;

    try {
      await invoke("start_build");
    } catch (cause) {
      running.value = false;
      error.value = String(cause);
    }
  }

  async function cancel() {
    try {
      await invoke("cancel_build");
    } catch {
      // 取消失败时保留运行状态，由 done 事件收尾。
    }
  }

  async function copyLogs(): Promise<boolean> {
    const text = logs.value.map((log) => log.line).join("\n");

    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      const area = document.createElement("textarea");
      area.value = text;
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.appendChild(area);
      area.select();
      const copied = document.execCommand("copy");
      area.remove();
      return copied;
    }
  }

  async function openOutput() {
    try {
      await invoke("open_build_output");
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function openModal() {
    modalOpen.value = true;
  }

  function closeModal() {
    modalOpen.value = false;
  }

  return {
    running,
    logs,
    summary,
    durationMs,
    exitCode,
    cancelled,
    error,
    modalOpen,
    start,
    cancel,
    copyLogs,
    openOutput,
    openModal,
    closeModal,
  };
});
