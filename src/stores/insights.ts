import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "./project";

export interface Counts {
  pages: number;
  posts: number;
  drafts: number;
  media: number;
  themes: number;
}

export interface SectionStat {
  name: string;
  pages: number;
  drafts: number;
  words: number;
}

export interface FacetStat {
  name: string;
  count: number;
}

export interface RecentFile {
  path: string;
  name: string;
  modifiedMs: number;
}

export interface Insights {
  configFile: string;
  title: string | null;
  baseUrl: string | null;
  theme: string | null;
  language: string | null;
  counts: Counts;
  words: number;
  sections: SectionStat[];
  tags: FacetStat[];
  categories: FacetStat[];
  recent: RecentFile[];
  missingTitle: number;
  missingDate: number;
}

export const useInsightsStore = defineStore("insights", () => {
  const data = ref<Insights | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  let loadedRoot: string | null = null;

  async function load(force = false) {
    const project = useProjectStore();
    if (!project.info) {
      data.value = null;
      loadedRoot = null;
      error.value = null;
      return;
    }

    if (!force && data.value && loadedRoot === project.info.root) {
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      data.value = await invoke<Insights>("project_insights");
      loadedRoot = project.info.root;
    } catch (cause) {
      data.value = null;
      loadedRoot = null;
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  function reset() {
    data.value = null;
    loadedRoot = null;
    error.value = null;
  }

  return { data, loading, error, load, reset };
});
