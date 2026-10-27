import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { mediaReference, useMediaStore, type MediaItem } from "./media";
import { useProjectStore } from "./project";

const mockedInvoke = vi.mocked(invoke);

const item: MediaItem = {
  path: "static/images/cover.png",
  name: "cover.png",
  dir: "static/images",
  size: 2048,
  modifiedMs: 1,
  extension: "png",
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

describe("mediaReference", () => {
  it("static 转成站点绝对路径", () => {
    expect(mediaReference("static/images/a.png")).toBe("/images/a.png");
  });

  it("content bundle 只保留文件名", () => {
    expect(mediaReference("content/posts/hello/cover.jpg")).toBe("cover.jpg");
  });

  it("assets 保持相对路径", () => {
    expect(mediaReference("assets/logo.svg")).toBe("assets/logo.svg");
  });
});

describe("media store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
  });

  it("加载媒体列表", async () => {
    withProject();
    mockedInvoke.mockResolvedValue([item]);
    const store = useMediaStore();

    await store.load();

    expect(store.items).toHaveLength(1);
    expect(mockedInvoke).toHaveBeenCalledWith("list_media");
  });

  it("缩略图只请求一次", async () => {
    withProject();
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "list_media") return [item];
      if (command === "media_thumbnail") return "data:image/jpeg;base64,abc";
      return undefined;
    });
    const store = useMediaStore();
    await store.load();

    await store.ensureThumbnail(item);
    await store.ensureThumbnail(item);

    expect(
      mockedInvoke.mock.calls.filter(([command]) => command === "media_thumbnail"),
    ).toHaveLength(1);
    expect(store.thumbnails[item.path]).toContain("base64");
  });

  it("缩略图失败时记录空占位", async () => {
    withProject();
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "list_media") return [item];
      if (command === "media_thumbnail") throw "解码失败";
      return undefined;
    });
    const store = useMediaStore();
    await store.load();

    await store.ensureThumbnail(item);

    expect(store.thumbnails[item.path]).toBe("");
  });

  it("导入后刷新列表", async () => {
    withProject();
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "list_media") return [item];
      if (command === "import_media") return item;
      return undefined;
    });
    const store = useMediaStore();
    await store.load();

    const count = await store.importPaths(["/tmp/a.png", "/tmp/b.png"], "static/images");

    expect(count).toBe(2);
    expect(mockedInvoke).toHaveBeenCalledWith("import_media", {
      source: "/tmp/a.png",
      targetDir: "static/images",
    });
    expect(mockedInvoke.mock.calls.filter(([command]) => command === "list_media")).toHaveLength(2);
  });

  it("没有项目时不请求", async () => {
    const store = useMediaStore();

    await store.load();

    expect(mockedInvoke).not.toHaveBeenCalled();
  });
});
