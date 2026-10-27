import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "./project";

export interface MediaItem {
  path: string;
  name: string;
  dir: string;
  size: number;
  modifiedMs: number;
  extension: string;
}

/** 把项目内路径转成适合写进 Markdown 的引用。 */
export function mediaReference(path: string): string {
  if (path.startsWith("static/")) {
    return `/${path.slice("static/".length)}`;
  }
  if (path.startsWith("content/")) {
    return path.split("/").pop() ?? path;
  }
  return path;
}

export const useMediaStore = defineStore("media", () => {
  const items = ref<MediaItem[]>([]);
  const thumbnails = ref<Record<string, string>>({});
  const loading = ref(false);
  const importing = ref(false);
  const error = ref<string | null>(null);
  let loadedRoot: string | null = null;
  const pending = new Set<string>();

  async function load(force = false) {
    const project = useProjectStore();
    if (!project.info) {
      items.value = [];
      loadedRoot = null;
      return;
    }

    if (!force && items.value.length && loadedRoot === project.info.root) {
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      items.value = await invoke<MediaItem[]>("list_media");
      loadedRoot = project.info.root;
    } catch (cause) {
      items.value = [];
      loadedRoot = null;
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  async function ensureThumbnail(item: MediaItem) {
    if (thumbnails.value[item.path] !== undefined || pending.has(item.path)) {
      return;
    }

    pending.add(item.path);
    try {
      const url = await invoke<string>("media_thumbnail", { path: item.path });
      thumbnails.value = { ...thumbnails.value, [item.path]: url };
    } catch {
      thumbnails.value = { ...thumbnails.value, [item.path]: "" };
    } finally {
      pending.delete(item.path);
    }
  }

  async function importPaths(sources: string[], targetDir: string) {
    importing.value = true;
    error.value = null;
    let imported = 0;

    try {
      for (const source of sources) {
        await invoke("import_media", { source, targetDir });
        imported += 1;
      }
      if (imported > 0) {
        await load(true);
      }
      return imported;
    } catch (cause) {
      error.value = String(cause);
      return imported;
    } finally {
      importing.value = false;
    }
  }

  async function open(item: MediaItem) {
    try {
      await invoke("open_media", { path: item.path });
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function reset() {
    items.value = [];
    thumbnails.value = {};
    loadedRoot = null;
    error.value = null;
  }

  return {
    items,
    thumbnails,
    loading,
    importing,
    error,
    load,
    ensureThumbnail,
    importPaths,
    open,
    reset,
  };
});
