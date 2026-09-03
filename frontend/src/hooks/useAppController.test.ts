import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => {
  const rules = [
    { key: "rule-a", section: "空格", name: "规则 A", disputed: false, default: true },
    { key: "rule-b", section: "空格", name: "规则 B", disputed: false, default: false },
  ];

  const formatter = {
    output: "格式化结果",
    error: null,
    isFormatting: false,
    lastFormatDuration: 12,
    scheduleFormat: vi.fn(),
    cancelFormat: vi.fn(),
    clearOutput: vi.fn(),
    clearError: vi.fn(),
    reportError: vi.fn(),
  };
  const persistence = {
    settingsStatus: "idle" as const,
    settingsError: null,
    persistSettings: vi.fn(),
    schedulePersist: vi.fn(),
  };
  const input = {
    input: "当前输入",
    setInput: vi.fn(),
    onInputChange: vi.fn(),
  };
  const clear = {
    cleared: false,
    clear: vi.fn(),
  };
  const actions = {
    onToggleRule: vi.fn(),
    onSetAll: vi.fn(),
    onResetDefaults: vi.fn(),
    onThemeChange: vi.fn(),
    onFollowSystemChange: vi.fn(),
    onFontChange: vi.fn(),
    onResetFont: vi.fn(),
    onEditorFontSizeChange: vi.fn(),
    onUiScaleChange: vi.fn(),
    onShortcutsEnabledChange: vi.fn(),
    onSaveShortcutBinding: vi.fn(),
    onResetShortcuts: vi.fn(),
    onReplacementsChange: vi.fn(),
    onConversionChange: vi.fn(),
    onApplyPreset: vi.fn(),
  };

  return {
    rules,
    formatter,
    persistence,
    input,
    clear,
    actions,
    dialog: {
      open: false,
      triggerRef: { current: null },
      onOpenChange: vi.fn(),
    },
    settings: {
      buildCapabilities: { simplifiedTradConversion: true },
      enabled: ["rule-a"],
      setEnabled: vi.fn(),
      theme: "system" as const,
      setTheme: vi.fn(),
      font: "system" as const,
      setFont: vi.fn(),
      editorFontSize: "normal" as const,
      setEditorFontSize: vi.fn(),
      uiScale: "normal" as const,
      setUiScale: vi.fn(),
      outputMode: "realtime" as const,
      setOutputMode: vi.fn(),
      layoutMode: "auto" as const,
      setLayoutMode: vi.fn(),
      shortcutsEnabled: true,
      setShortcutsEnabled: vi.fn(),
      shortcutBindings: {
        format_now: "CtrlOrCmd+Enter",
        copy_output: "CtrlOrCmd+Shift+KeyC",
        open_settings: "CtrlOrCmd+Comma",
      },
      setShortcutBindings: vi.fn(),
      replacements: [] as { from: string; to: string; active: boolean }[],
      setReplacements: vi.fn(),
      conversion: "none" as "none" | "t2s" | "s2t",
      setConversion: vi.fn(),
      settingsLoadNotices: [],
      settingsPath: null,
      appVersion: "0.5.0-test",
      isHydrated: vi.fn(() => true),
      loadSettings: vi.fn(),
      presets: [],
    },
    catalogOptions: undefined as unknown,
    loaderOptions: undefined as unknown,
    inputOptions: undefined as unknown,
    clearOptions: undefined as unknown,
    isTauri: vi.fn(() => false),
    shortcutOptions: undefined as unknown,
  };
});

vi.mock("@/lib/tauri", () => ({
  isTauri: mocks.isTauri,
}));

vi.mock("./useClearFeedback", () => ({
  useClearFeedback: (options: unknown) => {
    mocks.clearOptions = options;
    return mocks.clear;
  },
}));

vi.mock("./useClipboardStatus", () => ({
  useClipboardStatus: () => ({ copied: false, copy: vi.fn() }),
}));

vi.mock("./useFormatter", () => ({
  useFormatter: () => mocks.formatter,
}));

vi.mock("./useInputFormatting", () => ({
  useInputFormatting: (options: unknown) => {
    mocks.inputOptions = options;
    return mocks.input;
  },
}));

vi.mock("./useRuleCatalog", () => ({
  useRuleCatalog: (options: unknown) => {
    mocks.catalogOptions = options;
    return { rules: mocks.rules, presets: [] };
  },
}));

vi.mock("./useSettingsActions", () => ({
  useSettingsActions: () => mocks.actions,
}));

vi.mock("./useSettingsDialog", () => ({
  useSettingsDialog: () => mocks.dialog,
}));

vi.mock("./useSettingsLoader", () => ({
  useSettingsLoader: (options: unknown) => {
    mocks.loaderOptions = options;
    return mocks.settings;
  },
}));

vi.mock("./useSettingsPersistence", () => ({
  useSettingsPersistence: () => mocks.persistence,
}));

vi.mock("./useShortcuts", () => ({
  useShortcuts: (options: unknown) => {
    mocks.shortcutOptions = options;
  },
}));

vi.mock("./useThemeAndFont", () => ({
  useThemeAndFont: vi.fn(),
}));

vi.mock("./useWindowControls", () => ({
  useWindowControls: () => ({
    onMinimize: vi.fn(async () => {}),
    onToggleMaximize: vi.fn(async () => {}),
    onClose: vi.fn(async () => {}),
    onHeaderMouseDown: vi.fn(),
  }),
}));

import { useAppController } from "./useAppController";

describe("useAppController", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.catalogOptions = undefined;
    mocks.loaderOptions = undefined;
    mocks.inputOptions = undefined;
    mocks.clearOptions = undefined;
    mocks.shortcutOptions = undefined;
    mocks.isTauri.mockReturnValue(false);
  });

  it("连接设置恢复、输入格式化和设置保存调度", () => {
    const { result } = renderHook(() => useAppController());
    const catalogOptions = mocks.catalogOptions as {
      loadSettings: (rules: typeof mocks.rules, defaults: string[]) => Promise<void>;
    };
    const inputOptions = mocks.inputOptions as {
      scheduleFormat: typeof mocks.formatter.scheduleFormat;
      schedulePersist: typeof mocks.persistence.schedulePersist;
    };

    expect(catalogOptions.loadSettings).toBe(mocks.settings.loadSettings);
    expect(inputOptions.scheduleFormat).toBe(mocks.formatter.scheduleFormat);
    expect(inputOptions.schedulePersist).toBe(mocks.persistence.schedulePersist);

    expect(result.current.settingsDialogProps.enabled).toEqual(["rule-a"]);
    expect(result.current.settingsDialogProps.rules).toBe(mocks.rules);
  });

  it("把设置恢复回调接到输入状态和立即格式化", () => {
    renderHook(() => useAppController());

    const loader = mocks.loaderOptions as {
      onRestoreInput: (
        input: string,
        enabled: string[],
        replacements: { from: string; to: string; active: boolean }[],
        conversion: "none" | "t2s" | "s2t",
      ) => void;
    };
    loader.onRestoreInput("恢复的内容", ["rule-a", "rule-b"], [], "none");

    expect(mocks.input.setInput).toHaveBeenCalledWith("恢复的内容");
    expect(mocks.formatter.scheduleFormat).toHaveBeenCalledWith(
      "恢复的内容",
      ["rule-a", "rule-b"],
      undefined,
      { replacements: [], conversion: "none" },
    );
  });

  it("快捷键立即排版携带当前替换和转换设置", () => {
    mocks.settings.replacements = [{ from: "A", to: "甲", active: true }];
    mocks.settings.conversion = "s2t";
    renderHook(() => useAppController());

    const shortcuts = mocks.shortcutOptions as {
      onFormatNow: () => void;
    };
    shortcuts.onFormatNow();

    expect(mocks.formatter.scheduleFormat).toHaveBeenCalledWith(
      "当前输入",
      ["rule-a"],
      0,
      { replacements: [{ from: "A", to: "甲", active: true }], conversion: "s2t" },
    );
  });

  it("清空时清理输出并防抖保存空输入", () => {
    const { result } = renderHook(() => useAppController());
    const clearOptions = mocks.clearOptions as {
      clearInput: () => void;
      clearOutput: () => void;
      cancelFormat: () => void;
      clearError: () => void;
      persistEmptyInput: () => void;
    };

    clearOptions.clearInput();
    clearOptions.clearOutput();
    clearOptions.cancelFormat();
    clearOptions.clearError();
    clearOptions.persistEmptyInput();

    expect(mocks.input.setInput).toHaveBeenCalledWith("");
    expect(mocks.formatter.clearOutput).toHaveBeenCalledOnce();
    expect(mocks.formatter.cancelFormat).toHaveBeenCalledOnce();
    expect(mocks.formatter.clearError).toHaveBeenCalledOnce();
    expect(mocks.persistence.schedulePersist).toHaveBeenCalledWith({
      enabled: ["rule-a"],
      last_input: "",
    });
    expect(result.current.onClear).toBe(mocks.clear.clear);
  });

  it("浏览器环境标记为演示模式", () => {
    const { result } = renderHook(() => useAppController());

    expect(result.current.isDemoMode).toBe(true);
    expect(mocks.isTauri).toHaveBeenCalled();
  });
});