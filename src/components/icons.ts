/** Material Symbols（经 Iconify）图标类名映射。类名必须保持完整字面量，UnoCSS 才能静态提取。 */
export const ICON_CLASSES = {
  edit: "i-material-symbols:edit-note",
  dashboard: "i-material-symbols:dashboard",
  media: "i-material-symbols:image",
  theme: "i-material-symbols:palette",
  analytics: "i-material-symbols:analytics",
  settings: "i-material-symbols:settings",
  folder: "i-material-symbols:folder",
  file: "i-material-symbols:description",
  search: "i-material-symbols:search",
  chevron: "i-material-symbols:chevron-right",
  terminal: "i-material-symbols:terminal",
  build: "i-material-symbols:build",
  upload: "i-material-symbols:upload",
  refresh: "i-material-symbols:refresh",
  external: "i-material-symbols:open-in-new",
  device: "i-material-symbols:devices",
} as const;

export type IconName = keyof typeof ICON_CLASSES;
