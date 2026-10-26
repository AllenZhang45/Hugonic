<script setup lang="ts">
import Icon from "./Icon.vue";
</script>

<template>
  <section class="preview" aria-label="实时预览">
    <div class="head">
      <span class="panel-title">实时预览</span>
      <span class="url"><i class="dot" />localhost:1313</span>
      <div class="actions">
        <button class="icon-btn" disabled title="刷新（M3 接入）">
          <Icon name="refresh" class="icon" />
        </button>
        <button class="icon-btn" disabled title="在浏览器中打开（M3 接入）">
          <Icon name="external" class="icon" />
        </button>
        <button class="icon-btn" disabled title="切换设备（M3 接入）">
          <Icon name="device" class="icon" />
        </button>
      </div>
    </div>

    <div class="empty">
      <p class="empty-title">预览未启动</p>
      <p class="empty-hint">M3 会在这里用 iframe 加载 hugo server 的输出</p>
    </div>
  </section>
</template>

<style scoped>
.preview {
  display: flex;
  flex-direction: column;
  width: clamp(320px, 38%, 560px);
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
  border-radius: 999px;
  background: var(--bg-color);
  color: var(--text-tertiary);
  font-size: 11px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--text-tertiary);
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

.icon {
  width: 14px;
  height: 14px;
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
}
</style>
