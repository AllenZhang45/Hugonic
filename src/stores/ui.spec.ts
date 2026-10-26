import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import { useUiStore } from "./ui";

describe("ui store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("默认停在内容管理视图", () => {
    expect(useUiStore().activeView).toBe("content");
  });

  it("切换视图后状态更新", () => {
    const ui = useUiStore();

    ui.setActiveView("media");

    expect(ui.activeView).toBe("media");
  });

  it("预览设备在桌面端与移动端之间切换", () => {
    const ui = useUiStore();
    expect(ui.previewDevice).toBe("desktop");

    ui.togglePreviewDevice();

    expect(ui.previewDevice).toBe("mobile");
  });
});
