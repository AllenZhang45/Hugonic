<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Icon from "./Icon.vue";
import { mediaReference, useMediaStore, type MediaItem } from "../stores/media";

const props = defineProps<{ item: MediaItem }>();
const media = useMediaStore();
const copied = ref(false);

onMounted(() => {
  void media.ensureThumbnail(props.item);
});

const thumbnail = computed(() => media.thumbnails[props.item.path]);
const reference = computed(() => mediaReference(props.item.path));

const size = computed(() => {
  const bytes = props.item.size;
  if (bytes >= 1024 * 1024) {
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${Math.round(bytes / 1024)} KB`;
  }
  return `${bytes} B`;
});

async function copyReference() {
  try {
    await navigator.clipboard.writeText(reference.value);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1200);
  } catch {
    copied.value = false;
  }
}
</script>

<template>
  <article class="card">
    <div class="preview">
      <img v-if="thumbnail" :src="thumbnail" :alt="item.name" loading="lazy" />
      <div v-else class="placeholder">
        <Icon :name="item.extension === 'svg' ? 'media' : 'file'" class="placeholder-icon" />
      </div>
    </div>
    <div class="body">
      <p class="name" :title="item.name">{{ item.name }}</p>
      <p class="meta">{{ size }} · {{ item.extension.toUpperCase() }}</p>
      <p class="dir" :title="item.path">{{ item.dir }}</p>
    </div>
    <div class="actions">
      <button type="button" :title="`复制 ${reference}`" @click="copyReference">
        {{ copied ? "已复制" : "复制引用" }}
      </button>
      <button type="button" @click="media.open(item)">打开</button>
    </div>
  </article>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
}

.preview {
  aspect-ratio: 4 / 3;
  overflow: hidden;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-tertiary);
}

.preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.placeholder {
  display: grid;
  height: 100%;
  place-items: center;
}

.placeholder-icon {
  width: 26px;
  height: 26px;
  color: var(--border-color);
}

.body {
  padding: 8px 10px 0;
}

.name {
  margin: 0;
  overflow: hidden;
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.meta {
  margin: 3px 0 0;
  color: var(--text-tertiary);
  font-size: 10px;
}

.dir {
  margin: 2px 0 0;
  overflow: hidden;
  color: var(--text-tertiary);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 10px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.actions {
  display: flex;
  gap: 5px;
  padding: 8px 10px 10px;
}

.actions button {
  flex: 1;
  height: 24px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
}

.actions button:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
</style>
