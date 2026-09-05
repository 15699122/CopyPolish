import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useThemeAndFont } from "./useThemeAndFont";

function createMediaQuery(matches: boolean) {
  return {
    matches,
    media: "(prefers-color-scheme: dark)",
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  };
}

describe("useThemeAndFont", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.removeAttribute("style");
    vi.restoreAllMocks();
  });

  it("应用显式主题、字体、字号和缩放令牌", () => {
    const mediaQuery = createMediaQuery(false);
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: vi.fn(() => mediaQuery),
    });

    renderHook(() =>
      useThemeAndFont({
        theme: "dark",
        font: "pingfang",
        editorFontSize: "large",
        uiScale: "small",
      }),
    );

    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement.style.getPropertyValue("--app-font-family")).toContain("PingFang SC");
    expect(document.documentElement.style.getPropertyValue("--editor-font-size")).toBe("16px");
    expect(document.documentElement.style.getPropertyValue("--editor-line-height")).toBe("1.75");
    expect(document.documentElement.style.getPropertyValue("--app-ui-scale")).toBe("0.9");
    expect(mediaQuery.addEventListener).not.toHaveBeenCalled();
  });

  it("system 主题跟随媒体变化并在卸载时移除监听", () => {
    const mediaQuery = createMediaQuery(false);
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: vi.fn(() => mediaQuery),
    });
    const { unmount } = renderHook(() =>
      useThemeAndFont({
        theme: "system",
        font: "system",
        editorFontSize: "normal",
        uiScale: "normal",
      }),
    );

    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    expect(mediaQuery.addEventListener).toHaveBeenCalledWith("change", expect.any(Function));
    const applyTheme = mediaQuery.addEventListener.mock.calls[0][1] as () => void;
    mediaQuery.matches = true;
    act(() => applyTheme());
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");

    unmount();
    expect(mediaQuery.removeEventListener).toHaveBeenCalledWith("change", applyTheme);
  });
});