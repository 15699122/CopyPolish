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
        scheduleFormat,
        schedulePersist,
      }),
    );

    act(() => result.current.onInputChange("新输入"));

    expect(result.current.input).toBe("新输入");
    expect(scheduleFormat).toHaveBeenCalledWith("新输入", ["rule-a", "rule-b"]);
    expect(schedulePersist).toHaveBeenCalledWith({
      enabled: ["rule-a", "rule-b"],
      last_input: "新输入",
    });
  });

  it("暴露的 setInput 可用于恢复历史输入且不会触发新的调度", () => {
    const scheduleFormat = vi.fn();
    const schedulePersist = vi.fn();
    const { result } = renderHook(() =>
      useInputFormatting({ enabled: [], scheduleFormat, schedulePersist }),
    );

    act(() => result.current.setInput("历史输入"));

    expect(result.current.input).toBe("历史输入");
    expect(scheduleFormat).not.toHaveBeenCalled();
    expect(schedulePersist).not.toHaveBeenCalled();
  });
});