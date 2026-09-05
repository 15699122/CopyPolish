import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(),
  minimize: vi.fn(),
  toggleMaximize: vi.fn(),
  close: vi.fn(),
  startDragging: vi.fn(),
}));

vi.mock("@/lib/tauri", () => ({
  isTauri: mocks.isTauri,
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: mocks.minimize,
    toggleMaximize: mocks.toggleMaximize,
    close: mocks.close,
    startDragging: mocks.startDragging,
  }),
}));

import { useWindowControls } from "./useWindowControls";

function mouseEvent(overrides: Partial<React.MouseEvent<HTMLElement>> = {}) {
  return {
    button: 0,
    detail: 1,
    target: document.createElement("div"),
    ...overrides,
  } as React.MouseEvent<HTMLElement>;
}

describe("useWindowControls", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.isTauri.mockReturnValue(false);
    mocks.minimize.mockResolvedValue(undefined);
    mocks.toggleMaximize.mockResolvedValue(undefined);
    mocks.close.mockResolvedValue(undefined);
    mocks.startDragging.mockResolvedValue(undefined);
  });

  it("浏览器预览模式下窗口动作均为 no-op", async () => {
    const onError = vi.fn();
    const { result } = renderHook(() => useWindowControls({ onError }));

    await act(async () => {
      await result.current.onMinimize();
      await result.current.onToggleMaximize();
      await result.current.onClose();
      result.current.onHeaderMouseDown(mouseEvent());
    });

    expect(mocks.minimize).not.toHaveBeenCalled();
    expect(mocks.toggleMaximize).not.toHaveBeenCalled();
    expect(mocks.close).not.toHaveBeenCalled();
    expect(mocks.startDragging).not.toHaveBeenCalled();
    expect(onError).not.toHaveBeenCalled();
  });

  it("Tauri 模式调用窗口动作并过滤非拖动标题栏事件", async () => {
    mocks.isTauri.mockReturnValue(true);
    const onError = vi.fn();
    const { result } = renderHook(() => useWindowControls({ onError }));

    await act(async () => {
      await result.current.onMinimize();
      await result.current.onToggleMaximize();
      await result.current.onClose();
    });
    expect(mocks.minimize).toHaveBeenCalledOnce();
    expect(mocks.toggleMaximize).toHaveBeenCalledOnce();
    expect(mocks.close).toHaveBeenCalledOnce();

    result.current.onHeaderMouseDown(mouseEvent());
    result.current.onHeaderMouseDown(mouseEvent({ button: 1 }));
    result.current.onHeaderMouseDown(mouseEvent({ detail: 2 }));
    const control = document.createElement("button");
    control.setAttribute("data-window-control", "true");
    result.current.onHeaderMouseDown(mouseEvent({ target: control }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(mocks.startDragging).toHaveBeenCalledOnce();
    expect(onError).not.toHaveBeenCalled();
  });

  it("窗口动作或拖动失败时上报带上下文的错误", async () => {
    mocks.isTauri.mockReturnValue(true);
    mocks.minimize.mockRejectedValue(new Error("minimize failed"));
    mocks.startDragging.mockRejectedValue(new Error("drag failed"));
    const onError = vi.fn();
    const { result } = renderHook(() => useWindowControls({ onError }));

    await act(async () => {
      await result.current.onMinimize();
    });
    result.current.onHeaderMouseDown(mouseEvent());
    await act(async () => {
      await Promise.resolve();
    });

    expect(onError).toHaveBeenNthCalledWith(1, "窗口操作失败：Error: minimize failed");
    expect(onError).toHaveBeenNthCalledWith(2, "窗口拖动失败：Error: drag failed");
  });
});