import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useSettingsActions } from "./useSettingsActions";

const rules = [
  { key: "rule-a", section: "空格", name: "规则 A", disputed: false, default: true },
  { key: "rule-b", section: "空格", name: "规则 B", disputed: false, default: false },
];

const defaultBindings = {
  format_now: "CtrlOrCmd+Enter",
  copy_output: "CtrlOrCmd+Shift+KeyC",
  open_settings: "CtrlOrCmd+Comma",
};

function createOptions() {
  return {
    rules,
    enabled: ["rule-a"],
    enabledSet: new Set(["rule-a"]),
    input: "原文",
    setEnabled: vi.fn(),
    setTheme: vi.fn(),
    setFont: vi.fn(),
    setEditorFontSize: vi.fn(),
    setUiScale: vi.fn(),
    replacements: [{ from: "A", to: "甲", active: true }],
    setReplacements: vi.fn(),
    conversion: "s2t" as const,
    setConversion: vi.fn(),
    setShortcutsEnabled: vi.fn(),
    setShortcutBindings: vi.fn(),
    shortcutsEnabled: false,
    shortcutBindings: { ...defaultBindings },
    scheduleFormat: vi.fn(),
    persistSettings: vi.fn(),
  };
}

describe("useSettingsActions", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("规则操作同步更新启用集、立即排版并保存 patch", () => {
    const options = createOptions();
    const { result } = renderHook(() => useSettingsActions(options));

    act(() => result.current.onToggleRule("rule-b"));
    expect(options.setEnabled).toHaveBeenCalledWith(["rule-a", "rule-b"]);
    expect(options.scheduleFormat).toHaveBeenCalledWith("原文", ["rule-a", "rule-b"], 0, {
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });
    expect(options.persistSettings).toHaveBeenCalledWith({
      enabled: ["rule-a", "rule-b"],
      last_input: "原文",
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });

    options.setEnabled.mockClear();
    options.scheduleFormat.mockClear();
    options.persistSettings.mockClear();
    act(() => result.current.onSetAll(false));
    expect(options.setEnabled).toHaveBeenCalledWith([]);
    expect(options.scheduleFormat).toHaveBeenCalledWith("原文", [], 0, {
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });
    expect(options.persistSettings).toHaveBeenCalledWith({
      enabled: [],
      last_input: "原文",
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "s2t",
    });
  });

  it("替换列表和转换选择会立即重排并保存", () => {
    const options = createOptions();
    const { result } = renderHook(() => useSettingsActions(options));
    const nextReplacements = [{ from: "A", to: "乙", active: false }];

    act(() => {
      result.current.onReplacementsChange(nextReplacements);
      result.current.onConversionChange("none");
    });

    expect(options.setReplacements).toHaveBeenCalledWith(nextReplacements);
    expect(options.setConversion).toHaveBeenCalledWith("none");
    expect(options.scheduleFormat).toHaveBeenCalledWith("原文", ["rule-a"], 0, {
      replacements: nextReplacements,
      conversion: "s2t",
    });
    expect(options.scheduleFormat).toHaveBeenLastCalledWith("原文", ["rule-a"], 0, {
      replacements: [{ from: "A", to: "甲", active: true }],
      conversion: "none",
    });
  });

  it("显示设置和快捷键操作保存最新 patch", () => {
    const options = createOptions();
    const matchMedia = vi.fn(() => ({ matches: true }));
    vi.stubGlobal("matchMedia", matchMedia);
    const { result } = renderHook(() => useSettingsActions(options));

    act(() => {
      result.current.onThemeChange("dark");
      result.current.onFontChange("pingfang");
      result.current.onEditorFontSizeChange("large");
      result.current.onUiScaleChange("small");
      result.current.onFollowSystemChange(false);
      result.current.onShortcutsEnabledChange(true);
      result.current.onSaveShortcutBinding("format_now", "CtrlOrCmd+KeyF");
      result.current.onResetShortcuts();
    });

    expect(options.setTheme).toHaveBeenCalledWith("dark");
    expect(options.setFont).toHaveBeenCalledWith("pingfang");
    expect(options.setEditorFontSize).toHaveBeenCalledWith("large");
    expect(options.setUiScale).toHaveBeenCalledWith("small");
    expect(options.setShortcutsEnabled).toHaveBeenCalledWith(true);
    expect(options.setShortcutBindings).toHaveBeenCalledWith({
      ...defaultBindings,
      format_now: "CtrlOrCmd+KeyF",
    });
    expect(options.persistSettings).toHaveBeenCalledWith({ theme: "dark" });
    expect(options.persistSettings).toHaveBeenCalledWith({ font: "pingfang" });
    expect(options.persistSettings).toHaveBeenCalledWith({ editor_font_size: "large" });
    expect(options.persistSettings).toHaveBeenCalledWith({ ui_scale: "small" });
    expect(options.persistSettings).toHaveBeenCalledWith({ theme: "dark" });
    expect(options.persistSettings).toHaveBeenCalledWith({
      shortcuts: { enabled: true, bindings: { ...defaultBindings } },
    });
    expect(options.persistSettings).toHaveBeenCalledWith({
      shortcuts: {
        enabled: false,
        bindings: { ...defaultBindings, format_now: "CtrlOrCmd+KeyF" },
      },
    });
    expect(options.persistSettings).toHaveBeenLastCalledWith({
      shortcuts: { enabled: true, bindings: defaultBindings },
    });
  });
});