import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { OutputSection } from "./OutputSection";

describe("OutputSection", () => {
  it("切换输出模式和布局并回调最新值", async () => {
    const user = userEvent.setup();
    const onOutputModeChange = vi.fn();
    const onLayoutModeChange = vi.fn();

    render(
      <OutputSection
        outputMode="realtime"
        layoutMode="auto"
        onOutputModeChange={onOutputModeChange}
        onLayoutModeChange={onLayoutModeChange}
      />,
    );

    await user.selectOptions(screen.getByTestId("output-mode-select"), "manual");
    await user.selectOptions(screen.getByTestId("layout-mode-select"), "vertical");

    expect(onOutputModeChange).toHaveBeenCalledWith("manual");
    expect(onLayoutModeChange).toHaveBeenCalledWith("vertical");
  });
});