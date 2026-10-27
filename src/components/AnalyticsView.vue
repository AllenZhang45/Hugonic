<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useInsightsStore } from "../stores/insights";
import { useProjectStore } from "../stores/project";
import { useUiStore } from "../stores/ui";

const project = useProjectStore();
const insights = useInsightsStore();
const ui = useUiStore();

onMounted(() => {
  void insights.load();
});

const info = computed(() => insights.data);
const maxPages = computed(() =>
  Math.max(1, ...(info.value?.sections.map((section) => section.pages) ?? [1])),
);

function barWidth(pages: number) {
  return `${Math.max(4, Math.round((pages / maxPages.value) * 100))}%`;
}
</script>

<template>
  <section class="view" aria-label="分析">
    <div class="header">
      <h2 class="title">分析</h2>
      <span class="subtitle">本地内容指标，不包含访问统计</span>
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
      <article class="card">
        <h3 class="card-title">内容规模</h3>
        <div class="stats">
          <div class="stat">
            <span class="stat-value">{{ info.counts.pages }}</span
            ><span>页面</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ info.words }}</span
            ><span>字数</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ info.counts.drafts }}</span
            ><span>草稿</span>
          </div>
        </div>
        <p class="hint">字数口径：中文按字符、英文按空白分词，代码块不计入。</p>
      </article>

      <article class="card">
        <h3 class="card-title">元数据检查</h3>
        <ul class="checks">
          <li :class="{ warn: info.missingTitle > 0 }">
            <span>缺少 title</span><span>{{ info.missingTitle }}</span>
          </li>
          <li :class="{ warn: info.missingDate > 0 }">
            <span>缺少 date</span><span>{{ info.missingDate }}</span>
          </li>
          <li>
            <span>分类数</span><span>{{ info.categories.length }}</span>
          </li>
        </ul>
      </article>

      <article class="card wide">
        <h3 class="card-title">按 section 分布</h3>
        <div v-if="info.sections.length" class="bars">
          <div v-for="section in info.sections" :key="section.name" class="bar-row">
            <span class="bar-label">{{ section.name }}</span>
            <div class="bar-track">
              <div class="bar-fill" :style="{ width: barWidth(section.pages) }" />
            </div>
            <span class="bar-value">{{ section.pages }} 页 · {{ section.words }} 字</span>
          </div>
        </div>
        <p v-else class="muted">content/ 下没有 Markdown 文件</p>
      </article>

      <article class="card">
        <h3 class="card-title">标签 Top {{ info.tags.length }}</h3>
        <div v-if="info.tags.length" class="chips">
          <span v-for="tag in info.tags" :key="tag.name" class="chip">
            {{ tag.name }} <b>{{ tag.count }}</b>
          </span>
        </div>
        <p v-else class="muted">还没有标签</p>
      </article>

      <article class="card">
        <h3 class="card-title">分类 Top {{ info.categories.length }}</h3>
        <div v-if="info.categories.length" class="chips">
          <span v-for="category in info.categories" :key="category.name" class="chip">
            {{ category.name }} <b>{{ category.count }}</b>
          </span>
        </div>
        <p v-else class="muted">还没有分类</p>
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
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 12px;
  align-items: start;
}

.card {
  padding: 14px 16px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
}

.wide {
  grid-column: 1 / -1;
}

.card-title {
  margin: 0 0 10px;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.stats {
  display: flex;
  gap: 22px;
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

.hint {
  margin: 10px 0 0;
  color: var(--text-tertiary);
  font-size: 11px;
  line-height: 1.6;
}

.checks {
  margin: 0;
  padding: 0;
  list-style: none;
  font-size: 12px;
}

.checks li {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  color: var(--text-secondary);
}

.checks li.warn {
  color: var(--warning-color);
}

.bars {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bar-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bar-label {
  width: 84px;
  overflow: hidden;
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.bar-track {
  flex: 1;
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--bg-tertiary);
}

.bar-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--theme-color);
}

.bar-value {
  width: 120px;
  color: var(--text-tertiary);
  font-size: 11px;
  text-align: right;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  padding: 2px 8px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  color: var(--text-secondary);
  font-size: 11px;
}

.chip b {
  color: var(--text-tertiary);
  font-weight: 500;
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
