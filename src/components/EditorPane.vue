<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import { confirm } from "@tauri-apps/plugin-dialog";
import { useEditorStore, type EditorTab } from "../stores/editor";

const MonacoEditor = defineAsyncComponent(() => import("./MonacoEditor.vue"));
const editor = useEditorStore();

function dirty(tab: EditorTab) {
  return tab.content !== tab.savedContent;
}

async function requestClose(path: string) {
  const tab = editor.tabs.find((item) => item.path === path);
  if (!tab) {
    return;
  }

  if (dirty(tab)) {
    const confirmed = await confirm("有未保存的修改，关闭后无法恢复。", {
      title: "关闭标签",
      kind: "warning",
      okLabel: "放弃修改",
      cancelLabel: "取消",
    });
    if (!confirmed) {
      return;
    }
  }

  await editor.close(path);
}
</script>

<template>
  <section class="editor" aria-label="编辑器">
    <div class="tabs">
      <button
        v-for="tab in editor.tabs"
        :key="tab.path"
        type="button"
        class="tab"
        :class="{ active: tab.path === editor.activePath }"
        :title="tab.path"
        @click="editor.activate(tab.path)"
      >
        <span v-if="dirty(tab)" class="dirty" aria-label="未保存" />
        <span class="name">{{ tab.name }}</span>
        <span class="close" title="关闭" @click.stop="requestClose(tab.path)">×</span>
      </button>
      <span v-if="!editor.tabs.length" class="tab-empty">未打开文件</span>
    </div>

    <div v-if="editor.activeTab?.conflict" class="conflict">
      <span>该文件已在磁盘上被修改。</span>
      <button type="button" @click="editor.resolveConflict(editor.activeTab.path, 'reload')">
        重新加载
      </button>
      <button type="button" @click="editor.resolveConflict(editor.activeTab.path, 'keep')">
        保留我的修改
      </button>
    </div>

    <p v-if="editor.error" class="error" role="alert">{{ editor.error }}</p>

    <MonacoEditor v-if="editor.activeTab" />

    <div v-else class="empty">
      <p class="empty-title">没有打开的文件</p>
      <p class="empty-hint">从左侧资源管理器选择一个文件开始编辑</p>
    </div>
  </section>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--bg-color);
}

.tabs {
  display: flex;
  align-items: stretch;
  height: 34px;
  flex-shrink: 0;
  overflow-x: auto;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 200px;
  padding: 0 6px 0 12px;
  border: 0;
  border-right: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-tertiary);
  font-size: 12px;
  white-space: nowrap;
}

.tab:hover {
  color: var(--text-secondary);
}

.tab.active {
  background: var(--bg-color);
  color: var(--text-primary);
  box-shadow: inset 0 -2px 0 var(--theme-color);
}

.name {
  overflow: hidden;
  text-overflow: ellipsis;
}

.dirty {
  width: 6px;
  height: 6px;
  flex-shrink: 0;
  border-radius: var(--radius-full);
  background: var(--theme-color);
}

.close {
  display: grid;
  place-items: center;
  width: 16px;
  height: 16px;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  font-size: 14px;
  line-height: 1;
}

.close:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.tab-empty {
  padding: 0 12px;
  color: var(--text-tertiary);
  font-size: 12px;
}

.conflict {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding: 5px 12px;
  border-bottom: 1px solid var(--border-color);
  background: rgb(245 158 11 / 10%);
  color: var(--warning-color);
  font-size: 11px;
}

.conflict button {
  height: 20px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
}

.error {
  margin: 0;
  padding: 6px 12px;
  border-bottom: 1px solid var(--border-color);
  background: rgb(239 68 68 / 10%);
  color: var(--error-color);
  font-size: 11px;
  user-select: text;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin: auto;
  text-align: center;
}

.empty-title {
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.empty-hint {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
}
</style>
