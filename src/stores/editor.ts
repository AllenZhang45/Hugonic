import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface FileContent {
  path: string;
  content: string;
  size: number;
  mtimeMs: number;
}

export interface FileMeta {
  size: number;
  mtimeMs: number;
}

export interface EditorTab {
  path: string;
  name: string;
  language: string;
  content: string;
  savedContent: string;
  mtimeMs: number;
  size: number;
  conflict: boolean;
}

const LANGUAGE_BY_EXTENSION: Record<string, string> = {
  md: "markdown",
  markdown: "markdown",
  mdx: "markdown",
  toml: "ini",
  yaml: "yaml",
  yml: "yaml",
  html: "html",
  htm: "html",
  css: "css",
  scss: "scss",
  js: "javascript",
  mjs: "javascript",
  ts: "typescript",
  json: "json",
  go: "go",
  sh: "shell",
  xml: "xml",
  svg: "xml",
};

export function languageForPath(path: string): string {
  const extension = path.split(".").pop()?.toLowerCase() ?? "";
  return LANGUAGE_BY_EXTENSION[extension] ?? "plaintext";
}

export function baseName(path: string): string {
  return path.split("/").pop() ?? path;
}

export function isDirty(tab: EditorTab): boolean {
  return tab.content !== tab.savedContent;
}

export const useEditorStore = defineStore("editor", () => {
  const tabs = ref<EditorTab[]>([]);
  const activePath = ref<string | null>(null);
  const cursor = ref({ line: 1, column: 1 });
  const error = ref<string | null>(null);
  const saving = ref(false);

  const activeTab = computed(() => tabs.value.find((tab) => tab.path === activePath.value) ?? null);

  async function openFile(path: string) {
    const existing = tabs.value.find((tab) => tab.path === path);
    if (existing) {
      activePath.value = path;
      await checkExternal(path);
      return;
    }

    try {
      const file = await invoke<FileContent>("read_file", { path });
      tabs.value = [
        ...tabs.value,
        {
          path,
          name: baseName(path),
          language: languageForPath(path),
          content: file.content,
          savedContent: file.content,
          mtimeMs: file.mtimeMs,
          size: file.size,
          conflict: false,
        },
      ];
      activePath.value = path;
      error.value = null;
    } catch (cause) {
      error.value = String(cause);
    }
  }

  function activate(path: string) {
    activePath.value = path;
    void checkExternal(path);
  }

  function updateContent(path: string, content: string) {
    tabs.value = tabs.value.map((tab) => (tab.path === path ? { ...tab, content } : tab));
  }

  async function save(path: string | null = activePath.value) {
    if (!path || saving.value) {
      return false;
    }

    const tab = tabs.value.find((item) => item.path === path);
    if (!tab || !isDirty(tab)) {
      return false;
    }

    saving.value = true;
    error.value = null;

    try {
      const meta = await invoke<FileMeta>("save_file", { path, content: tab.content });
      tabs.value = tabs.value.map((item) =>
        item.path === path
          ? {
              ...item,
              savedContent: item.content,
              mtimeMs: meta.mtimeMs,
              size: meta.size,
              conflict: false,
            }
          : item,
      );
      return true;
    } catch (cause) {
      error.value = String(cause);
      return false;
    } finally {
      saving.value = false;
    }
  }

  async function checkExternal(path: string) {
    const tab = tabs.value.find((item) => item.path === path);
    if (!tab) {
      return;
    }

    try {
      const meta = await invoke<FileMeta>("file_meta", { path });
      if (meta.mtimeMs === tab.mtimeMs) {
        return;
      }

      if (isDirty(tab)) {
        tabs.value = tabs.value.map((item) =>
          item.path === path ? { ...item, conflict: true } : item,
        );
        return;
      }

      await reload(path);
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function reload(path: string) {
    try {
      const file = await invoke<FileContent>("read_file", { path });
      tabs.value = tabs.value.map((item) =>
        item.path === path
          ? {
              ...item,
              content: file.content,
              savedContent: file.content,
              mtimeMs: file.mtimeMs,
              size: file.size,
              conflict: false,
            }
          : item,
      );
      error.value = null;
    } catch (cause) {
      error.value = String(cause);
    }
  }

  async function resolveConflict(path: string, strategy: "reload" | "keep") {
    if (strategy === "reload") {
      await reload(path);
      return;
    }

    tabs.value = tabs.value.map((tab) => (tab.path === path ? { ...tab, conflict: false } : tab));
  }

  async function close(path: string) {
    const index = tabs.value.findIndex((tab) => tab.path === path);
    if (index === -1) {
      return;
    }

    const remaining = tabs.value.filter((tab) => tab.path !== path);
    tabs.value = remaining;

    if (activePath.value === path) {
      activePath.value = remaining[Math.min(index, remaining.length - 1)]?.path ?? null;
    }
  }

  function setCursor(line: number, column: number) {
    cursor.value = { line, column };
  }

  return {
    tabs,
    activePath,
    activeTab,
    cursor,
    error,
    saving,
    openFile,
    activate,
    updateContent,
    save,
    checkExternal,
    reload,
    resolveConflict,
    close,
    setCursor,
  };
});
