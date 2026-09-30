import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

import { SettingsFooter } from "./SettingsFooter";

describe("SettingsFooter", () => {
  const writeText = vi.fn();
  const onSetAll = vi.fn();
  const onResetDefaults = vi.fn();
  const onOpenChange = vi.fn();

  const WIN_PATH = "C:\\src\\CopyPolish\\rules.yaml";

  function renderFooter(
    settingsPath: string | null = null,
    ruleActions?: { showRuleActions: boolean; enabledCount?: number; rulesCount?: number },
  ) {
    return render(
      <SettingsFooter
        appVersion="0.6.0-test"
        settingsStatus="saved"
        settingsError={null}
        settingsLoadNotices={[]}
        settingsPath={settingsPath}
        {...ruleActions}
        onSetAll={onSetAll}
        onResetDefaults={onResetDefaults}
        onOpenChange={onOpenChange}
      />,
    );
  }

  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    writeText.mockResolvedValue(undefined);
  });

  it("仅显示文件名 rules.yaml，完整路径保留在标题和复制内容中", () => {
    renderFooter(WIN_PATH);
    const pathEl = screen.getByTestId("settings-path");
    expect(pathEl).toHaveTextContent("rules.yaml");
    expect(pathEl).toHaveAttribute("type", "button");
    expect(pathEl).toHaveAttribute("title", WIN_PATH);
    expect(pathEl).toHaveAttribute("aria-label", `点击复制设置文件完整路径：${WIN_PATH}`);
  });

  it("点击复制完整路径并显示成功反馈", async () => {
    renderFooter(WIN_PATH);
    fireEvent.click(screen.getByTestId("settings-path"));
    expect(writeText).toHaveBeenCalledWith(WIN_PATH);
    expect(await screen.findByText("路径已复制")).toBeInTheDocument();
  });

  it("复制失败时显示失败反馈", async () => {
    writeText.mockRejectedValueOnce(new Error("denied"));
    renderFooter(WIN_PATH);
    fireEvent.click(screen.getByTestId("settings-path"));
    expect(await screen.findByText("复制失败")).toBeInTheDocument();
  });

  it("没有路径时正常渲染且不显示路径按钮", () => {
    renderFooter(null);
    expect(screen.queryByTestId("settings-path")).not.toBeInTheDocument();
    expect(screen.getByTestId("settings-version")).toHaveTextContent("版本 0.6.0-test");
  });

  it("规则分类显示批量操作与启用计数，并使用明确的恢复默认规则文案", () => {
    renderFooter(null, { showRuleActions: true, enabledCount: 12, rulesCount: 30 });
    expect(screen.getByTestId("select-all")).toBeInTheDocument();
    expect(screen.getByTestId("select-none")).toBeInTheDocument();
    expect(screen.getByTestId("reset-defaults")).toHaveTextContent("恢复默认规则");
    expect(screen.getByTestId("rules-enabled-count")).toHaveTextContent("已启用 12/30 条");
  });

  it("非规则分类不渲染规则批量操作，避免恢复默认误改规则", () => {
    renderFooter(null, { showRuleActions: false });
    expect(screen.queryByTestId("select-all")).not.toBeInTheDocument();
    expect(screen.queryByTestId("select-none")).not.toBeInTheDocument();
    expect(screen.queryByTestId("reset-defaults")).not.toBeInTheDocument();
    expect(screen.queryByTestId("rules-enabled-count")).not.toBeInTheDocument();
    // 完成按钮在所有分类都可用。
    expect(screen.getByTestId("settings-done")).toBeInTheDocument();
  });

  it("规则分类点击恢复默认只触发规则恢复回调", () => {
    renderFooter(null, { showRuleActions: true });
    fireEvent.click(screen.getByTestId("reset-defaults"));
    expect(onResetDefaults).toHaveBeenCalledTimes(1);
    expect(onSetAll).not.toHaveBeenCalled();
  });
});