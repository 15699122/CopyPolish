import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useFormatter } from "./useFormatter";

const mocks = vi.hoisted(() => ({
  formatText: vi.fn(),
}));

vi.mock("@/lib/tauri", () => ({
  formatText: mocks.formatText,
  normalizeCommandError: (cause: unknown) => ({
    code: "internal",
    message: cause instanceof Error ? cause.message : "操作失败，请检查输入后重试。",
  }),
}));

const getSelection = (enabled: string[]) => ({ mode: "only" as const, keys: enabled });

describe("useFormatter", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("立即执行排版并传递当前规则选择", async () => {
    mocks.formatText.mockResolvedValue("formatted");
    const { result } = renderHook(() => useFormatter({ getSelection }));

    act(() => {
      result.current.scheduleFormat("原文", ["rule-a"], 0);
    });

    await waitFor(() => expect(result.current.output).toBe("formatted"));
    expect(mocks.formatText).toHaveBeenCalledWith({
      text: "原文",
      selection: { mode: "only", keys: ["rule-a"] },
    });
    expect(result.current.error).toBeNull();
    expect(result.current.isFormatting).toBe(false);
  });

  it("透传有序替换和简繁转换选项", async () => {
    mocks.formatText.mockResolvedValue("formatted");
    const { result } = renderHook(() => useFormatter({ getSelection }));

    act(() => {
      result.current.scheduleFormat(
        "原文",
        ["rule-a"],
        0,
        { replacements: [{ from: "A", to: "甲", active: true }], conversion: "s2t" },
      );
    });

    await waitFor(() => expect(result.current.output).toBe("formatted"));
    expect(mocks.formatText).toHaveBeenCalledWith({
      text: "原文",
      selection: { mode: "only", keys: ["rule-a"] },
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });
  });

  it("旧请求完成后不会覆盖较新的请求结果", async () => {
    let resolveFirst!: (value: string) => void;
    let resolveSecond!: (value: string) => void;
    mocks.formatText
      .mockImplementationOnce(() => new Promise<string>((resolve) => { resolveFirst = resolve; }))
      .mockImplementationOnce(() => new Promise<string>((resolve) => { resolveSecond = resolve; }));
    const { result } = renderHook(() => useFormatter({ getSelection }));

    act(() => {
      result.current.scheduleFormat("first", ["rule-a"], 0);
    });
    await waitFor(() => expect(mocks.formatText).toHaveBeenCalledTimes(1));

    act(() => {
      result.current.scheduleFormat("second", ["rule-b"], 0);
    });
    await waitFor(() => expect(mocks.formatText).toHaveBeenCalledTimes(2));
    resolveFirst("first-result");
    resolveSecond("second-result");

    await waitFor(() => expect(result.current.output).toBe("second-result"));
  });

  it("记录格式化错误并支持取消待执行任务", async () => {
    mocks.formatText.mockRejectedValue(new Error("engine failed"));
    const { result } = renderHook(() => useFormatter({ getSelection }));

    act(() => {
      result.current.scheduleFormat("待取消", [], 1000);
      result.current.cancelFormat();
    });
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(mocks.formatText).not.toHaveBeenCalled();

    act(() => {
      result.current.scheduleFormat("失败", [], 0);
    });
    await waitFor(() => expect(result.current.error).toContain("engine failed"));
    expect(result.current.isFormatting).toBe(false);
  });
});