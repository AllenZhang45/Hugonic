<script setup lang="ts">
import { computed, onMounted } from "vue";
import Icon from "./Icon.vue";
import { useBuildStore } from "../stores/build";
import { useDeployStore } from "../stores/deploy";
import { useEditorStore } from "../stores/editor";
import { useInsightsStore } from "../stores/insights";
import { usePreviewStore } from "../stores/preview";
import { useProjectStore } from "../stores/project";
import { useUiStore } from "../stores/ui";

const project = useProjectStore();
const insights = useInsightsStore();
const build = useBuildStore();
const deploy = useDeployStore();
const preview = usePreviewStore();
const editor = useEditorStore();
const ui = useUiStore();

onMounted(() => {
  void insights.load();
});

const info = computed(() => insights.data);

const buildLabel = computed(() => {
  if (build.running) {
    return "构建中…";
  }
  if (build.durationMs !== null) {
    return `上次构建 ${build.durationMs} ms`;
  }
  return "尚未构建";
});

async function openRecent(path: string) {
  await editor.openFile(path);
  ui.setActiveView("content");
}
</script>

<template>
  <section class="view" aria-label="仪表盘">
    <div class="header">
      <h2 class="title">仪表盘</h2>
      <button
        class="refresh"
        type="button"
        :disabled="insights.loading"
        @click="insights.load(true)"
      >
        {{ insights.loading ? "扫描中…" : "刷新" }}
      </button>
    </div>

    <div v-if="!project.info" class="empty">
      <p>尚未打开项目</p>
      <button type="button" @click="ui.setActiveView('content')">回到内容视图打开站点</button>
    </div>

    <p v-else-if="insights.error" class="error">{{ insights.error }}</p>

    <div v-else-if="info" class="grid">
      <article class="card site">
        <h3 class="card-title">站点</h3>
        <p class="site-name">{{ info.title ?? project.info.name }}</p>
        <p class="mono">{{ project.info.root }}</p>
        <dl class="meta">
          <div>
            <dt>baseURL</dt>
            <dd>{{ info.baseUrl ?? "未配置" }}</dd>
          </div>
          <div>
            <dt>主题</dt>
            <dd>{{ info.theme ?? "未配置" }}</dd>
          </div>
          <div>
            <dt>语言</dt>
            <dd>{{ info.language ?? "未配置" }}</dd>
          </div>
          <div>
            <dt>配置</dt>
            <dd>{{ info.configFile || "未找到" }}</dd>
          </div>
        </dl>
      </article>

      <article class="card">
        <h3 class="card-title">内容统计</h3>
        <div class="stats">
          <div class="stat">
            <span class="stat-value">{{ info.counts.pages }}</span
            ><span>页面</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ info.counts.posts }}</span
            ><span>文章</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ info.counts.drafts }}</span
            ><span>草稿</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ info.counts.media }}</span
            ><span>媒体</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ info.counts.themes }}</span
            ><span>主题</span>
          </div>
        </div>
      </article>

      <article class="card">
        <h3 class="card-title">最近修改</h3>
        <ul v-if="info.recent.length" class="recent">
          <li v-for="file in info.recent" :key="file.path">
            <button type="button" :title="file.path" @click="openRecent(file.path)">
              <span class="recent-name">{{ file.name }}</span>
              <span class="recent-path">{{ file.path }}</span>
            </button>
          </li>
        </ul>
        <p v-else class="muted">content/ 下还没有 Markdown 文件</p>
      </article>

      <article class="card actions-card">
        <h3 class="card-title">快捷操作</h3>
        <div class="actions">
          <button type="button" :disabled="preview.running" @click="preview.start()">
            <Icon name="refresh" class="action-icon" />启动预览
          </button>
          <button type="button" :disabled="build.running" @click="build.openModal()">
            <Icon name="build" class="action-icon" />构建站点
          </button>
          <button type="button" @click="deploy.openModal()">
            <Icon name="upload" class="action-icon" />部署
          </button>
          <button type="button" @click="build.openOutput()">
            <Icon name="folder" class="action-icon" />打开输出目录
          </button>
        </div>
      </article>

      <article class="card">
        <h3 class="card-title">状态</h3>
        <ul class="status">
          <li>
            <span>预览</span
            ><span>{{ preview.running ? `运行中 · :${preview.port}` : "未启动" }}</span>
          </li>
          <li>
            <span>构建</span><span>{{ buildLabel }}</span>
          </li>
          <li>
            <span>部署</span><span>{{ deploy.url ?? "尚未部署" }}</span>
          </li>
        </ul>
      </article>
    </div>
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
  align-items: center;
  gap: 12px;
  margin-bottom: 18px;
}

.title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
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

.refresh:disabled {
  opacity: 0.5;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
  align-items: start;
}

.card {
  padding: 14px 16px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
}

.site {
  grid-column: span 2;
}

.card-title {
  margin: 0 0 10px;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.site-name {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}

.mono {
  margin: 4px 0 10px;
  color: var(--text-tertiary);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  word-break: break-all;
  user-select: text;
}

.meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 6px 14px;
  margin: 0;
}

.meta div {
  display: flex;
  gap: 6px;
  font-size: 11px;
}

.meta dt {
  color: var(--text-tertiary);
}

.meta dd {
  margin: 0;
  overflow: hidden;
  color: var(--text-secondary);
  white-space: nowrap;
  text-overflow: ellipsis;
  user-select: text;
}

.stats {
  display: flex;
  flex-wrap: wrap;
  gap: 14px 22px;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  color: var(--text-tertiary);
  font-size: 11px;
}

.stat-value {
  color: var(--text-primary);
  font-size: 22px;
  font-weight: 600;
  line-height: 1.2;
}

.recent {
  margin: 0;
  padding: 0;
  list-style: none;
}

.recent button {
  display: flex;
  flex-direction: column;
  gap: 1px;
  width: 100%;
  padding: 5px 6px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  text-align: left;
}

.recent button:hover {
  background: var(--bg-tertiary);
}

.recent-name {
  color: var(--text-secondary);
  font-size: 12px;
}

.recent-path {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 10px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

.actions button {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
}

.actions button:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.actions button:disabled {
  opacity: 0.5;
}

.action-icon {
  width: 14px;
  height: 14px;
}

.status {
  margin: 0;
  padding: 0;
  list-style: none;
  font-size: 12px;
}

.status li {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 4px 0;
  color: var(--text-secondary);
}

.status li span:first-child {
  color: var(--text-tertiary);
}

.muted {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
}

.error {
  margin: 0;
  padding: 8px 10px;
  border: 1px solid var(--error-color);
  border-radius: var(--radius-sm);
  color: var(--error-color);
  font-size: 12px;
  user-select: text;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  margin-top: 80px;
  color: var(--text-tertiary);
  font-size: 12px;
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
</style>
