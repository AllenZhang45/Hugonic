import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { usePreviewStore } from "./preview";

const mockedInvoke = vi.mocked(invoke);
const mockedOpenUrl = vi.mocked(openUrl);

describe("preview store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
    mockedOpenUrl.mockReset();
  });

  it("启动成功后记录 URL 与端口", async () => {
    mockedInvoke.mockResolvedValue({ url: "http://127.0.0.1:1314", port: 1314 });
    const preview = usePreviewStore();

    await preview.start();

    expect(preview.status).toBe("running");
    expect(preview.url).toBe("http://127.0.0.1:1314");
    expect(preview.port).toBe(1314);
    expect(preview.running).toBe(true);
    expect(mockedInvoke).toHaveBeenCalledWith("start_preview");
  });

  it("重复启动不会并发请求", async () => {
    mockedInvoke.mockResolvedValue({ url: "http://127.0.0.1:1313", port: 1313 });
    const preview = usePreviewStore();

    await preview.start();
    await preview.start();

    expect(mockedInvoke).toHaveBeenCalledTimes(1);
  });

  it("启动失败时记录错误", async () => {
    mockedInvoke.mockRejectedValue("等待 hugo server 就绪超时");
    const preview = usePreviewStore();

    await preview.start();

    expect(preview.status).toBe("error");
    expect(preview.error).toContain("超时");
    expect(preview.running).toBe(false);
  });

  it("停止后回到空闲状态", async () => {
    mockedInvoke.mockResolvedValue({ url: "http://127.0.0.1:1313", port: 1313 });
    const preview = usePreviewStore();
    await preview.start();

    await preview.stop();

    expect(preview.status).toBe("idle");
    expect(preview.url).toBeNull();
    expect(mockedInvoke).toHaveBeenCalledWith("stop_preview");
  });

  it("设备在桌面与移动之间切换", () => {
    const preview = usePreviewStore();

    expect(preview.device).toBe("desktop");
    preview.toggleDevice();
    expect(preview.device).toBe("mobile");
  });

  it("用系统浏览器打开预览地址", async () => {
    mockedInvoke.mockResolvedValue({ url: "http://127.0.0.1:1313", port: 1313 });
    const preview = usePreviewStore();
    await preview.start();

    await preview.openExternal();

    expect(mockedOpenUrl).toHaveBeenCalledWith("http://127.0.0.1:1313");
  });

  it("刷新只递增 reloadToken", () => {
    const preview = usePreviewStore();
    const before = preview.reloadToken;

    preview.reload();

    expect(preview.reloadToken).toBe(before + 1);
  });
});
