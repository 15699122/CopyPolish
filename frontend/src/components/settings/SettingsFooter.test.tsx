import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

import { SettingsFooter } from "./SettingsFooter";

describe("SettingsFooter", () => {
  const writeText = vi.fn();
  const onSetAll = vi.fn();
  const onResetDefaults = vi.fn();
  const onOpenChange = vi.fn();

  const WIN_PATH = "C:\\src\\CopyPolish\\rules.yaml";

  function renderFooter(settingsPath: string | null = null) {
    return render(
      <SettingsFooter
        appVersion="0.6.0-test"
        settingsStatus="saved"
        settingsError={null}
        settingsLoadNotices={[]}
        settingsPath={settingsPath}
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
});