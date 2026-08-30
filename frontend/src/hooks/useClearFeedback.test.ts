import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useClearFeedback } from "./useClearFeedback";

describe("useClearFeedback", () => {
  it("触发全部清理动作并在超时后清除完成反馈", () => {
    vi.useFakeTimers();
    const clearInput = vi.fn();
    const clearOutput = vi.fn();
    const cancelFormat = vi.fn();
    const clearError = vi.fn();
    const persistEmptyInput = vi.fn();

    const { result } = renderHook(() =>
      useClearFeedback({
        clearInput,
        clearOutput,
        cancelFormat,
        clearError,
        persistEmptyInput,
        durationMs: 1200,
      }),
    );

    expect(result.current.cleared).toBe(false);
    act(() => result.current.clear());
    expect(clearInput).toHaveBeenCalledOnce();
    expect(clearOutput).toHaveBeenCalledOnce();
    expect(cancelFormat).toHaveBeenCalledOnce();
    expect(clearError).toHaveBeenCalledOnce();
    expect(persistEmptyInput).toHaveBeenCalledOnce();
    expect(result.current.cleared).toBe(true);

    act(() => vi.advanceTimersByTime(1200));
    expect(result.current.cleared).toBe(false);
    vi.useRealTimers();
  });

  it("卸载时清理待执行的反馈定时器", () => {
    vi.useFakeTimers();
    const { result, unmount } = renderHook(() =>
      useClearFeedback({
        clearInput: vi.fn(),
        clearOutput: vi.fn(),
        cancelFormat: vi.fn(),
        clearError: vi.fn(),
        persistEmptyInput: vi.fn(),
        durationMs: 1200,
      }),
    );

    act(() => result.current.clear());
    unmount();
    act(() => vi.runAllTimers());
    vi.useRealTimers();
  });
});