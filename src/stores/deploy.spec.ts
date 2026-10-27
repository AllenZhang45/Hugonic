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

import { invoke } from "@tauri-apps/api/core";
import { useDeployStore, type DeployDone } from "./deploy";

const mockedInvoke = vi.mocked(invoke);

describe("deploy store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockedInvoke.mockReset();
    eventState.handlers = {};
  });

  it("读取凭据状态", async () => {
    mockedInvoke.mockResolvedValue({ available: true, hasToken: true });
    const deploy = useDeployStore();

    await deploy.refreshCredentials();

    expect(deploy.credentialAvailable).toBe(true);
    expect(deploy.hasToken).toBe(true);
  });

  it("保存 token 后验证连接", async () => {
    mockedInvoke.mockImplementation(async (command: string) => {
      if (command === "netlify_credential_status") {
        return { available: true, hasToken: true };
      }
      if (command === "test_netlify_connection") {
        return { name: "Allen", email: "a@example.com" };
      }
      return undefined;
    });
    const deploy = useDeployStore();

    await deploy.saveToken("token-123");

    expect(mockedInvoke).toHaveBeenCalledWith("save_netlify_token", { token: "token-123" });
    expect(deploy.hasToken).toBe(true);
    expect(deploy.account?.name).toBe("Allen");
  });

  it("连接测试失败时记录错误", async () => {
    mockedInvoke.mockRejectedValue("Netlify token 无效或权限不足，请重新配置");
    const deploy = useDeployStore();

    const ok = await deploy.testConnection();

    expect(ok).toBe(false);
    expect(deploy.error).toContain("token 无效");
    expect(deploy.account).toBeNull();
  });

  it("部署开始后进入运行状态", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const deploy = useDeployStore();

    await deploy.start("demo-site");

    expect(deploy.status).toBe("running");
    expect(mockedInvoke).toHaveBeenCalledWith("start_deploy", { siteName: "demo-site" });
  });

  it("收到日志与成功事件后展示结果", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const deploy = useDeployStore();
    await deploy.start("demo-site");

    eventState.handlers["deploy://log"]({ payload: { line: "上传到 Netlify…" } });
    const done: DeployDone = {
      ok: true,
      url: "https://demo-site.netlify.app",
      siteName: "demo-site",
      deployId: "dep_1",
      error: null,
    };
    eventState.handlers["deploy://done"]({ payload: done });

    expect(deploy.logs).toContain("上传到 Netlify…");
    expect(deploy.status).toBe("done");
    expect(deploy.url).toBe("https://demo-site.netlify.app");
    expect(deploy.error).toBeNull();
  });

  it("部署失败事件切换到错误状态", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const deploy = useDeployStore();
    await deploy.start("demo-site");

    eventState.handlers["deploy://done"]({
      payload: {
        ok: false,
        url: null,
        siteName: "demo-site",
        deployId: null,
        error: "部署失败：站点名已被占用",
      },
    });

    expect(deploy.status).toBe("error");
    expect(deploy.error).toContain("已被占用");
  });

  it("启动失败时直接报错", async () => {
    mockedInvoke.mockRejectedValue("已有部署正在进行");
    const deploy = useDeployStore();

    await deploy.start("demo-site");

    expect(deploy.status).toBe("error");
    expect(deploy.error).toContain("已有部署");
  });

  it("清除 token 后回到未配置状态", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    const deploy = useDeployStore();
    deploy.hasToken = true;
    deploy.account = { name: "Allen", email: null };

    await deploy.clearToken();

    expect(deploy.hasToken).toBe(false);
    expect(deploy.account).toBeNull();
  });
});
