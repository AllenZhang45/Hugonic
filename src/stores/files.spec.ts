import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { useFilesStore, type Entry } from "./files";

const mockedInvoke = vi.mocked(invoke);

const rootEntries: Entry[] = [
  { name: "content", path: "content", kind: "dir" },
  { name: "hugo.toml", path: "hugo.toml", kind: "file" },
];

const contentEntries: Entry[] = [{ name: "posts", path: "content/posts", kind: "dir" }];

describe("files store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
  });

  it("每个目录只加载一次并缓存结果", async () => {
    mockedInvoke.mockResolvedValueOnce(rootEntries);
    const files = useFilesStore();

    await files.loadDir("");
    await files.loadDir("");

    expect(mockedInvoke).toHaveBeenCalledTimes(1);
    expect(files.entries[""]).toEqual(rootEntries);
  });

  it("展开目录时才请求子项", async () => {
    mockedInvoke.mockResolvedValueOnce(contentEntries);
    const files = useFilesStore();

    expect(mockedInvoke).not.toHaveBeenCalled();

    await files.toggleDir("content");

    expect(mockedInvoke).toHaveBeenCalledWith("read_dir", { path: "content" });
    expect(files.expanded).toContain("content");
    expect(files.entries["content"]).toEqual(contentEntries);
  });

  it("收起目录不会重新请求", async () => {
    mockedInvoke.mockResolvedValueOnce(contentEntries);
    const files = useFilesStore();

    await files.toggleDir("content");
    await files.toggleDir("content");

    expect(files.expanded).not.toContain("content");
    expect(mockedInvoke).toHaveBeenCalledTimes(1);
  });

  it("记录选中的文件", () => {
    const files = useFilesStore();

    files.select("hugo.toml");

    expect(files.selected).toBe("hugo.toml");
  });

  it("读取失败时记录错误且不写入缓存", async () => {
    mockedInvoke.mockRejectedValueOnce("无法读取目录：Permission denied");
    const files = useFilesStore();

    await expect(files.toggleDir("secret")).rejects.toBeTruthy();

    expect(files.error).toContain("Permission denied");
    expect(files.entries["secret"]).toBeUndefined();
    expect(files.expanded).not.toContain("secret");
  });
});
