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

    let copied: boolean | undefined;
    await act(async () => {
      copied = await result.current.copy();
    });
    expect(copied).toBe(true);
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
    let emptyCopied: boolean | undefined;
    await act(async () => {
      emptyCopied = await empty.result.current.copy();
    });
    expect(emptyCopied).toBe(false);
    expect(writeText).not.toHaveBeenCalled();

    empty.unmount();
    writeText.mockRejectedValueOnce(new Error("clipboard denied"));
    const failed = renderHook(() =>
      useClipboardStatus({ getText: () => "结果", onError, resetMs: 100 }),
    );
    let failedCopied: boolean | undefined;
    await act(async () => {
      failedCopied = await failed.result.current.copy();
    });
    expect(failedCopied).toBe(false);
    await waitFor(() => expect(onError).toHaveBeenCalledWith(expect.any(Error)));
    expect(failed.result.current.copied).toBe(false);
  });
});