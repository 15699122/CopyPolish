import { useState } from "react";

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ReplacementsSection } from "./ReplacementsSection";

describe("ReplacementsSection", () => {
  function ControlledSection({
    initialReplacements,
    initialConversion = "none",
    onReplacementsChange,
    onConversionChange,
  }: {
    initialReplacements: { from: string; to: string; active: boolean }[];
    initialConversion?: "none" | "t2s" | "s2t";
    onReplacementsChange?: (next: { from: string; to: string; active: boolean }[]) => void;
    onConversionChange?: (next: "none" | "t2s" | "s2t") => void;
  }) {
    const [replacements, setReplacements] = useState(initialReplacements);
    const [conversion, setConversion] = useState(initialConversion);

    return (
      <ReplacementsSection
        replacements={replacements}
        conversion={conversion}
        onReplacementsChange={(next) => {
          setReplacements(next);
          onReplacementsChange?.(next);
        }}
        onConversionChange={(next) => {
          setConversion(next);
          onConversionChange?.(next);
        }}
      />
    );
  }

  it("添加、编辑、停用和删除替换项", async () => {
    const user = userEvent.setup();
    const onReplacementsChange = vi.fn();

    render(<ControlledSection initialReplacements={[]} onReplacementsChange={onReplacementsChange} />);

    await user.click(screen.getByTestId("replacement-add"));
    expect(onReplacementsChange).toHaveBeenLastCalledWith([
      { from: "", to: "", active: true },
    ]);

    await user.type(screen.getByTestId("replacement-from-0"), "TODO");
    expect(onReplacementsChange).toHaveBeenLastCalledWith([
      { from: "TODO", to: "", active: true },
    ]);

    await user.type(screen.getByTestId("replacement-to-0"), "待办");
    expect(onReplacementsChange).toHaveBeenLastCalledWith([
      { from: "TODO", to: "待办", active: true },
    ]);

    await user.click(screen.getByTestId("replacement-active-0"));
    expect(onReplacementsChange).toHaveBeenLastCalledWith([
      { from: "TODO", to: "待办", active: false },
    ]);

    await user.click(screen.getByTestId("replacement-remove-0"));
    expect(onReplacementsChange).toHaveBeenLastCalledWith([]);
  });

  it("按列表顺序显示替换项并切换简繁转换", async () => {
    const user = userEvent.setup();
    const onConversionChange = vi.fn();
    const replacements = [
      { from: "A", to: "甲", active: true },
      { from: "B", to: "乙", active: false },
    ];

    render(
      <ControlledSection
        initialReplacements={replacements}
        onConversionChange={onConversionChange}
      />,
    );

    expect(screen.getByTestId("replacement-from-0")).toHaveValue("A");
    expect(screen.getByTestId("replacement-from-1")).toHaveValue("B");
    expect(
      screen.getByTestId("replacement-from-0").compareDocumentPosition(screen.getByTestId("replacement-from-1")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(screen.getByTestId("replacement-active-1")).not.toBeChecked();

    await user.selectOptions(screen.getByTestId("conversion-select"), "s2t");
    expect(onConversionChange).toHaveBeenCalledWith("s2t");
    expect(screen.getByTestId("conversion-select")).toHaveValue("s2t");
  });
});