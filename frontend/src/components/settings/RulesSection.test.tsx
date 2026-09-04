import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { RulesSection } from "./RulesSection";
import type { Rule } from "../../lib/tauri";

const ruleA: Rule = {
  key: "spacing.cjk-latin",
  section: "空格",
  name: "中英文之间需要增加空格",
  description: "在中文与拉丁字母之间增加空格。",
  example: { before: "在LeanCloud上", after: "在 LeanCloud 上" },
  kind: "typography",
  risk: "safe",
  disputed: false,
  default: true,
};

describe("RulesSection", () => {
  it("规则卡片提供悬停示例提示并关联辅助描述", () => {
    const onToggle = vi.fn();
    render(
      <RulesSection
        rules={[ruleA]}
        enabledSet={new Set([ruleA.key])}
        onToggleRule={onToggle}
      />,
    );

    const card = screen.getByTestId("rule-card-spacing.cjk-latin");
    expect(card).toHaveAttribute(
      "title",
      "示例：“在LeanCloud上” → “在 LeanCloud 上”",
    );

    const checkbox = screen.getByTestId("rule-spacing.cjk-latin");
    expect(checkbox).toHaveAttribute("aria-describedby", "rule-example-spacing.cjk-latin");
    expect(
      screen.getByText("示例：“在LeanCloud上” → “在 LeanCloud 上”"),
    ).toBeInTheDocument();
  });

  it("点击规则复选框触发切换", async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    render(<RulesSection rules={[ruleA]} enabledSet={new Set()} onToggleRule={onToggle} />);

    await user.click(screen.getByTestId("rule-spacing.cjk-latin"));
    expect(onToggle).toHaveBeenCalledWith("spacing.cjk-latin");
  });
});