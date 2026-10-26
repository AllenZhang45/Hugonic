<script setup lang="ts">
import Icon from "./Icon.vue";
</script>

<template>
  <header class="titlebar">
    <div class="brand">
      <span class="logo">H</span>
      <span class="name">Hugonic</span>
    </div>
    <div class="project">
      <Icon name="folder" class="icon" />
      <span>未打开项目</span>
    </div>
    <div class="actions">
      <button class="title-btn" disabled title="命令面板（M2 接入）">
        <Icon name="terminal" class="icon" />命令
      </button>
      <button class="title-btn" disabled title="文件搜索（M2 接入）">
        <Icon name="search" class="icon" />搜索
      </button>
      <button class="title-btn primary" disabled title="部署（M5 接入）">
        <Icon name="upload" class="icon" />部署
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 44px;
  flex-shrink: 0;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.logo {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border-radius: var(--radius-sm);
  background: var(--theme-color);
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}

.name {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.project {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-left: 10px;
  border-left: 1px solid var(--border-color);
  color: var(--text-tertiary);
  font-size: 12px;
}

.actions {
  display: flex;
  gap: 4px;
  margin-left: auto;
}

.icon {
  width: 14px;
  height: 14px;
}

.title-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
}

.title-btn:disabled {
  opacity: 0.55;
}

.title-btn.primary {
  background: var(--theme-color);
  border-color: var(--theme-color);
  color: #fff;
}
</style>
