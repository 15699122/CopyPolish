import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useSettingsDialog } from "./useSettingsDialog";

describe("useSettingsDialog", () => {
  it("切换 open 状态并在关闭后恢复触发按钮焦点", () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useSettingsDialog());
    const trigger = document.createElement("button");
    document.body.appendChild(trigger);
    result.current.triggerRef.current = trigger;

    act(() => result.current.onOpenChange(true));
    expect(result.current.open).toBe(true);

    act(() => result.current.onOpenChange(false));
    expect(result.current.open).toBe(false);
    expect(document.activeElement).not.toBe(trigger);

    act(() => vi.runAllTimers());
    expect(trigger).toHaveFocus();

    trigger.remove();
    vi.useRealTimers();
  });

  it("卸载时清理待执行的焦点恢复任务", () => {
    vi.useFakeTimers();
    const { result, unmount } = renderHook(() => useSettingsDialog());
    const trigger = document.createElement("button");
    document.body.appendChild(trigger);
    result.current.triggerRef.current = trigger;

    act(() => result.current.onOpenChange(false));
    unmount();
    act(() => vi.runAllTimers());

    expect(trigger).not.toHaveFocus();
    trigger.remove();
    vi.useRealTimers();
  });
});