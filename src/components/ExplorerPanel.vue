<script setup lang="ts">
import Icon from "./Icon.vue";
</script>

<template>
  <aside class="explorer" aria-label="资源管理器">
    <div class="head">
      <span class="panel-title">资源管理器</span>
      <button class="icon-btn" disabled title="新建文件（M1 接入）">
        <Icon name="file" class="icon" />
      </button>
    </div>

    <div class="search">
      <Icon name="search" class="search-icon" />
      <input class="search-input" type="search" placeholder="搜索文件…" disabled />
    </div>

    <div class="empty">
      <Icon name="folder" class="empty-icon" />
      <p class="empty-title">尚未打开项目</p>
      <p class="empty-hint">打开一个 Hugo 站点后，目录树会显示在这里</p>
      <button class="open-btn" disabled title="M1 接入">打开站点</button>
    </div>
  </aside>
</template>

<style scoped>
.explorer {
  display: flex;
  flex-direction: column;
  width: 260px;
  flex-shrink: 0;
  border-right: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 34px;
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
  width: 15px;
  height: 15px;
}

.search {
  position: relative;
  padding: 8px 10px;
}

.search-icon {
  position: absolute;
  top: 50%;
  left: 18px;
  width: 13px;
  height: 13px;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 26px;
  padding: 0 8px 0 26px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-color);
  color: var(--text-secondary);
  font: inherit;
  font-size: 12px;
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin-top: 48px;
  padding: 0 20px;
  text-align: center;
}

.empty-icon {
  width: 28px;
  height: 28px;
  color: var(--border-color);
}

.empty-title {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.empty-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-tertiary);
}

.open-btn {
  margin-top: 8px;
  height: 26px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 12px;
}

.open-btn:disabled {
  opacity: 0.55;
}
</style>
