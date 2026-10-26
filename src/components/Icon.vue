<script setup lang="ts">
const paths: Record<string, string[]> = {
  edit: ["M12 20h9", "M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"],
  dashboard: ["M3 3h7v7H3z", "M14 3h7v4h-7z", "M14 11h7v10h-7z", "M3 14h7v7H3z"],
  media: ["M3 5h18v14H3z", "M8.5 10a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z", "M21 15l-5-5L5 21"],
  theme: [
    "M12 3a9 9 0 1 0 0 18c1.7 0 2-1.3 1.2-2.2-.9-1.1-.2-2.8 1.3-2.8H18a3 3 0 0 0 3-3 9 9 0 0 0-9-9Z",
    "M7.5 9h.01",
    "M10.5 6.5h.01",
    "M15 7h.01",
    "M7.5 14h.01",
  ],
  analytics: ["M3 3v18h18", "M7 15l4-4 3 3 5-6"],
  settings: [
    "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z",
    "M12 2v3",
    "M12 19v3",
    "M4.2 4.2l2.1 2.1",
    "M17.7 17.7l2.1 2.1",
    "M2 12h3",
    "M19 12h3",
    "M4.2 19.8l2.1-2.1",
    "M17.7 6.3l2.1-2.1",
  ],
  folder: ["M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"],
  file: ["M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z", "M14 2v6h6"],
  search: ["M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14Z", "M20 20l-4-4"],
  terminal: ["M4 17l6-6-6-6", "M12 19h8"],
  upload: ["M12 16V3", "M7 8l5-5 5 5", "M4 16v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3"],
  refresh: ["M21 12a9 9 0 1 1-3-6.7", "M21 3v6h-6"],
  external: [
    "M14 3h7v7",
    "M21 3l-9 9",
    "M10 5H5a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5",
  ],
  build: ["M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.3 2.3-3-3z"],
  chevron: ["M9 6l6 6-6 6"],
  device: ["M7 2h10a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1Z", "M12 18h.01"],
};

defineProps<{ name: string }>();
</script>

<template>
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path v-for="d in paths[name] ?? []" :key="d" :d="d" />
  </svg>
</template>
