import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useClipboardStatus } from "./useClipboardStatus";

describe("useClipboardStatus", () => {
  const writeText = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    writeText.mockResolvedValue(undefined);
  });

  it("复制当前文本并在延迟后清除成功状态", async () => {
    vi.useFakeTimers();
    const onError = vi.fn();
    const { result } = renderHook(() =>
      useClipboardStatus({ getText: () => "结果", onError, resetMs: 100 }),
    );

    await act(async () => {
      await result.current.copy();
    });
    expect(writeText).toHaveBeenCalledWith("结果");
    expect(result.current.copied).toBe(true);

    act(() => vi.advanceTimersByTime(100));
    expect(result.current.copied).toBe(false);
    expect(onError).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it("空文本不复制，写入失败时通过回调上报错误", async () => {
    const onError = vi.fn();
    const empty = renderHook(() =>
      useClipboardStatus({ getText: () => "", onError, resetMs: 100 }),
    );
    await act(async () => {
      await empty.result.current.copy();
    });
    expect(writeText).not.toHaveBeenCalled();

    empty.unmount();
    writeText.mockRejectedValueOnce(new Error("clipboard denied"));
    const failed = renderHook(() =>
      useClipboardStatus({ getText: () => "结果", onError, resetMs: 100 }),
    );
    await act(async () => {
      await failed.result.current.copy();
    });
    await waitFor(() => expect(onError).toHaveBeenCalledWith(expect.any(Error)));
    expect(failed.result.current.copied).toBe(false);
  });
});