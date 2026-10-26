<script setup lang="ts">
import { computed } from "vue";
import Icon from "./Icon.vue";
import { useFilesStore, type Entry } from "../stores/files";

const props = defineProps<{
  entries: Entry[];
  depth?: number;
}>();

const files = useFilesStore();
const depth = computed(() => props.depth ?? 0);

function activate(entry: Entry) {
  if (entry.kind === "dir") {
    void files.toggleDir(entry.path);
  } else {
    files.select(entry.path);
  }
}
</script>

<template>
  <ul class="tree" role="tree">
    <li v-for="entry in entries" :key="entry.path" class="node">
      <button
        type="button"
        class="row"
        :class="{ selected: files.selected === entry.path }"
        :style="{ paddingLeft: `${8 + depth * 12}px` }"
        @click="activate(entry)"
      >
        <span
          v-if="entry.kind === 'dir'"
          class="chevron"
          :class="{ open: files.expanded.includes(entry.path) }"
        >
          <Icon name="chevron" class="chevron-icon" />
        </span>
        <span v-else class="chevron" />

        <Icon
          :name="entry.kind === 'dir' ? 'folder' : 'file'"
          class="entry-icon"
          :class="{ dir: entry.kind === 'dir' }"
        />
        <span class="label">{{ entry.name }}</span>
      </button>

      <FileTree
        v-if="entry.kind === 'dir' && files.expanded.includes(entry.path)"
        :entries="files.entries[entry.path] ?? []"
        :depth="depth + 1"
      />
    </li>
  </ul>
</template>

<style scoped>
.tree {
  margin: 0;
  padding: 0;
  list-style: none;
}

.row {
  display: flex;
  align-items: center;
  gap: 5px;
  width: 100%;
  height: 24px;
  padding-right: 8px;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  text-align: left;
}

.row:hover {
  background: var(--bg-tertiary);
}

.row.selected {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.chevron {
  display: grid;
  place-items: center;
  width: 12px;
  height: 12px;
  color: var(--text-tertiary);
  transition: transform 0.12s ease;
}

.chevron.open {
  transform: rotate(90deg);
}

.chevron-icon {
  width: 12px;
  height: 12px;
}

.entry-icon {
  width: 14px;
  height: 14px;
  color: var(--text-tertiary);
}

.entry-icon.dir {
  color: var(--theme-color);
  opacity: 0.75;
}

.label {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
</style>
