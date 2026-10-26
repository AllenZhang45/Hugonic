import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type EntryKind = "dir" | "file";

export interface Entry {
  name: string;
  /** 相对项目根目录的路径 */
  path: string;
  kind: EntryKind;
}

export const useFilesStore = defineStore("files", () => {
  /** path -> 该层目录的子项；按需加载 */
  const entries = ref<Record<string, Entry[]>>({});
  const expanded = ref<string[]>([]);
  const selected = ref<string | null>(null);
  const loading = ref<string[]>([]);
  const error = ref<string | null>(null);

  async function loadDir(path: string, force = false) {
    if (!force && entries.value[path]) {
      return;
    }

    loading.value = [...loading.value, path];
    error.value = null;

    try {
      const children = await invoke<Entry[]>("read_dir", { path });
      entries.value = { ...entries.value, [path]: children };
    } catch (cause) {
      error.value = String(cause);
      throw cause;
    } finally {
      loading.value = loading.value.filter((item) => item !== path);
    }
  }

  async function toggleDir(path: string) {
    if (expanded.value.includes(path)) {
      expanded.value = expanded.value.filter((item) => item !== path);
      return;
    }

    await loadDir(path);
    expanded.value = [...expanded.value, path];
  }

  function select(path: string) {
    selected.value = path;
  }

  function reset() {
    entries.value = {};
    expanded.value = [];
    selected.value = null;
    loading.value = [];
    error.value = null;
  }

  return { entries, expanded, selected, loading, error, loadDir, toggleDir, select, reset };
});
