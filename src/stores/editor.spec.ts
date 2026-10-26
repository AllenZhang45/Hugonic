import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { isDirty, languageForPath, useEditorStore } from "./editor";

const mockedInvoke = vi.mocked(invoke);

const PATH = "content/post.md";

function mockFile(content = "# 标题\n", mtimeMs = 1000) {
  mockedInvoke.mockImplementation(async (command: string) => {
    if (command === "read_file") {
      return { path: PATH, content, size: content.length, mtimeMs };
    }
    if (command === "save_file") {
      return { size: content.length, mtimeMs: mtimeMs + 1 };
    }
    if (command === "file_meta") {
      return { size: content.length, mtimeMs };
    }
    return [];
  });
}

describe("editor store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
  });

  it("打开文件后创建标签并识别语言", async () => {
    mockFile();
    const editor = useEditorStore();

    await editor.openFile(PATH);

    expect(editor.tabs).toHaveLength(1);
    expect(editor.activePath).toBe(PATH);
    expect(editor.activeTab?.language).toBe("markdown");
    expect(editor.activeTab?.content).toBe("# 标题\n");
  });

  it("修改后标记为脏，保存成功后清除", async () => {
    mockFile();
    const editor = useEditorStore();
    await editor.openFile(PATH);

    editor.updateContent(PATH, "# 新标题\n");

    expect(isDirty(editor.activeTab!)).toBe(true);

    const saved = await editor.save();

    expect(saved).toBe(true);
    expect(isDirty(editor.activeTab!)).toBe(false);
    expect(mockedInvoke).toHaveBeenCalledWith("save_file", {
      path: PATH,
      content: "# 新标题\n",
    });
  });

  it("保存失败时保留修改并记录错误", async () => {
    mockFile();
    const editor = useEditorStore();
    await editor.openFile(PATH);
    editor.updateContent(PATH, "# 新标题\n");
    mockedInvoke.mockRejectedValueOnce("文件是只读的，无法保存");

    const saved = await editor.save();

    expect(saved).toBe(false);
    expect(editor.error).toContain("只读");
    expect(isDirty(editor.activeTab!)).toBe(true);
  });

  it("关闭当前标签后激活相邻标签", async () => {
    mockFile();
    const editor = useEditorStore();
    await editor.openFile("content/a.md");
    await editor.openFile("content/b.md");

    await editor.close("content/b.md");

    expect(editor.activePath).toBe("content/a.md");
  });

  it("磁盘变更且本地无修改时自动重载", async () => {
    mockFile("old", 1000);
    const editor = useEditorStore();
    await editor.openFile(PATH);

    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "file_meta") {
        return { size: 3, mtimeMs: 2000 };
      }
      if (command === "read_file") {
        return { path: PATH, content: "new", size: 3, mtimeMs: 2000 };
      }
      return [];
    });

    await editor.checkExternal(PATH);

    expect(editor.activeTab?.content).toBe("new");
    expect(editor.activeTab?.mtimeMs).toBe(2000);
  });

  it("磁盘变更且本地有修改时标记冲突", async () => {
    mockFile("old", 1000);
    const editor = useEditorStore();
    await editor.openFile(PATH);
    editor.updateContent(PATH, "mine");

    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "file_meta") {
        return { size: 3, mtimeMs: 2000 };
      }
      return [];
    });

    await editor.checkExternal(PATH);

    expect(editor.activeTab?.conflict).toBe(true);
    expect(editor.activeTab?.content).toBe("mine");
  });

  it("按扩展名映射语言", () => {
    expect(languageForPath("hugo.toml")).toBe("ini");
    expect(languageForPath("config.yaml")).toBe("yaml");
    expect(languageForPath("layout.html")).toBe("html");
    expect(languageForPath("unknown.bin")).toBe("plaintext");
  });
});
