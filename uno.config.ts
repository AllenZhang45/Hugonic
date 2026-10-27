import { defineConfig, presetIcons, presetUno } from "unocss";
import { ICON_CLASSES } from "./src/components/icons";

export default defineConfig({
  presets: [
    presetUno(),
    presetIcons({
      mode: "mask",
      scale: 1,
      extraProperties: {
        display: "inline-block",
        "vertical-align": "middle",
      },
    }),
  ],
  // 图标类名是动态取值的，显式加入 safelist，避免依赖扫描结果。
  safelist: Object.values(ICON_CLASSES),
  theme: {
    colors: {
      canvas: "var(--bg-color)",
      surface: "var(--bg-secondary)",
      raised: "var(--bg-tertiary)",
      line: "var(--border-color)",
      brand: "var(--theme-color)",
      primary: "var(--text-primary)",
      secondary: "var(--text-secondary)",
      muted: "var(--text-tertiary)",
    },
  },
});
