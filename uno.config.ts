import { defineConfig, presetUno } from "unocss";

export default defineConfig({
  presets: [presetUno()],
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
