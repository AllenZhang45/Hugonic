<script setup lang="ts">
import { computed } from "vue";
import Icon from "./Icon.vue";
import { usePreviewStore } from "../stores/preview";
import { useProjectStore } from "../stores/project";

const preview = usePreviewStore();
const project = useProjectStore();

const statusText = computed(() => {
  switch (preview.status) {
    case "starting":
      return "正在启动 hugo server…";
    case "error":
      return "预览启动失败";
    case "running":
      return "预览已启动";
    default:
      return "预览未启动";
  }
});

const hint = computed(() => {
  if (!project.info) {
    return "打开一个 Hugo 站点后会自动启动预览";
  }
  if (preview.status === "error") {
    return "检查 Hugo 配置或端口占用后重试";
  }
  return "保存文件后 Hugo 会自动重建并刷新 iframe";
});

const frameUrl = computed(() => {
  if (!preview.url) {
    return "";
  }
  // 用查询参数触发重新导航，而不是销毁重建 iframe：
  // 在 WebKitGTK（尤其是虚拟机软件渲染）下，重建 iframe 容易留下白屏。
  return preview.reloadToken === 0
    ? preview.url
    : `${preview.url}/?hugonic_reload=${preview.reloadToken}`;
});

const deviceTitle = computed(() =>
  preview.device === "desktop" ? "切换到移动端视图" : "切换到桌面端视图",
);
</script>

<template>
  <section class="preview" aria-label="实时预览">
    <div class="head">
      <span class="panel-title">实时预览</span>
      <span class="url" :class="{ live: preview.running }">
        <i class="dot" />{{ preview.url ? preview.url.replace("http://", "") : "未启动" }}
      </span>
      <div class="actions">
        <button
          class="icon-btn"
          :disabled="!preview.running"
          title="刷新"
          @click="preview.reload()"
        >
          <Icon name="refresh" class="icon" />
        </button>
        <button
          class="icon-btn"
          :disabled="!preview.running"
          title="在浏览器中打开"
          @click="preview.openExternal()"
        >
          <Icon name="external" class="icon" />
        </button>
        <button
          class="icon-btn"
          :disabled="!preview.running"
          :title="deviceTitle"
          @click="preview.toggleDevice()"
        >
          <Icon name="device" class="icon" />
        </button>
      </div>
    </div>

    <div class="body">
      <div
        v-if="preview.running && preview.url"
        class="frame-wrap"
        :class="{ mobile: preview.device === 'mobile' }"
      >
        <iframe :src="frameUrl" class="frame" title="Hugo 站点预览" />
      </div>

      <div v-else class="empty">
        <p class="empty-title">{{ statusText }}</p>
        <p class="empty-hint">{{ hint }}</p>
        <p v-if="preview.error" class="error">{{ preview.error }}</p>
        <button
          v-if="project.info && preview.status !== 'starting'"
          class="start"
          type="button"
          @click="preview.start()"
        >
          {{ preview.status === "error" ? "重试" : "启动预览" }}
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.preview {
  display: flex;
  flex-direction: column;
  width: clamp(320px, 42%, 640px);
  flex-shrink: 0;
  border-left: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.head {
  display: flex;
  align-items: center;
  gap: 10px;
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

.url {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--bg-color);
  color: var(--text-tertiary);
  font-size: 11px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.url.live {
  color: var(--text-secondary);
}

.dot {
  width: 6px;
  height: 6px;
  flex-shrink: 0;
  border-radius: var(--radius-full);
  background: var(--text-tertiary);
}

.url.live .dot {
  background: var(--success-color);
}

.actions {
  display: flex;
  gap: 2px;
  margin-left: auto;
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

.icon-btn:disabled {
  opacity: 0.4;
}

.icon {
  width: 14px;
  height: 14px;
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
  background: var(--bg-color);
}

.frame-wrap {
  display: flex;
  flex: 1;
  justify-content: center;
  min-height: 0;
}

.frame-wrap.mobile {
  padding: 12px;
}

.frame {
  width: 100%;
  height: 100%;
  border: 0;
  background: #fff;
}

.frame-wrap.mobile .frame {
  width: 390px;
  border-radius: var(--radius-md);
  box-shadow: 0 0 0 1px var(--border-color);
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin: auto;
  padding: 0 24px;
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
  line-height: 1.6;
}

.error {
  margin: 6px 0 0;
  max-width: 100%;
  color: var(--error-color);
  font-size: 11px;
  line-height: 1.5;
  user-select: text;
  word-break: break-word;
}

.start {
  margin-top: 10px;
  height: 26px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 12px;
}

.start:hover {
  border-color: var(--theme-color);
  color: var(--text-primary);
}
</style>
