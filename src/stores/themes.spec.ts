import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "./project";
import { useThemesStore, type ThemeList } from "./themes";

const mockedInvoke = vi.mocked(invoke);

const payload: ThemeList = {
  configFile: "hugo.toml",
  canSwitch: true,
  reason: null,
  themes: [
    {
      dir: "ananke",
      name: "Ananke",
      description: "Demo",
      author: "Bud",
      license: "MIT",
      homepage: null,
      minVersion: "0.128.0",
      active: true,
      hasScreenshot: true,
    },
    {
      dir: "other",
      name: "Other",
      description: null,
      author: null,
      license: null,
      homepage: null,
      minVersion: null,
      active: false,
      hasScreenshot: false,
    },
  ],
};

function withProject() {
  const project = useProjectStore();
  project.info = {
    root: "/tmp/site",
    name: "site",
    configFile: "hugo.toml",
    hugoVersion: "0.168.0",
  };
}

describe("themes store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
  });

  it("加载主题列表并获取截图", async () => {
    withProject();
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "list_themes") return payload;
      if (command === "theme_screenshot") return "data:image/png;base64,abc";
      return undefined;
    });
    const store = useThemesStore();

    await store.load();

    expect(store.data?.themes).toHaveLength(2);
    expect(store.data?.canSwitch).toBe(true);
    expect(store.screenshots.ananke).toContain("base64");
    expect(store.screenshots.other).toBeUndefined();
  });

  it("同一项目缓存列表", async () => {
    withProject();
    mockedInvoke.mockResolvedValue(payload);
    const store = useThemesStore();

    await store.load();
    await store.load();

    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("应用主题后重新加载列表", async () => {
    withProject();
    mockedInvoke.mockResolvedValue(payload);
    const store = useThemesStore();
    await store.load();

    const changed = await store.apply("other");

    expect(changed).toBe(true);
    expect(mockedInvoke).toHaveBeenCalledWith("apply_theme", { dir: "other" });
    expect(mockedInvoke.mock.calls.filter(([command]) => command === "list_themes")).toHaveLength(
      2,
    );
  });

  it("应用失败时记录错误", async () => {
    withProject();
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "list_themes") return payload;
      if (command === "apply_theme") throw "当前站点使用 YAML 配置";
      return undefined;
    });
    const store = useThemesStore();
    await store.load();

    const changed = await store.apply("other");

    expect(changed).toBe(false);
    expect(store.error).toContain("YAML");
  });

  it("没有项目时不请求", async () => {
    const store = useThemesStore();

    await store.load();

    expect(mockedInvoke).not.toHaveBeenCalled();
  });
});
