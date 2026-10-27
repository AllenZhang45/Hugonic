<script setup lang="ts">
import type { IconName } from "./icons";
import Icon from "./Icon.vue";

defineProps<{ title: string; icon: IconName; description: string }>();
</script>

<template>
  <section class="view" :aria-label="title">
    <div class="placeholder">
      <Icon :name="icon" class="icon" />
      <h2>{{ title }}</h2>
      <p>{{ description }}</p>
    </div>
  </section>
</template>

<style scoped>
.view {
  display: grid;
  flex: 1;
  place-items: center;
  background: var(--bg-color);
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.icon {
  width: 28px;
  height: 28px;
  color: var(--border-color);
}

h2 {
  margin: 0;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 600;
}

p {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 12px;
}
</style>
