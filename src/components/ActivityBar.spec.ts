import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import ActivityBar from "./ActivityBar.vue";
import { useUiStore } from "../stores/ui";

describe("ActivityBar", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("渲染五个视图入口和一个设置入口", () => {
    const wrapper = mount(ActivityBar);

    expect(wrapper.findAll("button.activity-item")).toHaveLength(6);
  });

  it("点击媒体库后更新 store 并高亮该入口", async () => {
    const wrapper = mount(ActivityBar);
    const ui = useUiStore();

    await wrapper.findAll("button.activity-item")[2].trigger("click");

    expect(ui.activeView).toBe("media");
    expect(wrapper.findAll("button.activity-item")[2].classes()).toContain("is-active");
  });
});
