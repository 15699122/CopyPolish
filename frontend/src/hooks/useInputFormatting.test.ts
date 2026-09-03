import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useInputFormatting } from "./useInputFormatting";

describe("useInputFormatting", () => {
  it("输入变化时更新值、立即调度格式化并防抖保存设置", () => {
    const scheduleFormat = vi.fn();
    const schedulePersist = vi.fn();
    const { result } = renderHook(() =>
      useInputFormatting({
        enabled: ["rule-a", "rule-b"],
        replacements: [{ from: "A", to: "甲", active: true }],
        conversion: "s2t",
        scheduleFormat,
        schedulePersist,
      }),
    );

    act(() => result.current.onInputChange("新输入"));

    expect(result.current.input).toBe("新输入");
    expect(scheduleFormat).toHaveBeenCalledWith("新输入", ["rule-a", "rule-b"], undefined, {
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });
    expect(schedulePersist).toHaveBeenCalledWith({
      enabled: ["rule-a", "rule-b"],
      last_input: "新输入",
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });
  });

  it("暴露的 setInput 可用于恢复历史输入且不会触发新的调度", () => {
    const scheduleFormat = vi.fn();
    const schedulePersist = vi.fn();
    const { result } = renderHook(() =>
      useInputFormatting({
        enabled: [],
        replacements: [],
        conversion: "none",
        scheduleFormat,
        schedulePersist,
      }),
    );

    act(() => result.current.setInput("历史输入"));

    expect(result.current.input).toBe("历史输入");
    expect(scheduleFormat).not.toHaveBeenCalled();
    expect(schedulePersist).not.toHaveBeenCalled();
  });

  it("手动模式输入变化只保存输入，不自动调度排版", () => {
    const scheduleFormat = vi.fn();
    const schedulePersist = vi.fn();
    const { result } = renderHook(() =>
      useInputFormatting({
        enabled: ["rule-a"],
        outputMode: "manual",
        scheduleFormat,
        schedulePersist,
      }),
    );

    act(() => result.current.onInputChange("等待手动排版"));

    expect(result.current.input).toBe("等待手动排版");
    expect(scheduleFormat).not.toHaveBeenCalled();
    expect(schedulePersist).toHaveBeenCalledWith({
      enabled: ["rule-a"],
      last_input: "等待手动排版",
      replacements: [],
      conversion: "none",
    });
  });
});