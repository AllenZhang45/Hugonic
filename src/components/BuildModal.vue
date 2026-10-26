<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import Icon from "./Icon.vue";
import { useBuildStore } from "../stores/build";
import { useProjectStore } from "../stores/project";

const build = useBuildStore();
const project = useProjectStore();
const logRef = ref<HTMLPreElement | null>(null);

watch(
  () => build.logs.length,
  async () => {
    await nextTick();
    if (logRef.value) {
      logRef.value.scrollTop = logRef.value.scrollHeight;
    }
  },
);

const statusText = computed(() => {
  if (build.running) {
    return "构建中…";
  }
  if (build.cancelled) {
    return "已取消";
  }
  if (build.exitCode === 0) {
    return "构建完成";
  }
  if (build.exitCode !== null) {
    return `构建失败（退出码 ${build.exitCode}）`;
  }
  return "尚未构建";
});

const hasResult = computed(() => build.exitCode !== null || build.cancelled);

function startBuild() {
  void build.start();
}
</script>

<template>
  <div v-if="build.modalOpen" class="overlay" @click.self="build.closeModal()">
    <div class="modal" role="dialog" aria-label="构建站点" aria-modal="true">
      <div class="head">
        <Icon name="build" class="head-icon" />
        <span class="title">构建站点</span>
        <span
          class="status"
          :class="{
            ok: build.exitCode === 0,
            bad: build.exitCode !== null && build.exitCode !== 0,
          }"
        >
          {{ statusText }}
        </span>
        <button class="close" type="button" title="关闭" @click="build.closeModal()">×</button>
      </div>

      <div class="bar" :class="{ active: build.running }">
        <div v-if="build.running" class="bar-indeterminate" />
        <div
          v-else
          class="bar-fill"
          :class="{
            ok: build.exitCode === 0,
            bad: build.exitCode !== null && build.exitCode !== 0,
          }"
        />
      </div>

      <div class="summary">
        <span v-if="build.durationMs !== null" class="chip">耗时 {{ build.durationMs }} ms</span>
        <span v-if="build.summary?.pages != null" class="chip"
          >Pages {{ build.summary.pages }}</span
        >
        <span v-if="build.summary?.staticFiles != null" class="chip">
          Static {{ build.summary.staticFiles }}
        </span>
        <span v-if="build.summary?.processedImages != null" class="chip">
          Images {{ build.summary.processedImages }}
        </span>
      </div>

      <pre ref="logRef" class="log">{{ build.logs.map((log) => log.line).join("\n") }}</pre>

      <p v-if="build.error" class="error">{{ build.error }}</p>

      <div class="actions">
        <button v-if="build.running" class="btn danger" type="button" @click="build.cancel()">
          取消构建
        </button>
        <button
          v-else
          class="btn primary"
          type="button"
          :disabled="!project.info"
          @click="startBuild"
        >
          {{ hasResult ? "重新构建" : "开始构建" }}
        </button>

        <button class="btn" type="button" :disabled="!build.logs.length" @click="build.copyLogs()">
          复制日志
        </button>
        <button class="btn" type="button" :disabled="!project.info" @click="build.openOutput()">
          打开输出目录
        </button>
        <span class="spacer" />
        <button class="btn" type="button" @click="build.closeModal()">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: grid;
  place-items: center;
  background: rgb(0 0 0 / 55%);
}

.modal {
  display: flex;
  flex-direction: column;
  width: min(760px, calc(100vw - 80px));
  max-height: calc(100vh - 80px);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
  box-shadow: 0 24px 64px rgb(0 0 0 / 45%);
}

.head {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 44px;
  flex-shrink: 0;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
}

.head-icon {
  width: 16px;
  height: 16px;
  color: var(--theme-color);
}

.title {
  font-size: 13px;
  font-weight: 600;
}

.status {
  margin-left: 4px;
  color: var(--text-tertiary);
  font-size: 11px;
}

.status.ok {
  color: var(--success-color);
}

.status.bad {
  color: var(--error-color);
}

.close {
  margin-left: auto;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  font-size: 16px;
}

.close:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.bar {
  height: 3px;
  flex-shrink: 0;
  overflow: hidden;
  background: var(--bg-tertiary);
}

.bar-fill {
  height: 100%;
  background: var(--text-tertiary);
  transition: width 0.2s ease;
}

.bar-fill.ok {
  width: 100%;
  background: var(--success-color);
}

.bar-fill.bad {
  width: 100%;
  background: var(--error-color);
}

.bar-indeterminate {
  width: 35%;
  height: 100%;
  background: var(--theme-color);
  animation: slide 1.1s ease-in-out infinite;
}

@keyframes slide {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(320%);
  }
}

.summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex-shrink: 0;
  padding: 10px 12px 0;
}

.summary:empty {
  padding: 0;
}

.chip {
  padding: 2px 8px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  color: var(--text-secondary);
  font-size: 11px;
}

.log {
  flex: 1;
  min-height: 180px;
  max-height: 46vh;
  margin: 10px 12px;
  padding: 10px;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
}

.log:empty::before {
  content: "等待构建开始…";
  color: var(--text-tertiary);
}

.error {
  margin: 0 12px 8px;
  color: var(--error-color);
  font-size: 11px;
}

.actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  padding: 10px 12px;
  border-top: 1px solid var(--border-color);
}

.spacer {
  flex: 1;
}

.btn {
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
}

.btn:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn:disabled {
  opacity: 0.45;
}

.btn.primary {
  border-color: var(--theme-color);
  background: var(--theme-color);
  color: #fff;
}

.btn.danger {
  border-color: var(--error-color);
  color: var(--error-color);
}
</style>
