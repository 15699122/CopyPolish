import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { HelpDialog } from "./HelpDialog";

describe("HelpDialog", () => {
  it("通过帮助入口打开静态说明并可关闭", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();

    render(<HelpDialog open={false} onOpenChange={onOpenChange} />);
    await user.click(screen.getByTestId("open-help"));

    expect(onOpenChange).toHaveBeenCalledWith(true);
  });

  it("展示规则风险、结构保护和演示模式边界", () => {
    render(<HelpDialog open onOpenChange={vi.fn()} />);

    expect(screen.getByTestId("help-content")).toHaveTextContent("先检查规则风险");
    expect(screen.getByTestId("help-content")).toHaveTextContent("结构内容会受到保护");
    expect(screen.getByTestId("help-content")).toHaveTextContent("浏览器演示模式的边界");
    expect(screen.getByTestId("help-done")).toBeInTheDocument();
  });
});