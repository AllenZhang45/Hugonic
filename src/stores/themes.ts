import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "./project";

export interface ThemeInfo {
  dir: string;
  name: string;
  description: string | null;
  author: string | null;
  license: string | null;
  homepage: string | null;
  minVersion: string | null;
  active: boolean;
  hasScreenshot: boolean;
}

export interface ThemeList {
  themes: ThemeInfo[];
  configFile: string;
  canSwitch: boolean;
  reason: string | null;
}

export const useThemesStore = defineStore("themes", () => {
  const data = ref<ThemeList | null>(null);
  const screenshots = ref<Record<string, string>>({});
  const loading = ref(false);
  const applying = ref<string | null>(null);
  const error = ref<string | null>(null);
  let loadedRoot: string | null = null;

  async function load(force = false) {
    const project = useProjectStore();
    if (!project.info) {
      data.value = null;
      loadedRoot = null;
      return;
    }

    if (!force && data.value && loadedRoot === project.info.root) {
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      data.value = await invoke<ThemeList>("list_themes");
      loadedRoot = project.info.root;
      await loadScreenshots();
    } catch (cause) {
      data.value = null;
      loadedRoot = null;
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  async function loadScreenshots() {
    if (!data.value) {
      return;
    }

    await Promise.all(
      data.value.themes
        .filter((theme) => theme.hasScreenshot && !screenshots.value[theme.dir])
        .map(async (theme) => {
          try {
            const url = await invoke<string | null>("theme_screenshot", { dir: theme.dir });
            if (url) {
              screenshots.value = { ...screenshots.value, [theme.dir]: url };
            }
          } catch {
            // 截图只是展示用，失败时保留占位。
          }
        }),
    );
  }

  async function apply(dir: string) {
    applying.value = dir;
    error.value = null;

    try {
      await invoke("apply_theme", { dir });
      await load(true);
      return true;
    } catch (cause) {
      error.value = String(cause);
      return false;
    } finally {
      applying.value = null;
    }
  }

  async function openFolder(dir: string) {
    try {
      await invoke("open_theme_folder", { dir });
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function reset() {
    data.value = null;
    screenshots.value = {};
    loadedRoot = null;
    error.value = null;
  }

  return {
    data,
    screenshots,
    loading,
    applying,
    error,
    load,
    loadScreenshots,
    apply,
    openFolder,
    reset,
  };
});
