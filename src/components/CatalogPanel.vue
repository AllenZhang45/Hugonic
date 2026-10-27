<script setup lang="ts">
import { computed, onMounted, ref, type DirectiveBinding } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "./Icon.vue";
import { useCatalogStore, type CatalogTheme } from "../stores/catalog";
import { useThemesStore } from "../stores/themes";

const catalog = useCatalogStore();
const themes = useThemesStore();

const query = ref("");
const tag = ref("");
const sort = ref("stars");
const selected = ref<CatalogTheme | null>(null);
const detailScreenshot = ref<string | null>(null);
const loadingScreenshot = ref(false);
const copiedIndex = ref<number | null>(null);

onMounted(() => {
  void catalog.init();
});

const observers = new WeakMap<Element, IntersectionObserver>();

const vLazyShot = {
  mounted(element: HTMLElement, binding: DirectiveBinding<string>) {
    if (typeof IntersectionObserver === "undefined") {
      void catalog.loadScreenshot(binding.value);
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          observer.disconnect();
          observers.delete(element);
          void catalog.loadScreenshot(binding.value);
        }
      },
      { rootMargin: "240px" },
    );
    observer.observe(element);
    observers.set(element, observer);
  },
  unmounted(element: HTMLElement) {
    observers.get(element)?.disconnect();
    observers.delete(element);
  },
};

const installedDirs = computed(
  () => new Set((themes.data?.themes ?? []).map((theme) => theme.dir)),
);

const tags = computed(() => {
  const counts = new Map<string, number>();
  for (const theme of catalog.data?.themes ?? []) {
    for (const item of theme.tags) {
      counts.set(item, (counts.get(item) ?? 0) + 1);
    }
  }
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, 30)
    .map(([name]) => name);
});

const filtered = computed(() => {
  const needle = query.value.trim().toLowerCase();
  const result = (catalog.data?.themes ?? []).filter((theme) => {
    const matchesTag = !tag.value || theme.tags.includes(tag.value);
    const matchesQuery =
      !needle ||
      theme.name.toLowerCase().includes(needle) ||
      theme.description.toLowerCase().includes(needle) ||
      theme.tags.some((item) => item.toLowerCase().includes(needle));
    return matchesTag && matchesQuery;
  });

  return result.sort((a, b) => {
    if (sort.value === "name") {
      return a.name.localeCompare(b.name);
    }
    if (sort.value === "updated") {
      return (b.updated ?? "").localeCompare(a.updated ?? "");
    }
    return (b.stars ?? 0) - (a.stars ?? 0);
  });
});

const lastUpdated = computed(() => {
  if (!catalog.data?.fetchedAt) {
    return "尚未同步";
  }
  return new Date(catalog.data.fetchedAt * 1000).toLocaleString();
});

const progressPercent = computed(() => {
  const progress = catalog.progress;
  if (!progress || !progress.total) {
    return 0;
  }
  return Math.round((progress.done / progress.total) * 100);
});

function isInstalled(theme: CatalogTheme) {
  return installedDirs.value.has(theme.slug);
}

async function openDetail(theme: CatalogTheme) {
  selected.value = theme;
  detailScreenshot.value = catalog.screenshots[theme.slug] ?? null;
  if (!detailScreenshot.value) {
    loadingScreenshot.value = true;
    detailScreenshot.value = await catalog.loadScreenshot(theme.slug);
    loadingScreenshot.value = false;
  }
}

function closeDetail() {
  selected.value = null;
  detailScreenshot.value = null;
}

async function copyCommand(command: string, index: number) {
  try {
    await navigator.clipboard.writeText(command);
    copiedIndex.value = index;
    window.setTimeout(() => {
      copiedIndex.value = null;
    }, 1200);
  } catch {
    copiedIndex.value = null;
  }
}

function openLink(url: string | null | undefined) {
  if (url) {
    void openUrl(url);
  }
}

async function syncNow() {
  await catalog.start(true);
}
</script>

<template>
  <div class="catalog">
    <div class="toolbar">
      <input v-model="query" class="search" type="search" placeholder="搜索名称、描述或标签…" />
      <select v-model="tag" class="select" aria-label="按标签筛选">
        <option value="">全部标签</option>
        <option v-for="item in tags" :key="item" :value="item">{{ item }}</option>
      </select>
      <select v-model="sort" class="select" aria-label="排序">
        <option value="stars">按 Stars</option>
        <option value="updated">按更新时间</option>
        <option value="name">按名称</option>
      </select>
      <span class="updated">更新于 {{ lastUpdated }}</span>
      <button class="sync" type="button" :disabled="catalog.running" @click="syncNow">
        {{ catalog.running ? "同步中…" : "同步目录" }}
      </button>
    </div>

    <div v-if="catalog.running" class="progress">
      <div class="track">
        <div class="fill" :style="{ width: `${progressPercent}%` }" />
      </div>
      <p class="progress-text">
        {{ catalog.progress?.message || "同步中…" }}
        <span v-if="catalog.progress?.failed">· 失败 {{ catalog.progress.failed }}</span>
      </p>
    </div>

    <p v-if="catalog.error" class="error">{{ catalog.error }}</p>

    <div v-if="catalog.data?.themes.length" class="summary">
      <span>{{ filtered.length }} / {{ catalog.data.themes.length }} 个主题</span>
      <span>仅 GitHub · 数据来自 themes.gohugo.io</span>
    </div>

    <div v-if="filtered.length" class="grid">
      <article
        v-for="theme in filtered"
        :key="theme.slug"
        class="card"
        tabindex="0"
        @click="openDetail(theme)"
        @keydown.enter="openDetail(theme)"
      >
        <div v-lazy-shot="theme.slug" class="shot">
          <img
            v-if="catalog.screenshots[theme.slug]"
            :src="catalog.screenshots[theme.slug]"
            :alt="theme.name"
          />
          <div v-else class="shot-placeholder">
            <Icon name="theme" class="shot-icon" />
          </div>
        </div>
        <div class="body">
          <div class="title-row">
            <h3>{{ theme.name }}</h3>
            <span v-if="isInstalled(theme)" class="badge">已安装</span>
          </div>
          <p class="desc">{{ theme.description || "暂无描述" }}</p>
          <div class="tags">
            <span v-for="item in theme.tags.slice(0, 3)" :key="item" class="tag">{{ item }}</span>
          </div>
          <p class="meta">
            <span v-if="theme.stars !== null">★ {{ theme.stars }}</span>
            <span v-if="theme.license">{{ theme.license }}</span>
            <span v-if="theme.updated">{{ theme.updated }}</span>
          </p>
        </div>
      </article>
    </div>

    <div v-else-if="!catalog.running" class="empty">
      <template v-if="catalog.data?.themes.length">
        <p>没有匹配的主题</p>
      </template>
      <template v-else>
        <p>还没有主题目录缓存</p>
        <p class="hint">首次同步会抓取约 320 个主题的元数据和安装命令，在后台运行。</p>
        <button type="button" @click="syncNow">开始同步</button>
      </template>
    </div>

    <p class="footnote">
      安装命令由主题 README 自动提取，仅供参考；不会自动执行，也不会下载主题代码。
    </p>

    <div v-if="selected" class="overlay" @click.self="closeDetail">
      <div class="modal">
        <button class="close" type="button" aria-label="关闭" @click="closeDetail">×</button>

        <div class="detail-shot">
          <img v-if="detailScreenshot" :src="detailScreenshot" :alt="selected.name" />
          <div v-else class="shot-placeholder">
            <Icon :name="loadingScreenshot ? 'refresh' : 'theme'" class="shot-icon" />
          </div>
        </div>

        <div class="detail-body">
          <div class="title-row">
            <h2>{{ selected.name }}</h2>
            <span v-if="isInstalled(selected)" class="badge">已安装</span>
          </div>
          <p class="detail-desc">{{ selected.description || "暂无描述" }}</p>

          <dl class="detail-meta">
            <div v-if="selected.license">
              <dt>License</dt>
              <dd>{{ selected.license }}</dd>
            </div>
            <div v-if="selected.stars !== null">
              <dt>GitHub Stars</dt>
              <dd>{{ selected.stars }}</dd>
            </div>
            <div v-if="selected.updated">
              <dt>Last Updated</dt>
              <dd>{{ selected.updated }}</dd>
            </div>
            <div>
              <dt>Tags</dt>
              <dd>{{ selected.tags.join(" · ") || "—" }}</dd>
            </div>
          </dl>

          <div class="links">
            <button type="button" @click="openLink(selected.repo)">仓库</button>
            <button v-if="selected.demo" type="button" @click="openLink(selected.demo)">
              Demo
            </button>
            <button type="button" @click="openLink(selected.installSource ?? selected.repo)">
              安装来源
            </button>
          </div>

          <section class="install">
            <h3>安装方式</h3>
            <p class="install-note">
              {{
                selected.installExtracted
                  ? "从主题 README 自动提取，请以官方说明为准。"
                  : "README 未找到明确命令，以下是按仓库地址生成的建议。"
              }}
            </p>
            <div
              v-for="(command, index) in selected.installCommands"
              :key="command"
              class="command"
            >
              <code>{{ command }}</code>
              <button type="button" @click="copyCommand(command, index)">
                {{ copiedIndex === index ? "已复制" : "复制" }}
              </button>
            </div>
            <p class="install-note">在站点根目录执行；主题目录名以 {{ selected.slug }} 为例。</p>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.catalog {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 0 24px 24px;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.search {
  width: 220px;
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

.select {
  height: 28px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-size: 11px;
}

.updated {
  margin-left: auto;
  color: var(--text-tertiary);
  font-size: 11px;
}

.sync {
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--theme-color);
  border-radius: var(--radius-sm);
  background: var(--theme-color);
  color: #fff;
  font-size: 12px;
}

.sync:disabled {
  opacity: 0.5;
}

.progress {
  margin-bottom: 12px;
}

.track {
  height: 5px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--bg-tertiary);
}

.fill {
  height: 100%;
  background: var(--theme-color);
  transition: width 0.2s ease;
}

.progress-text {
  margin: 4px 0 0;
  color: var(--text-tertiary);
  font-size: 10px;
}

.summary {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
  color: var(--text-tertiary);
  font-size: 10px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.card {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
  cursor: pointer;
}

.card:hover {
  border-color: var(--theme-color);
}

.shot {
  height: 110px;
  overflow: hidden;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(135deg, var(--bg-tertiary), var(--bg-secondary));
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
  width: 24px;
  height: 24px;
  color: var(--border-color);
}

.body {
  padding: 10px 12px 12px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-row h3 {
  margin: 0;
  overflow: hidden;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.badge {
  flex: none;
  padding: 1px 6px;
  border-radius: 999px;
  background: var(--theme-color);
  color: #fff;
  font-size: 10px;
}

.desc {
  display: -webkit-box;
  margin: 6px 0 0;
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 11px;
  line-height: 1.55;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 8px;
}

.tag {
  padding: 1px 6px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  color: var(--text-tertiary);
  font-size: 10px;
}

.meta {
  display: flex;
  gap: 10px;
  margin: 8px 0 0;
  color: var(--text-tertiary);
  font-size: 10px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: 60px;
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

.hint,
.footnote {
  color: var(--text-tertiary);
  font-size: 10px;
  line-height: 1.6;
}

.footnote {
  margin: 18px 0 0;
}

.error {
  margin: 0 0 10px;
  padding: 8px 10px;
  border: 1px solid var(--error-color);
  border-radius: var(--radius-sm);
  color: var(--error-color);
  font-size: 12px;
}

.overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgb(0 0 0 / 55%);
}

.modal {
  position: relative;
  display: flex;
  width: min(760px, 100%);
  max-height: 86vh;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
  box-shadow: 0 24px 60px rgb(0 0 0 / 40%);
}

.close {
  position: absolute;
  top: 8px;
  right: 10px;
  z-index: 1;
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: var(--radius-sm);
  background: rgb(0 0 0 / 35%);
  color: #fff;
  font-size: 18px;
  line-height: 1;
}

.detail-shot {
  width: 300px;
  flex: none;
  background: var(--bg-tertiary);
}

.detail-shot img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.detail-body {
  flex: 1;
  min-width: 0;
  padding: 18px 20px 20px;
}

.detail-body h2 {
  margin: 0;
  font-size: 16px;
}

.detail-desc {
  margin: 8px 0 0;
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.65;
}

.detail-meta {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px 14px;
  margin: 12px 0 0;
  font-size: 11px;
}

.detail-meta div {
  display: flex;
  gap: 6px;
}

.detail-meta dt {
  color: var(--text-tertiary);
}

.detail-meta dd {
  margin: 0;
  color: var(--text-secondary);
}

.links {
  display: flex;
  gap: 6px;
  margin-top: 12px;
}

.links button {
  height: 26px;
  padding: 0 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
}

.install {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border-color);
}

.install h3 {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 600;
}

.install-note {
  margin: 0 0 8px;
  color: var(--text-tertiary);
  font-size: 10px;
  line-height: 1.6;
}

.command {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  padding: 6px 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
}

.command code {
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 10px;
  white-space: pre;
}

.command button {
  flex: none;
  height: 22px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 10px;
}

@media (max-width: 720px) {
  .modal {
    flex-direction: column;
  }

  .detail-shot {
    width: 100%;
    height: 180px;
  }
}
</style>
