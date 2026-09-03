import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { PresetsSection } from "./PresetsSection";

const presets = [
  {
    key: "copywriting",
    name: "中文文案",
    description: "默认排版",
    selection: { mode: "defaults" as const },
    replacements: [],
    conversion: "none" as const,
  },
  {
    key: "pdf-cleaning",
    name: "PDF 清洗",
    description: "清洗 PDF 复制文本",
    selection: { mode: "only" as const, keys: ["cleanup.reference-square"] },
    replacements: [],
    conversion: "none" as const,
  },
];

describe("PresetsSection", () => {
  it("显示内置预设并回调应用动作", async () => {
    const user = userEvent.setup();
    const onApplyPreset = vi.fn();
    render(<PresetsSection presets={presets} onApplyPreset={onApplyPreset} />);

    expect(screen.getByTestId("preset-list")).toBeInTheDocument();
    expect(screen.getByText("中文文案")).toBeInTheDocument();
    expect(screen.getByText("PDF 清洗")).toBeInTheDocument();

    await user.click(screen.getByTestId("preset-apply-pdf-cleaning"));
    expect(onApplyPreset).toHaveBeenCalledWith(presets[1]);
  });

  it("没有预设时显示浏览器演示提示", () => {
    render(<PresetsSection onApplyPreset={vi.fn()} />);
    expect(screen.getByTestId("presets-empty")).toHaveTextContent("浏览器演示模式");
  });
});