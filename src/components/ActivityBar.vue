<script setup lang="ts">
import Icon from "./Icon.vue";
import { useUiStore, type ViewId } from "../stores/ui";
import type { IconName } from "./icons";

const ui = useUiStore();

const items: { id: ViewId; icon: IconName; label: string }[] = [
  { id: "content", icon: "edit", label: "内容管理" },
  { id: "dashboard", icon: "dashboard", label: "仪表盘" },
  { id: "media", icon: "media", label: "媒体库" },
  { id: "themes", icon: "theme", label: "主题" },
  { id: "analytics", icon: "analytics", label: "分析" },
];
</script>

<template>
  <nav class="activity-bar" aria-label="主导航">
    <button
      v-for="item in items"
      :key="item.id"
      class="activity-item"
      :class="{ 'is-active': ui.activeView === item.id }"
      :title="item.label"
      :aria-label="item.label"
      :aria-pressed="ui.activeView === item.id"
      @click="ui.setActiveView(item.id)"
    >
      <Icon :name="item.icon" class="activity-icon" />
    </button>

    <div class="bottom">
      <button class="activity-item" disabled title="设置（后续版本）">
        <Icon name="settings" class="activity-icon" />
      </button>
      <div class="avatar" title="账户（后续版本）">A</div>
    </div>
  </nav>
</template>

<style scoped>
.activity-bar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  width: 48px;
  flex-shrink: 0;
  padding: 8px 0;
  border-right: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.activity-item {
  display: grid;
  place-items: center;
  width: 36px;
  height: 36px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
}

.activity-item:hover:not(:disabled) {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.activity-item.is-active {
  color: var(--theme-color);
  background: var(--bg-tertiary);
}

.activity-icon {
  width: 18px;
  height: 18px;
}

.bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: auto;
}

.avatar {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  border-radius: var(--radius-full);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 11px;
}
</style>
