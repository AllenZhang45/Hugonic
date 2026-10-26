import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import PreviewPane from "./PreviewPane.vue";
import { usePreviewStore } from "../stores/preview";

describe("PreviewPane", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("运行中用 iframe 加载预览地址", async () => {
    const preview = usePreviewStore();
    preview.status = "running";
    preview.url = "http://127.0.0.1:1314";

    const wrapper = mount(PreviewPane);

    expect(wrapper.find("iframe").attributes("src")).toBe("http://127.0.0.1:1314");
    expect(wrapper.text()).toContain("127.0.0.1:1314");
  });

  it("刷新只改变 iframe 的 src（带缓存参数），不重建节点", async () => {
    const preview = usePreviewStore();
    preview.status = "running";
    preview.url = "http://127.0.0.1:1314";
    const wrapper = mount(PreviewPane);
    const before = wrapper.find("iframe").element;

    preview.reload();
    await wrapper.vm.$nextTick();

    expect(wrapper.find("iframe").element).toBe(before);
    expect(wrapper.find("iframe").attributes("src")).toBe(
      "http://127.0.0.1:1314/?hugonic_reload=1",
    );
  });

  it("未启动时显示空状态与启动按钮", () => {
    const preview = usePreviewStore();
    preview.status = "idle";

    const wrapper = mount(PreviewPane);

    expect(wrapper.find("iframe").exists()).toBe(false);
    expect(wrapper.text()).toContain("预览未启动");
  });
});
