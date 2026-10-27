import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { useInsightsStore, type Insights } from "./insights";
import { useProjectStore } from "./project";

const mockedInvoke = vi.mocked(invoke);

const insights: Insights = {
  configFile: "hugo.toml",
  title: "Demo",
  baseUrl: "https://example.org/",
  theme: "ananke",
  language: "zh-cn",
  counts: { pages: 3, posts: 1, drafts: 1, media: 1, themes: 1 },
  words: 8,
  sections: [{ name: "posts", pages: 1, drafts: 1, words: 3 }],
  tags: [{ name: "hugo", count: 2 }],
  categories: [],
  recent: [{ path: "content/posts/first.md", name: "first.md", modifiedMs: 1 }],
  missingTitle: 1,
  missingDate: 1,
};

function withProject() {
  const project = useProjectStore();
  project.info = {
    root: "/tmp/site",
    name: "site",
    configFile: "hugo.toml",
    hugoVersion: "0.168.0",
  };
  return project;
}

describe("insights store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
  });

  it("没有打开项目时不发请求", async () => {
    const store = useInsightsStore();

    await store.load();

    expect(mockedInvoke).not.toHaveBeenCalled();
    expect(store.data).toBeNull();
  });

  it("加载项目洞察并缓存同一项目", async () => {
    withProject();
    mockedInvoke.mockResolvedValue(insights);
    const store = useInsightsStore();

    await store.load();
    await store.load();

    expect(mockedInvoke).toHaveBeenCalledTimes(1);
    expect(store.data?.counts.pages).toBe(3);
  });

  it("force 会重新扫描", async () => {
    withProject();
    mockedInvoke.mockResolvedValue(insights);
    const store = useInsightsStore();

    await store.load();
    await store.load(true);

    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("失败时记录错误并清空数据", async () => {
    withProject();
    mockedInvoke.mockRejectedValue("读取 content 失败");
    const store = useInsightsStore();

    await store.load();

    expect(store.error).toContain("读取 content");
    expect(store.data).toBeNull();
  });
});
