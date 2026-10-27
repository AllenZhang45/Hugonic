<script setup lang="ts">
import { computed, onMounted } from "vue";
import { confirm } from "@tauri-apps/plugin-dialog";
import Icon from "./Icon.vue";
import { useInsightsStore } from "../stores/insights";
import { usePreviewStore } from "../stores/preview";
import { useProjectStore } from "../stores/project";
import { useThemesStore } from "../stores/themes";
import { useUiStore } from "../stores/ui";

const project = useProjectStore();
const themes = useThemesStore();
const preview = usePreviewStore();
const insights = useInsightsStore();
const ui = useUiStore();

onMounted(() => {
  void themes.load();
});

const list = computed(() => themes.data);

async function switchTheme(dir: string, name: string) {
  if (!list.value?.canSwitch) {
    return;
  }

  const confirmed = await confirm(
    `切换主题为 ${name}？\n会修改 ${list.value.configFile || "Hugo 配置"}，并重启预览。`,
    {
      title: "切换主题",
      kind: "warning",
      okLabel: "切换",
      cancelLabel: "取消",
    },
  );
  if (!confirmed) {
    return;
  }

  const changed = await themes.apply(dir);
  if (!changed) {
    return;
  }

  insights.reset();
  if (preview.running) {
    await preview.stop();
    await preview.start();
  }
}
</script>

<template>
  <section class="view" aria-label="主题">
    <div class="header">
      <h2 class="title">主题</h2>
      <span v-if="list?.configFile" class="subtitle">配置：{{ list.configFile }}</span>
      <button class="refresh" type="button" :disabled="themes.loading" @click="themes.load(true)">
        {{ themes.loading ? "扫描中…" : "刷新" }}
      </button>
    </div>

    <div v-if="!project.info" class="empty">
      <p>尚未打开项目</p>
      <button type="button" @click="ui.setActiveView('content')">回到内容视图打开站点</button>
    </div>

    <p v-else-if="themes.error" class="error">{{ themes.error }}</p>

    <template v-else-if="list">
      <p v-if="!list.canSwitch" class="warning">
        {{ list.reason ?? "当前配置不支持自动切换主题" }}
      </p>

      <div v-if="list.themes.length" class="grid">
        <article
          v-for="theme in list.themes"
          :key="theme.dir"
          class="card"
          :class="{ active: theme.active }"
        >
          <div class="shot">
            <img
              v-if="themes.screenshots[theme.dir]"
              :src="themes.screenshots[theme.dir]"
              :alt="theme.name"
            />
            <div v-else class="shot-placeholder">
              <Icon name="theme" class="shot-icon" />
            </div>
          </div>

          <div class="body">
            <div class="title-row">
              <h3>{{ theme.name }}</h3>
              <span v-if="theme.active" class="badge">当前</span>
            </div>
            <p v-if="theme.description" class="desc">{{ theme.description }}</p>
            <dl class="meta">
              <div v-if="theme.author">
                <dt>作者</dt>
                <dd>{{ theme.author }}</dd>
              </div>
              <div v-if="theme.license">
                <dt>许可</dt>
                <dd>{{ theme.license }}</dd>
              </div>
              <div v-if="theme.minVersion">
                <dt>最低 Hugo</dt>
                <dd>{{ theme.minVersion }}</dd>
              </div>
              <div>
                <dt>目录</dt>
                <dd class="mono">{{ theme.dir }}</dd>
              </div>
            </dl>
          </div>

          <div class="actions">
            <button
              type="button"
              class="primary"
              :disabled="theme.active || !list.canSwitch || themes.applying === theme.dir"
              @click="switchTheme(theme.dir, theme.name)"
            >
              {{ theme.active ? "使用中" : themes.applying === theme.dir ? "切换中…" : "应用" }}
            </button>
            <button type="button" @click="themes.openFolder(theme.dir)">打开目录</button>
          </div>
        </article>
      </div>

      <div v-else class="empty">
        <p>themes/ 下还没有安装主题</p>
        <p class="hint">把主题放到项目的 themes/&lt;name&gt;/ 目录后点击刷新。</p>
      </div>
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
  margin-bottom: 18px;
}

.title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.subtitle {
  color: var(--text-tertiary);
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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

.warning {
  margin: 0 0 12px;
  padding: 8px 10px;
  border: 1px solid var(--warning-color);
  border-radius: var(--radius-sm);
  background: rgb(245 158 11 / 8%);
  color: var(--warning-color);
  font-size: 11px;
  line-height: 1.6;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
  align-items: start;
}

.card {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
}

.card.active {
  border-color: var(--theme-color);
}

.shot {
  height: 150px;
  overflow: hidden;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
}

.shot img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.shot-placeholder {
  display: grid;
  height: 100%;
  place-items: center;
}

.shot-icon {
  width: 32px;
  height: 32px;
  color: var(--border-color);
}

.body {
  padding: 12px 14px 6px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-row h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}

.badge {
  padding: 1px 6px;
  border-radius: 999px;
  background: var(--theme-color);
  color: #fff;
  font-size: 10px;
}

.desc {
  margin: 6px 0 0;
  color: var(--text-tertiary);
  font-size: 11px;
  line-height: 1.6;
}

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
  margin: 10px 0 0;
  font-size: 11px;
}

.meta div {
  display: flex;
  gap: 5px;
}

.meta dt {
  color: var(--text-tertiary);
}

.meta dd {
  margin: 0;
  color: var(--text-secondary);
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.actions {
  display: flex;
  gap: 6px;
  padding: 10px 14px 12px;
}

.actions button {
  height: 26px;
  padding: 0 12px;
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

.actions button.primary {
  border-color: var(--theme-color);
  background: var(--theme-color);
  color: #fff;
}

.actions button:disabled {
  opacity: 0.5;
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

.error {
  margin: 0;
  padding: 8px 10px;
  border: 1px solid var(--error-color);
  border-radius: var(--radius-sm);
  color: var(--error-color);
  font-size: 12px;
}
</style>
