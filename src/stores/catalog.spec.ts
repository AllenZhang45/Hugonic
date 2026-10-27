import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCatalogStore, type CatalogCache, type CatalogTheme } from "./catalog";

const mockedInvoke = vi.mocked(invoke);
const mockedListen = vi.mocked(listen);
const handlers: Record<string, (event: { payload: unknown }) => void> = {};

const theme: CatalogTheme = {
  slug: "ananke",
  name: "Ananke",
  authors: ["Bud Parr"],
  description: "Demo",
  tags: ["blog"],
  license: "MIT",
  stars: 1397,
  updated: "2026-09-26",
  repo: "https://github.com/gohugo-ananke/ananke",
  demo: null,
  screenshotUrl: null,
  installCommands: ["git submodule add https://github.com/gohugo-ananke/ananke themes/ananke"],
  installExtracted: false,
  installSource: null,
  repoUpdatedAt: "2026-10-01T00:00:00Z",
};

function cache(fetchedAt: number): CatalogCache {
  return {
    schema: 1,
    fetchedAt,
    checkedAt: fetchedAt,
    complete: true,
    source: "https://themes.gohugo.io/",
    themes: [theme],
  };
}

function mockCatalog(current: CatalogCache | null, started = false) {
  mockedInvoke.mockImplementation(async (command: string) => {
    if (command === "load_theme_catalog") return current;
    if (command === "theme_catalog_running") return false;
    if (command === "start_theme_sync") return started;
    if (command === "theme_catalog_progress") {
      return { phase: "idle", done: 0, total: 0, failed: 0, message: "" };
    }
    if (command === "catalog_screenshot") return "data:image/png;base64,abc";
    return null;
  });
}

describe("catalog store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
    mockedListen.mockReset();
    for (const key of Object.keys(handlers)) {
      delete handlers[key];
    }
    mockedListen.mockImplementation(async (event: string, handler: unknown) => {
      handlers[event] = handler as (event: { payload: unknown }) => void;
      return () => {};
    });
  });

  it("缓存新鲜时不会自动同步", async () => {
    mockCatalog(cache(Math.floor(Date.now() / 1000)));
    const store = useCatalogStore();

    await store.init();

    expect(store.data?.themes).toHaveLength(1);
    expect(mockedInvoke.mock.calls.some(([command]) => command === "start_theme_sync")).toBe(false);
  });

  it("缓存过期时自动同步", async () => {
    const old = Math.floor(Date.now() / 1000) - 8 * 24 * 3600;
    mockCatalog(cache(old), true);
    const store = useCatalogStore();

    await store.init();

    expect(mockedInvoke).toHaveBeenCalledWith("start_theme_sync", { force: false });
    expect(store.running).toBe(true);
  });

  it("没有缓存时自动同步", async () => {
    mockCatalog(null, true);
    const store = useCatalogStore();

    await store.init();

    expect(mockedInvoke).toHaveBeenCalledWith("start_theme_sync", { force: false });
  });

  it("截图按主题缓存", async () => {
    mockCatalog(cache(Math.floor(Date.now() / 1000)));
    const store = useCatalogStore();
    await store.init();

    await store.loadScreenshot("ananke");
    await store.loadScreenshot("ananke");

    expect(
      mockedInvoke.mock.calls.filter(([command]) => command === "catalog_screenshot"),
    ).toHaveLength(1);
    expect(store.screenshots.ananke).toContain("base64");
  });

  it("接收同步进度与完成事件", async () => {
    mockCatalog(cache(Math.floor(Date.now() / 1000)));
    const store = useCatalogStore();
    await store.init();

    handlers["theme-catalog://progress"]({
      payload: { phase: "details", done: 10, total: 100, failed: 1, message: "抓取中" },
    });
    expect(store.running).toBe(true);
    expect(store.progress?.done).toBe(10);

    handlers["theme-catalog://done"]({ payload: { ok: true } });
    expect(store.running).toBe(false);
    expect(store.data?.themes).toHaveLength(1);
  });
});
