import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getUserSettings: vi.fn(),
  getSettingsPath: vi.fn(),
  getAppVersion: vi.fn(),
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
}));

import { useSettingsLoader } from "./useSettingsLoader";

const rules = [
  { key: "rule-a", section: "空格", name: "规则 A", disputed: false, default: true },
  { key: "rule-b", section: "空格", name: "规则 B", disputed: false, default: true },
];

describe("useSettingsLoader", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getUserSettings.mockResolvedValue(null);
    mocks.getSettingsPath.mockResolvedValue("/tmp/rules.yaml");
    mocks.getAppVersion.mockResolvedValue("0.5.0-test");
  });

  it("恢复设置、过滤未知规则并通知输入恢复", async () => {
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-b", "unknown-rule"],
      last_input: "恢复内容",
      theme: "dark",
      font: "pingfang",
      editor_font_size: "large",
      ui_scale: "small",
      shortcuts: {
        enabled: false,
        bindings: {
          format_now: "CtrlOrCmd+KeyF",
          copy_output: "CtrlOrCmd+Shift+KeyC",
          open_settings: "CtrlOrCmd+Comma",
        },
      },
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
    expect(result.current.shortcutsEnabled).toBe(false);
    expect(result.current.shortcutBindings.format_now).toBe("CtrlOrCmd+KeyF");
    expect(result.current.settingsLoadNotices).toEqual([
      "primary_settings_corrupt_recovered_from_backup",
    ]);
    expect(result.current.settingsPath).toBe("/tmp/rules.yaml");
    expect(result.current.appVersion).toBe("0.5.0-test");
    expect(result.current.isHydrated()).toBe(true);
    expect(onRestoreInput).toHaveBeenCalledWith("恢复内容", ["rule-b"]);
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
    expect(result.current.shortcutsEnabled).toBe(true);
    expect(result.current.isHydrated()).toBe(true);
    await waitFor(() => expect(mocks.getSettingsPath).toHaveBeenCalledOnce());
    expect(onRestoreInput).not.toHaveBeenCalled();
    expect(onLoadError).not.toHaveBeenCalled();
  });
});