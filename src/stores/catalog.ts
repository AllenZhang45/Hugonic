import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface CatalogTheme {
  slug: string;
  name: string;
  description: string;
  tags: string[];
  license: string | null;
  stars: number | null;
  updated: string | null;
  repo: string;
  demo: string | null;
  screenshotUrl: string | null;
  installCommands: string[];
  installExtracted: boolean;
  installSource: string | null;
}

export interface CatalogCache {
  schema: number;
  fetchedAt: number;
  complete: boolean;
  source: string;
  themes: CatalogTheme[];
}

export interface SyncProgress {
  phase: string;
  done: number;
  total: number;
  failed: number;
  message: string;
}

const TTL_SECONDS = 7 * 24 * 60 * 60;
const ACTIVE_PHASES = new Set(["starting", "catalog", "details"]);

export const useCatalogStore = defineStore("catalog", () => {
  const data = ref<CatalogCache | null>(null);
  const progress = ref<SyncProgress | null>(null);
  const running = ref(false);
  const error = ref<string | null>(null);
  const screenshots = ref<Record<string, string>>({});
  let initialized = false;
  let unlisten: UnlistenFn | null = null;

  const stale = computed(() => {
    if (!data.value?.complete) {
      return true;
    }
    return Date.now() / 1000 - data.value.fetchedAt > TTL_SECONDS;
  });

  async function bindEvents() {
    if (unlisten) {
      return;
    }

    const offProgress = await listen<SyncProgress>("theme-catalog://progress", (event) => {
      progress.value = event.payload;
      running.value = ACTIVE_PHASES.has(event.payload.phase);
      if (event.payload.phase === "error") {
        error.value = event.payload.message;
      }
    });

    const offDone = await listen<{ ok: boolean; error?: string }>(
      "theme-catalog://done",
      async (event) => {
        running.value = false;
        if (event.payload.ok) {
          await load();
        } else {
          error.value = event.payload.error ?? "同步失败";
        }
      },
    );

    unlisten = () => {
      offProgress();
      offDone();
    };
  }

  async function load() {
    try {
      data.value = await invoke<CatalogCache | null>("load_theme_catalog");
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function start(force = false) {
    error.value = null;
    try {
      const started = await invoke<boolean>("start_theme_sync", { force });
      if (started) {
        running.value = true;
      } else {
        const active = await invoke<boolean>("theme_catalog_running");
        running.value = active;
      }
      return started;
    } catch (cause) {
      error.value = String(cause);
      return false;
    }
  }

  async function init() {
    await bindEvents();
    await load();

    const active = await invoke<boolean>("theme_catalog_running");
    running.value = active;
    if (active) {
      progress.value = await invoke<SyncProgress>("theme_catalog_progress");
      return;
    }

    if (initialized) {
      return;
    }
    initialized = true;

    if (!data.value?.complete || stale.value) {
      await start(false);
    }
  }

  async function loadScreenshot(slug: string) {
    if (screenshots.value[slug] !== undefined) {
      return screenshots.value[slug];
    }
    try {
      const url = await invoke<string | null>("catalog_screenshot", { slug });
      if (url) {
        screenshots.value = { ...screenshots.value, [slug]: url };
        return url;
      }
    } catch {
      // 截图失败不阻塞详情展示。
    }
    return null;
  }

  function reset() {
    data.value = null;
    progress.value = null;
    running.value = false;
    error.value = null;
    screenshots.value = {};
    initialized = false;
  }

  return {
    data,
    progress,
    running,
    error,
    screenshots,
    stale,
    init,
    load,
    start,
    loadScreenshot,
    reset,
  };
});
