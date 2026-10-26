import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { load } from "@tauri-apps/plugin-store";

export interface ProjectInfo {
  root: string;
  name: string;
  configFile: string;
  hugoVersion: string | null;
}

export interface RecentProject {
  root: string;
  name: string;
}

const STORE_FILE = "projects.json";
const RECENT_KEY = "recentProjects";
const MAX_RECENT = 8;

export const useProjectStore = defineStore("project", () => {
  const info = ref<ProjectInfo | null>(null);
  const recent = ref<RecentProject[]>([]);
  const error = ref<string | null>(null);
  const opening = ref(false);

  async function open(path: string) {
    opening.value = true;
    error.value = null;

    try {
      const project = await invoke<ProjectInfo>("open_project", { path });
      info.value = project;
      await remember(project);
      return project;
    } catch (cause) {
      info.value = null;
      error.value = String(cause);
      throw cause;
    } finally {
      opening.value = false;
    }
  }

  async function remember(project: ProjectInfo) {
    const next = [
      { root: project.root, name: project.name },
      ...recent.value.filter((item) => item.root !== project.root),
    ].slice(0, MAX_RECENT);

    recent.value = next;
    await persist(next);
  }

  async function loadRecent() {
    try {
      const store = await load(STORE_FILE);
      recent.value = (await store.get<RecentProject[]>(RECENT_KEY)) ?? [];
    } catch {
      recent.value = [];
    }
  }

  async function persist(list: RecentProject[]) {
    try {
      const store = await load(STORE_FILE);
      await store.set(RECENT_KEY, list);
      await store.save();
    } catch {
      // 最近项目只是便利功能，持久化失败不影响打开项目。
    }
  }

  return { info, recent, error, opening, open, loadRecent };
});
