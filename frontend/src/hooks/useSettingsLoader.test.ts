import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getUserSettings: vi.fn(),
  getSettingsPath: vi.fn(),
  getAppVersion: vi.fn(),
  getBuildCapabilities: vi.fn(),
}));

vi.mock("@/lib/tauri", () => ({
  DEFAULT_SHORTCUT_SETTINGS: {
    enabled: true,
    bindings: {
      format_now: "CtrlOrCmd+Enter",
      copy_output: "CtrlOrCmd+Shift+KeyC",
      open_settings: "CtrlOrCmd+Comma",
    },
  },
  getUserSettings: mocks.getUserSettings,
  getSettingsPath: mocks.getSettingsPath,
  getAppVersion: mocks.getAppVersion,
  getBuildCapabilities: mocks.getBuildCapabilities,
}));

import { useSettingsLoader } from "./useSettingsLoader";

const rules = [
  { key: "rule-a", section: "空格", name: "规则 A", example: { before: "a", after: "b" }, disputed: false, default: true },
  { key: "rule-b", section: "空格", name: "规则 B", example: { before: "a", after: "b" }, disputed: false, default: true },
];

describe("useSettingsLoader", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getUserSettings.mockResolvedValue(null);
    mocks.getSettingsPath.mockResolvedValue("/tmp/rules.yaml");
    mocks.getAppVersion.mockResolvedValue("0.5.0-test");
    mocks.getBuildCapabilities.mockResolvedValue({ simplifiedTradConversion: true });
  });

  it("恢复设置、过滤未知规则并通知输入恢复", async () => {
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-b", "unknown-rule"],
      last_input: "恢复内容",
      restore_last_input: true,
      theme: "dark",
      font: "pingfang",
      editor_font_size: "large",
      ui_scale: "small",
      output_mode: "manual",
      layout_mode: "vertical",
      shortcuts: {
        enabled: false,
        bindings: {
          format_now: "CtrlOrCmd+KeyF",
          copy_output: "CtrlOrCmd+Shift+KeyC",
          open_settings: "CtrlOrCmd+Comma",
        },
      },
      replacements: [
        { from: "TODO", to: "待办", active: true },
        { from: "FIXME", to: "修复", active: false },
      ],
      conversion: "s2t",
      notices: ["primary_settings_corrupt_recovered_from_backup"],
    });
    const onRestoreInput = vi.fn();
    const onLoadError = vi.fn();
    const { result } = renderHook(() => useSettingsLoader({ onRestoreInput, onLoadError }));

    await act(async () => {
      await result.current.loadSettings(rules, ["rule-a", "rule-b"]);
      await result.current.loadSettings(rules, ["rule-a"]);
    });

    expect(result.current.enabled).toEqual(["rule-b"]);
    expect(result.current.theme).toBe("dark");
    expect(result.current.font).toBe("pingfang");
    expect(result.current.editorFontSize).toBe("large");
    expect(result.current.uiScale).toBe("small");
    expect(result.current.outputMode).toBe("manual");
    expect(result.current.layoutMode).toBe("vertical");
    expect(result.current.shortcutsEnabled).toBe(false);
    expect(result.current.shortcutBindings.format_now).toBe("CtrlOrCmd+KeyF");
    expect(result.current.replacements).toEqual([
      { from: "TODO", to: "待办", active: true },
      { from: "FIXME", to: "修复", active: false },
    ]);
    expect(result.current.conversion).toBe("s2t");
    expect(result.current.settingsLoadNotices).toEqual([
      "primary_settings_corrupt_recovered_from_backup",
    ]);
    expect(result.current.settingsPath).toBe("/tmp/rules.yaml");
    expect(result.current.appVersion).toBe("0.5.0-test");
    expect(result.current.isHydrated()).toBe(true);
    expect(onRestoreInput).toHaveBeenCalledWith(
      "恢复内容",
      ["rule-b"],
      [
        { from: "TODO", to: "待办", active: true },
        { from: "FIXME", to: "修复", active: false },
      ],
      "s2t",
    );
    expect(mocks.getUserSettings).toHaveBeenCalledOnce();
    expect(onLoadError).not.toHaveBeenCalled();
  });

  it("没有已保存设置时恢复过滤后的默认规则", async () => {
    const onRestoreInput = vi.fn();
    const onLoadError = vi.fn();
    const { result } = renderHook(() => useSettingsLoader({ onRestoreInput, onLoadError }));

    await act(async () => {
      await result.current.loadSettings(rules, ["rule-a", "missing-rule"]);
    });

    expect(result.current.enabled).toEqual(["rule-a"]);
    expect(result.current.theme).toBe("system");
    expect(result.current.outputMode).toBe("realtime");
    expect(result.current.layoutMode).toBe("auto");
    expect(result.current.shortcutsEnabled).toBe(true);
    expect(result.current.isHydrated()).toBe(true);
    await waitFor(() => expect(mocks.getSettingsPath).toHaveBeenCalledOnce());
    expect(onRestoreInput).not.toHaveBeenCalled();
    expect(onLoadError).not.toHaveBeenCalled();
  });

  it("恢复旧设置时按构建能力归一化简繁转换", async () => {
    mocks.getBuildCapabilities.mockResolvedValue({ simplifiedTradConversion: false });
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-a"],
      last_input: "旧输入",
      restore_last_input: true,
      theme: "system",
      font: "system",
      editor_font_size: "normal",
      ui_scale: "normal",
      shortcuts: undefined,
      replacements: [],
      conversion: "t2s",
      notices: [],
    });
    const onRestoreInput = vi.fn();
    const onLoadError = vi.fn();
    const { result } = renderHook(() => useSettingsLoader({ onRestoreInput, onLoadError }));

    await act(async () => {
      await result.current.loadSettings(rules, ["rule-a"]);
    });

    expect(result.current.buildCapabilities).toEqual({ simplifiedTradConversion: false });
    expect(result.current.conversion).toBe("none");
    expect(onRestoreInput).toHaveBeenCalledWith("旧输入", ["rule-a"], [], "none");
  });

  it("旧设置缺少替换和转换字段时使用 GUI 默认值", async () => {
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-a"],
      last_input: "旧输入",
      restore_last_input: true,
      theme: "system",
      font: "system",
      editor_font_size: "normal",
      ui_scale: "normal",
      shortcuts: undefined,
      notices: [],
    });
    const onRestoreInput = vi.fn();
    const onLoadError = vi.fn();
    const { result } = renderHook(() => useSettingsLoader({ onRestoreInput, onLoadError }));

    await act(async () => {
      await result.current.loadSettings(rules, ["rule-a"]);
    });

    expect(result.current.replacements).toEqual([]);
    expect(result.current.conversion).toBe("none");
    expect(onRestoreInput).toHaveBeenCalledWith("旧输入", ["rule-a"], [], "none");
    expect(onLoadError).not.toHaveBeenCalled();
  });
});