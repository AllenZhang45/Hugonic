<script setup lang="ts">
import { onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import FileTree from "./FileTree.vue";
import Icon from "./Icon.vue";
import { useFilesStore } from "../stores/files";
import { useProjectStore } from "../stores/project";
import { usePreviewStore } from "../stores/preview";

const files = useFilesStore();
const project = useProjectStore();
const preview = usePreviewStore();

onMounted(() => {
  void project.loadRecent();
});

async function chooseProject() {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") {
    await openPath(path);
  }
}

async function openPath(path: string) {
  await preview.stop();

  try {
    await project.open(path);
    files.reset();
    await files.loadDir("");
    void preview.start();
  } catch {
    // 错误已写入 project.error，这里只负责不让异常冒泡。
  }
}
</script>

<template>
  <aside class="explorer" aria-label="资源管理器">
    <div class="head">
      <span class="panel-title">资源管理器</span>
      <button
        class="icon-btn"
        :disabled="project.opening"
        title="打开 Hugo 站点"
        @click="chooseProject"
      >
        <Icon name="folder" class="icon" />
      </button>
    </div>

    <div class="search">
      <Icon name="search" class="search-icon" />
      <input class="search-input" type="search" placeholder="搜索文件…" disabled />
    </div>

    <p v-if="project.error" class="error" role="alert">{{ project.error }}</p>

    <template v-if="project.info">
      <div class="project">
        <div class="project-text">
          <span class="project-name">{{ project.info.name }}</span>
          <span class="project-meta">{{ project.info.configFile }}</span>
        </div>
        <button class="switch" :disabled="project.opening" @click="chooseProject">切换</button>
      </div>

      <div class="tree-wrap">
        <FileTree :entries="files.entries[''] ?? []" />
        <p v-if="files.loading.includes('')" class="hint">正在读取目录…</p>
      </div>
    </template>

    <div v-else class="empty">
      <Icon name="folder" class="empty-icon" />
      <p class="empty-title">尚未打开项目</p>
      <p class="empty-hint">选择一个包含 hugo.toml 或 config.* 的目录</p>
      <button class="open-btn" :disabled="project.opening" @click="chooseProject">
        {{ project.opening ? "打开中…" : "打开站点" }}
      </button>

      <ul v-if="project.recent.length" class="recent">
        <li v-for="item in project.recent" :key="item.root">
          <button class="recent-item" :title="item.root" @click="openPath(item.root)">
            {{ item.name }}
          </button>
        </li>
      </ul>
    </div>
  </aside>
</template>

<style scoped>
.explorer {
  display: flex;
  flex-direction: column;
  width: 260px;
  flex-shrink: 0;
  border-right: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 34px;
  flex-shrink: 0;
  padding: 0 10px 0 12px;
  border-bottom: 1px solid var(--border-color);
}

.panel-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-tertiary);
}

.icon-btn {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
}

.icon-btn:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.icon {
  width: 15px;
  height: 15px;
}

.search {
  position: relative;
  flex-shrink: 0;
  padding: 8px 10px;
}

.search-icon {
  position: absolute;
  top: 50%;
  left: 18px;
  width: 13px;
  height: 13px;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 26px;
  padding: 0 8px 0 26px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
  color: var(--text-secondary);
  font: inherit;
  font-size: 12px;
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.error {
  margin: 0 10px 8px;
  padding: 6px 8px;
  border: 1px solid var(--error-color);
  border-radius: var(--radius-sm);
  background: rgb(239 68 68 / 10%);
  color: var(--error-color);
  font-size: 11px;
  line-height: 1.5;
  user-select: text;
}

.project {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px 6px 12px;
  border-top: 1px solid var(--border-color);
  border-bottom: 1px solid var(--border-color);
}

.project-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.project-name {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 12px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.project-meta {
  color: var(--text-tertiary);
  font-size: 10px;
}

.switch {
  height: 22px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
}

.switch:hover:not(:disabled) {
  background: var(--bg-tertiary);
}

.tree-wrap {
  flex: 1;
  overflow: auto;
  padding: 6px 0;
}

.hint {
  margin: 0;
  padding: 4px 12px;
  color: var(--text-tertiary);
  font-size: 11px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin-top: 48px;
  padding: 0 20px;
  text-align: center;
}

.empty-icon {
  width: 28px;
  height: 28px;
  color: var(--border-color);
}

.empty-title {
  margin: 4px 0 0;
  color: var(--text-secondary);
  font-size: 12px;
}

.empty-hint {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
  line-height: 1.6;
}

.open-btn {
  margin-top: 8px;
  height: 26px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 12px;
}

.open-btn:hover:not(:disabled) {
  border-color: var(--theme-color);
  color: var(--text-primary);
}

.recent {
  width: 100%;
  margin: 14px 0 0;
  padding: 0;
  list-style: none;
}

.recent li + li {
  margin-top: 2px;
}

.recent-item {
  width: 100%;
  height: 24px;
  padding: 0 8px;
  overflow: hidden;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  font-size: 11px;
  text-align: left;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.recent-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}
</style>
