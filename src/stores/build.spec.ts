import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

const eventState = vi.hoisted(() => ({
  handlers: {} as Record<string, (event: { payload: unknown }) => void>,
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    eventState.handlers[name] = handler;
    return () => {};
  }),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openPath: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { useBuildStore, type BuildDone } from "./build";

const mockedInvoke = vi.mocked(invoke);

const donePayload: BuildDone = {
  code: 0,
  durationMs: 287,
  cancelled: false,
  summary: { pages: 24, staticFiles: 47, processedImages: 12, durationMs: 287 },
};

describe("build store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
    eventState.handlers = {};
  });

  it("启动后进入运行状态并打开弹窗", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const build = useBuildStore();

    await build.start();

    expect(build.running).toBe(true);
    expect(build.modalOpen).toBe(true);
    expect(mockedInvoke).toHaveBeenCalledWith("start_build");
  });

  it("接收日志与完成事件", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const build = useBuildStore();
    await build.start();

    eventState.handlers["build://log"]({ payload: { line: "Building sites …", stream: "stdout" } });
    eventState.handlers["build://done"]({ payload: donePayload });

    expect(build.logs).toHaveLength(1);
    expect(build.running).toBe(false);
    expect(build.exitCode).toBe(0);
    expect(build.durationMs).toBe(287);
    expect(build.summary?.pages).toBe(24);
  });

  it("构建失败时记录退出码", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const build = useBuildStore();
    await build.start();

    eventState.handlers["build://done"]({
      payload: {
        ...donePayload,
        code: 1,
        summary: { pages: null, staticFiles: null, processedImages: null, durationMs: null },
      },
    });

    expect(build.exitCode).toBe(1);
    expect(build.summary?.pages).toBeNull();
  });

  it("取消调用后端命令", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const build = useBuildStore();
    await build.start();

    await build.cancel();

    expect(mockedInvoke).toHaveBeenCalledWith("cancel_build");
  });

  it("启动失败时退出运行状态并记录错误", async () => {
    mockedInvoke.mockRejectedValue("已有构建正在进行");
    const build = useBuildStore();

    await build.start();

    expect(build.running).toBe(false);
    expect(build.error).toContain("已有构建");
  });

  it("运行中重复点击不会重复启动", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const build = useBuildStore();
    await build.start();

    await build.start();

    expect(mockedInvoke).toHaveBeenCalledTimes(1);
  });
});
