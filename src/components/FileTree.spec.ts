import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import FileTree from "./FileTree.vue";
import type { Entry } from "../stores/files";
import { useEditorStore } from "../stores/editor";

const mockedInvoke = vi.mocked(invoke);

const entries: Entry[] = [
  { name: "content", path: "content", kind: "dir" },
  { name: "hugo.toml", path: "hugo.toml", kind: "file" },
];

describe("FileTree", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "read_file") {
        return { path: "hugo.toml", content: "baseURL = '/'\n", size: 13, mtimeMs: 1 };
      }
      if (command === "file_meta") {
        return { size: 13, mtimeMs: 1 };
      }
      return [];
    });
  });

  it("渲染传入的目录与文件", () => {
    const wrapper = mount(FileTree, { props: { entries } });

    expect(wrapper.text()).toContain("content");
    expect(wrapper.text()).toContain("hugo.toml");
  });

  it("展开目录后加载并显示子项", async () => {
    mockedInvoke.mockResolvedValueOnce([
      { name: "posts", path: "content/posts", kind: "dir" },
    ] satisfies Entry[]);

    const wrapper = mount(FileTree, { props: { entries } });
    await wrapper.findAll("button.row")[0].trigger("click");
    await flushPromises();

    expect(mockedInvoke).toHaveBeenCalledWith("read_dir", { path: "content" });
    expect(wrapper.text()).toContain("posts");
  });

  it("点击文件后选中并在编辑器中打开", async () => {
    const wrapper = mount(FileTree, { props: { entries } });
    const editor = useEditorStore();

    await wrapper.findAll("button.row")[1].trigger("click");
    await flushPromises();

    expect(wrapper.findAll("button.row")[1].classes()).toContain("selected");
    expect(mockedInvoke).toHaveBeenCalledWith("read_file", { path: "hugo.toml" });
    expect(editor.activePath).toBe("hugo.toml");
  });
});
