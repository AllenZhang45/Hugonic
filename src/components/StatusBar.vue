<script setup lang="ts">
import { computed } from "vue";
import { useProjectStore } from "../stores/project";

const project = useProjectStore();

const projectLabel = computed(() => project.info?.name ?? "未打开项目");
const hugoLabel = computed(() => {
  const version = project.info?.hugoVersion;
  return version ? `Hugo ${version}` : "Hugo —";
});
</script>

<template>
  <footer class="statusbar">
    <div class="group">
      <span class="item"><i class="dot" :class="{ ready: project.info }" />{{ projectLabel }}</span>
      <span class="item">hugo server 未启动</span>
    </div>
    <div class="group">
      <span class="item">{{ hugoLabel }}</span>
      <span class="item">UTF-8</span>
      <span class="item">Ln 1, Col 1</span>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 26px;
  flex-shrink: 0;
  padding: 0 12px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: var(--text-tertiary);
  font-size: 11px;
}

.group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.item {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--warning-color);
}

.dot.ready {
  background: var(--success-color);
}
</style>
