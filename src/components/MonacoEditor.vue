<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import * as monaco from "monaco-editor";
import EditorWorker from "monaco-editor/editor/editor.worker.js?worker";
import { useEditorStore, type EditorTab } from "../stores/editor";

self.MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

monaco.editor.defineTheme("hugonic-dark", {
  base: "vs-dark",
  inherit: true,
  rules: [],
  colors: {
    "editor.background": "#0f1117",
    "editor.foreground": "#e2e8f0",
    "editorLineNumber.foreground": "#3d435c",
    "editorLineNumber.activeForeground": "#94a3b8",
    "editor.selectionBackground": "#6366f155",
    "editor.lineHighlightBackground": "#161822",
    "editorCursor.foreground": "#6366f1",
    "editorIndentGuide.background1": "#1e2030",
  },
});

const actions = [
  { id: "bold", label: "B", title: "粗体（**…**）" },
  { id: "italic", label: "I", title: "斜体（*…*）" },
  { id: "code", label: "`", title: "行内代码" },
  { id: "link", label: "链接", title: "插入链接" },
  { id: "image", label: "图片", title: "插入图片" },
  { id: "table", label: "表格", title: "插入表格" },
];

const editorStore = useEditorStore();
const host = ref<HTMLDivElement | null>(null);

const isMarkdown = computed(() => editorStore.activeTab?.language === "markdown");

let instance: monaco.editor.IStandaloneCodeEditor | null = null;
let currentPath: string | null = null;
let applyingStoreChange = false;
const viewStates = new Map<string, monaco.editor.ICodeEditorViewState | null>();

function modelFor(tab: EditorTab) {
  const uri = monaco.Uri.from({ scheme: "hugonic", path: `/${tab.path}` });
  const existing = monaco.editor.getModel(uri);
  if (existing) {
    return existing;
  }
  return monaco.editor.createModel(tab.content, tab.language, uri);
}

function applyActiveTab() {
  if (!instance) {
    return;
  }

  const path = editorStore.activePath;
  if (currentPath) {
    viewStates.set(currentPath, instance.saveViewState());
  }

  if (!path) {
    instance.setModel(null);
    currentPath = null;
    return;
  }

  const tab = editorStore.tabs.find((item) => item.path === path);
  if (!tab) {
    return;
  }

  applyingStoreChange = true;
  instance.setModel(modelFor(tab));
  applyingStoreChange = false;
  currentPath = path;

  const state = viewStates.get(path);
  if (state) {
    instance.restoreViewState(state);
  }
  instance.focus();
}

function wrapSelection(before: string, after: string, placeholder: string) {
  if (!instance) {
    return;
  }

  const model = instance.getModel();
  const selection = instance.getSelection();
  if (!model || !selection) {
    return;
  }

  const selected = model.getValueInRange(selection);
  const text = `${before}${selected || placeholder}${after}`;
  instance.executeEdits("markdown-toolbar", [{ range: selection, text, forceMoveMarkers: true }]);
  instance.focus();
}

function insertText(text: string) {
  if (!instance) {
    return;
  }

  const selection = instance.getSelection();
  if (!selection) {
    return;
  }

  instance.executeEdits("markdown-toolbar", [{ range: selection, text, forceMoveMarkers: true }]);
  instance.focus();
}

function applyAction(id: string) {
  switch (id) {
    case "bold":
      wrapSelection("**", "**", "粗体");
      break;
    case "italic":
      wrapSelection("*", "*", "斜体");
      break;
    case "code":
      wrapSelection("`", "`", "代码");
      break;
    case "link":
      wrapSelection("[", "](https://)", "链接文字");
      break;
    case "image":
      wrapSelection("![", "](path/to/image.png)", "替代文字");
      break;
    case "table":
      insertText("| 列 1 | 列 2 |\n| --- | --- |\n| 内容 | 内容 |\n");
      break;
  }
}

onMounted(() => {
  if (!host.value) {
    return;
  }

  instance = monaco.editor.create(host.value, {
    theme: "hugonic-dark",
    automaticLayout: true,
    fontSize: 13,
    lineHeight: 20,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    renderLineHighlight: "line",
    fontFamily:
      "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace",
    tabSize: 2,
    padding: { top: 10 },
    smoothScrolling: true,
  });

  instance.onDidChangeModelContent(() => {
    if (applyingStoreChange || !currentPath) {
      return;
    }
    editorStore.updateContent(currentPath, instance!.getValue());
  });

  instance.onDidChangeCursorPosition((event) => {
    editorStore.setCursor(event.position.lineNumber, event.position.column);
  });

  instance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    void editorStore.save();
  });

  applyActiveTab();
});

onBeforeUnmount(() => {
  instance?.dispose();
  instance = null;
  monaco.editor.getModels().forEach((model) => model.dispose());
});

watch(
  () => editorStore.activePath,
  () => applyActiveTab(),
);

watch(
  () => editorStore.activeTab?.content,
  (content) => {
    if (!instance || !currentPath || content === undefined) {
      return;
    }

    const model = instance.getModel();
    if (model && model.getValue() !== content) {
      applyingStoreChange = true;
      model.setValue(content);
      applyingStoreChange = false;
    }
  },
);
</script>

<template>
  <div class="editor-shell">
    <div v-if="isMarkdown" class="toolbar" role="toolbar" aria-label="Markdown 工具栏">
      <button
        v-for="action in actions"
        :key="action.id"
        type="button"
        class="tool"
        :title="action.title"
        @click="applyAction(action.id)"
      >
        {{ action.label }}
      </button>
      <span class="spacer" />
      <span class="hint">Markdown</span>
    </div>

    <div ref="host" class="monaco-host" />
  </div>
</template>

<style scoped>
.editor-shell {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 32px;
  flex-shrink: 0;
  padding: 0 8px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.tool {
  min-width: 24px;
  height: 22px;
  padding: 0 6px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
}

.tool:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.spacer {
  flex: 1;
}

.hint {
  color: var(--text-tertiary);
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.monaco-host {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
</style>
