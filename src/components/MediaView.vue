<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import MediaCard from "./MediaCard.vue";
import { useMediaStore } from "../stores/media";
import { useProjectStore } from "../stores/project";
import { useUiStore } from "../stores/ui";

const project = useProjectStore();
const media = useMediaStore();
const ui = useUiStore();

const query = ref("");
const filter = ref("all");
const targetDir = ref("static/images");
const message = ref("");

const filters = [
  { value: "all", label: "全部" },
  { value: "image", label: "图片" },
  { value: "png", label: "PNG" },
  { value: "jpg", label: "JPG" },
  { value: "svg", label: "SVG" },
  { value: "gif", label: "GIF" },
  { value: "webp", label: "WebP" },
];

onMounted(() => {
  void media.load();
});

const filtered = computed(() => {
  const needle = query.value.trim().toLowerCase();
  return media.items.filter((item) => {
    const matchesType =
      filter.value === "all" ||
      (filter.value === "image" &&
        ["png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp"].includes(item.extension)) ||
      (filter.value === "jpg" && ["jpg", "jpeg"].includes(item.extension)) ||
      item.extension === filter.value;

    const matchesQuery =
      !needle ||
      item.name.toLowerCase().includes(needle) ||
      item.path.toLowerCase().includes(needle);

    return matchesType && matchesQuery;
  });
});

async function importImages() {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [
      { name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "gif", "svg", "avif", "bmp"] },
    ],
  });

  if (!selected) {
    return;
  }

  const paths = Array.isArray(selected) ? selected : [selected];
  if (!paths.length) {
    return;
  }

  message.value = "";
  const imported = await media.importPaths(paths, targetDir.value);
  if (imported > 0) {
    message.value = `已导入 ${imported} 个文件到 ${targetDir.value}`;
  }
}
</script>

<template>
  <section class="view" aria-label="媒体库">
    <div class="header">
      <h2 class="title">媒体库</h2>
      <span class="subtitle">{{ media.items.length }} 个文件</span>
      <button class="refresh" type="button" :disabled="media.loading" @click="media.load(true)">
        {{ media.loading ? "扫描中…" : "刷新" }}
      </button>
    </div>

    <div v-if="!project.info" class="empty">
      <p>尚未打开项目</p>
      <button type="button" @click="ui.setActiveView('content')">回到内容视图打开站点</button>
    </div>

    <template v-else>
      <div class="toolbar">
        <input v-model="query" class="search" type="search" placeholder="搜索文件名或路径…" />

        <div class="filters">
          <button
            v-for="item in filters"
            :key="item.value"
            type="button"
            class="chip"
            :class="{ active: filter === item.value }"
            @click="filter = item.value"
          >
            {{ item.label }}
          </button>
        </div>

        <div class="import">
          <select v-model="targetDir" class="target" aria-label="导入目录">
            <option value="static/images">static/images</option>
            <option value="assets">assets</option>
          </select>
          <button type="button" class="primary" :disabled="media.importing" @click="importImages">
            {{ media.importing ? "导入中…" : "导入图片" }}
          </button>
        </div>
      </div>

      <p v-if="message" class="message">{{ message }}</p>
      <p v-if="media.error" class="error">{{ media.error }}</p>

      <div v-if="filtered.length" class="grid">
        <MediaCard v-for="item in filtered" :key="item.path" :item="item" />
      </div>

      <div v-else class="empty">
        <p v-if="media.items.length">没有匹配的图片</p>
        <p v-else>还没有图片</p>
        <p class="hint">
          支持 static/、assets/ 与 content bundle；导入会复制到目标目录，不覆盖同名文件。
        </p>
      </div>

      <p v-if="media.items.length >= 1000" class="limit">
        文件较多，当前只展示最近修改的 1000 个。
      </p>
    </template>
  </section>
</template>

<style scoped>
.view {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 22px 24px;
  background: var(--bg-color);
}

.header {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 14px;
}

.title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.subtitle {
  color: var(--text-tertiary);
  font-size: 11px;
}

.refresh {
  margin-left: auto;
  height: 26px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
}

.refresh:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin-bottom: 14px;
}

.search {
  width: 200px;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.search:focus {
  border-color: var(--theme-color);
}

.filters {
  display: flex;
  gap: 4px;
}

.chip {
  height: 26px;
  padding: 0 9px;
  border: 1px solid transparent;
  border-radius: 999px;
  background: transparent;
  color: var(--text-tertiary);
  font-size: 11px;
}

.chip:hover {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.chip.active {
  border-color: var(--border-color);
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.import {
  display: flex;
  gap: 6px;
  margin-left: auto;
}

.target {
  height: 28px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-size: 11px;
}

.primary {
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--theme-color);
  border-radius: var(--radius-sm);
  background: var(--theme-color);
  color: #fff;
  font-size: 12px;
}

.primary:disabled {
  opacity: 0.5;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}

.message {
  margin: 0 0 10px;
  color: var(--success-color);
  font-size: 11px;
}

.error {
  margin: 0 0 10px;
  padding: 8px 10px;
  border: 1px solid var(--error-color);
  border-radius: var(--radius-sm);
  color: var(--error-color);
  font-size: 12px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: 70px;
  color: var(--text-tertiary);
  font-size: 12px;
  text-align: center;
}

.empty button {
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
}

.hint {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
}

.limit {
  margin: 14px 0 0;
  color: var(--text-tertiary);
  font-size: 11px;
}
</style>
