<script setup lang="ts">
import { ICON_CLASSES, type IconName } from "./icons";

defineProps<{ name: IconName }>();
</script>

<template>
  <span :class="ICON_CLASSES[name]" aria-hidden="true" />
</template>
