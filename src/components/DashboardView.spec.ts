import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => () => {}) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openPath: vi.fn(), openUrl: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import DashboardView from "./DashboardView.vue";
import { useProjectStore } from "../stores/project";

const mockedInvoke = vi.mocked(invoke);

const payload = {
  configFile: "hugo.toml",
  title: "Demo",
  baseUrl: "https://example.org/",
  theme: "ananke",
  language: "zh-cn",
  counts: { pages: 11, posts: 6, drafts: 2, media: 3, themes: 1 },
  words: 1234,
  sections: [],
  tags: [],
  categories: [],
  recent: [],
  missingTitle: 0,
  missingDate: 0,
};

describe("DashboardView", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
    mockedInvoke.mockResolvedValue(payload);
    useProjectStore().info = {
      root: "/tmp/demo-site",
      name: "demo-site",
      configFile: "hugo.toml",
      hugoVersion: "0.168.0",
    };
  });

  it("展示站点与内容统计", async () => {
    const wrapper = mount(DashboardView);
    await flushPromises();

    expect(wrapper.text()).toContain("Demo");
    expect(wrapper.text()).toContain("ananke");
    expect(wrapper.text()).toContain("11");
  });

  it("无项目时显示空状态", async () => {
    useProjectStore().info = null;

    const wrapper = mount(DashboardView);
    await flushPromises();

    expect(wrapper.text()).toContain("尚未打开项目");
  });
});
