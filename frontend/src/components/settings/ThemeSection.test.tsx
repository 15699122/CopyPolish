import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ThemeSection } from "./ThemeSection";

describe("ThemeSection", () => {
  const onThemeChange = vi.fn();
  const onFollowSystemChange = vi.fn();
  const onUiScaleChange = vi.fn();

  function renderTheme(theme: "system" | "light" | "dark") {
    return render(
      <ThemeSection
        theme={theme}
        onThemeChange={onThemeChange}
        onFollowSystemChange={onFollowSystemChange}
        uiScale="normal"
        onUiScaleChange={onUiScaleChange}
      />,
    );
  }

  it("三个主题选项使用统一等宽网格布局", () => {
    renderTheme("system");
    const options = screen.getByTestId("theme-options");
    expect(options).toHaveClass("grid-cols-3");
    expect(options.children).toHaveLength(3);
  });

  it("跟随系统时浅色和深色选项禁用", () => {
    renderTheme("system");
    expect(screen.getByTestId("theme-system")).toBeChecked();
    expect(screen.getByTestId("theme-light")).toBeDisabled();
    expect(screen.getByTestId("theme-dark")).toBeDisabled();
  });

  it("选择浅色触发浅色切换", async () => {
    const user = userEvent.setup();
    renderTheme("dark");
    await user.click(screen.getByTestId("theme-light"));
    expect(onThemeChange).toHaveBeenCalledWith("light");
  });
});