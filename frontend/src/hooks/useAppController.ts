import { useCallback, useMemo, useRef, type RefObject } from "react";

import {
  isTauri,
  type CharacterConversion,
  type BuildCapabilities,
  type EditorFontSize,
  type FontFamily,
  type Rule,
  type RuleSelection,
  type ReplacementPair,
  type Preset,
  type SettingsLoadNotice,
  type ShortcutAction,
  type ShortcutBindings,
  type ThemeMode,
  type UiScale,
  type OutputMode,
  type LayoutMode,
  type UserSettings,
} from "@/lib/tauri";

import { useClearFeedback } from "./useClearFeedback";
import { useClipboardStatus } from "./useClipboardStatus";
import { useFormatter } from "./useFormatter";
import { useInputFormatting } from "./useInputFormatting";
import { useRuleCatalog } from "./useRuleCatalog";
import { useSettingsActions } from "./useSettingsActions";
import { useSettingsDialog } from "./useSettingsDialog";
import { useSettingsLoader } from "./useSettingsLoader";
import { useSettingsPersistence } from "./useSettingsPersistence";
import { useShortcuts } from "./useShortcuts";
import { useThemeAndFont } from "./useThemeAndFont";
import { useWindowControls } from "./useWindowControls";

export interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  triggerRef: RefObject<HTMLButtonElement | null>;
  rules: Rule[];
  enabled: string[];
  enabledSet: Set<string>;
  theme: ThemeMode;
  font: FontFamily;
  editorFontSize: EditorFontSize;
  uiScale: UiScale;
  replacements: ReplacementPair[];
  conversion: CharacterConversion;
  buildCapabilities: BuildCapabilities;
  settingsLoadNotices: SettingsLoadNotice[];
  appVersion: string;
  settingsStatus: "idle" | "saving" | "saved" | "error";
  settingsError: string | null;
  settingsPath: string | null;
  onToggleRule: (key: string) => void;
  onSetAll: (on: boolean) => void;
  onResetDefaults: () => void;
  onThemeChange: (theme: ThemeMode) => void;
  onFollowSystemChange: (follow: boolean) => void;
  onFontChange: (font: FontFamily) => void;
  onResetFont: () => void;
  onEditorFontSizeChange: (size: EditorFontSize) => void;
  onUiScaleChange: (scale: UiScale) => void;
  shortcutsEnabled: boolean;
  shortcutBindings: ShortcutBindings;
  onShortcutsEnabledChange: (enabled: boolean) => void;
  onSaveShortcutBinding: (action: ShortcutAction, binding: string) => void;
  onResetShortcuts: () => void;
  onReplacementsChange: (replacements: ReplacementPair[]) => void;
  onConversionChange: (conversion: CharacterConversion) => void;
  presets: Preset[];
  onApplyPreset: (preset: Preset) => void;
  outputMode: OutputMode;
  layoutMode: LayoutMode;
  onOutputModeChange: (mode: OutputMode) => void;
  onLayoutModeChange: (mode: LayoutMode) => void;
}

export interface UseAppControllerResult {
  isDemoMode: boolean;
  rules: Rule[];
  enabledSet: Set<string>;
  output: string;
  error: string | null;
  isFormatting: boolean;
  lastFormatDuration: number | null;
  input: string;
  onInputChange: (input: string) => void;
  copied: boolean;
  copyOutput: () => void;
  cleared: boolean;
  onClear: () => void;
  settingsDialogProps: SettingsDialogProps;
  onMinimize: () => Promise<void>;
  onToggleMaximize: () => Promise<void>;
  onClose: () => Promise<void>;
  onHeaderMouseDown: (event: React.MouseEvent<HTMLElement>) => void;
}

/**
 * 主界面控制器：编排格式化、设置、输入、快捷键、窗口等子 hook，
 * 计算 SettingsDialog 的派生 props，使 App 仅负责渲染。
 */
export function useAppController(): UseAppControllerResult {
  // ---- 1. 弹窗 ----
  const dialog = useSettingsDialog();

  // ---- 2. 格式化 ----
  const rulesRef = useRef<Rule[]>([]);
  const getRuleSelection = useMemo(
    () => (selected: string[]): RuleSelection => {
      const ruleCount = rulesRef.current.length;
      if (selected.length === 0) return { mode: "none" };
      if (selected.length === ruleCount && ruleCount > 0) return { mode: "all" };
      return { mode: "only", keys: selected };
    },
    [],
  );
  const formatter = useFormatter({ getSelection: getRuleSelection });

  // ---- 3. 剪贴板 ----
  const clipboard = useClipboardStatus({
    getText: () => formatter.output,
    onError: formatter.reportError,
    resetMs: 1200,
  });

  // ---- 4. 设置加载 ----
  const settings = useSettingsLoader({
    onRestoreInput: (restoredInput, restoredEnabled, restoredReplacements, restoredConversion) => {
      setInputRef.current(restoredInput);
      formatter.scheduleFormat(restoredInput, restoredEnabled, undefined, {
        replacements: restoredReplacements,
        conversion: restoredConversion,
      });
    },
    onLoadError: formatter.reportError,
  });
  const effectiveConversion = settings.buildCapabilities.simplifiedTradConversion
    ? settings.conversion
    : "none";

  // ---- 5. 规则目录 ----
  const rules = useRuleCatalog({ loadSettings: settings.loadSettings, onError: formatter.reportError });
  rulesRef.current = rules.rules;

  // ---- 6. 持久化 ----
  const currentSettingsRef = useRef<() => UserSettings>(() => ({
    enabled: [],
    last_input: "",
    theme: "system",
    font: "system",
    editor_font_size: "normal",
    ui_scale: "normal",
    output_mode: "realtime",
    layout_mode: "auto",
    replacements: settings.replacements,
    conversion: effectiveConversion,
    shortcuts: { enabled: false, bindings: {} as ShortcutBindings },
  }));
  const persistence = useSettingsPersistence({
    getSettings: () => currentSettingsRef.current(),
    isHydrated: settings.isHydrated,
    debounceMs: 800,
  });

  // ---- Refs：设置加载回调需要在输入 hook 创建前可用 ----
  const setInputRef = useRef<(input: string) => void>(() => {});

  // ---- 7. 输入 ----
  const input = useInputFormatting({
    enabled: settings.enabled,
    replacements: settings.replacements,
    conversion: effectiveConversion,
    outputMode: settings.outputMode,
    scheduleFormat: formatter.scheduleFormat,
    schedulePersist: persistence.schedulePersist,
  });
  setInputRef.current = input.setInput;

  // ---- 8. 设置动作 ----
  const actions = useSettingsActions({
    rules: rules.rules,
    enabled: settings.enabled,
    enabledSet: new Set(settings.enabled),
    input: input.input,
    setEnabled: settings.setEnabled,
    setTheme: settings.setTheme,
    setFont: settings.setFont,
    setEditorFontSize: settings.setEditorFontSize,
    setUiScale: settings.setUiScale,
    setOutputMode: settings.setOutputMode,
    setLayoutMode: settings.setLayoutMode,
    replacements: settings.replacements,
    buildCapabilities: settings.buildCapabilities,
    setReplacements: settings.setReplacements,
    conversion: effectiveConversion,
    setConversion: settings.setConversion,
    setShortcutsEnabled: settings.setShortcutsEnabled,
    setShortcutBindings: settings.setShortcutBindings,
    shortcutsEnabled: settings.shortcutsEnabled,
    shortcutBindings: settings.shortcutBindings,
    scheduleFormat: formatter.scheduleFormat,
    persistSettings: persistence.persistSettings,
  });

  // ---- 9. 清空反馈 ----
  const clear = useClearFeedback({
    clearInput: () => input.setInput(""),
    clearOutput: formatter.clearOutput,
    cancelFormat: formatter.cancelFormat,
    clearError: formatter.clearError,
    persistEmptyInput: () => persistence.schedulePersist({ enabled: settings.enabled, last_input: "" }),
    durationMs: 250,
  });

  // ---- 10. 主题/字体应用到 DOM ----
  useThemeAndFont({
    theme: settings.theme,
    font: settings.font,
    editorFontSize: settings.editorFontSize,
    uiScale: settings.uiScale,
  });

  // ---- 11. 窗口控制 ----
  const windowControls = useWindowControls({ onError: formatter.reportError });

  // ---- 12. 快捷键 ----
  useShortcuts({
    enabled: settings.shortcutsEnabled,
    bindings: settings.shortcutBindings,
    onFormatNow: () =>
      formatter.scheduleFormat(input.input, settings.enabled, 0, {
        replacements: settings.replacements,
        conversion: effectiveConversion,
      }),
    onCopyOutput: clipboard.copy,
    onOpenSettings: () => dialog.onOpenChange(true),
  });

  // ---- 派生：currentSettings ----
  const currentSettings = useCallback(
    (next: Partial<UserSettings> = {}): UserSettings => ({
      enabled: settings.enabled,
      last_input: input.input,
      theme: settings.theme,
      font: settings.font,
      editor_font_size: settings.editorFontSize,
      ui_scale: settings.uiScale,
      output_mode: settings.outputMode,
      layout_mode: settings.layoutMode,
      shortcuts: { enabled: settings.shortcutsEnabled, bindings: settings.shortcutBindings },
      replacements: settings.replacements,
      conversion: effectiveConversion,
      ...next,
    }),
    [
      settings.enabled,
      settings.theme,
      settings.font,
      settings.editorFontSize,
      settings.uiScale,
      settings.outputMode,
      settings.layoutMode,
      settings.shortcutsEnabled,
      settings.shortcutBindings,
      settings.replacements,
      effectiveConversion,
      input.input,
    ],
  );
  currentSettingsRef.current = currentSettings;

  // ---- 派生：SettingsDialog props ----
  const settingsDialogProps = useMemo<SettingsDialogProps>(
    () => ({
      open: dialog.open,
      onOpenChange: dialog.onOpenChange,
      triggerRef: dialog.triggerRef,
      rules: rules.rules,
      enabled: settings.enabled,
      enabledSet: new Set(settings.enabled),
      theme: settings.theme,
      font: settings.font,
      editorFontSize: settings.editorFontSize,
      uiScale: settings.uiScale,
      replacements: settings.replacements,
      conversion: effectiveConversion,
      buildCapabilities: settings.buildCapabilities,
      settingsLoadNotices: settings.settingsLoadNotices,
      appVersion: settings.appVersion,
      settingsStatus: persistence.settingsStatus,
      settingsError: persistence.settingsError,
      settingsPath: settings.settingsPath,
      onToggleRule: actions.onToggleRule,
      onSetAll: actions.onSetAll,
      onResetDefaults: actions.onResetDefaults,
      onThemeChange: actions.onThemeChange,
      onFollowSystemChange: actions.onFollowSystemChange,
      onFontChange: actions.onFontChange,
      onResetFont: actions.onResetFont,
      onEditorFontSizeChange: actions.onEditorFontSizeChange,
      onUiScaleChange: actions.onUiScaleChange,
      shortcutsEnabled: settings.shortcutsEnabled,
      shortcutBindings: settings.shortcutBindings,
      onShortcutsEnabledChange: actions.onShortcutsEnabledChange,
      onSaveShortcutBinding: actions.onSaveShortcutBinding,
      onResetShortcuts: actions.onResetShortcuts,
      onReplacementsChange: actions.onReplacementsChange,
      onConversionChange: actions.onConversionChange,
      presets: rules.presets,
      onApplyPreset: actions.onApplyPreset,
      outputMode: settings.outputMode,
      layoutMode: settings.layoutMode,
      onOutputModeChange: actions.onOutputModeChange,
      onLayoutModeChange: actions.onLayoutModeChange,
    }),
    [
      dialog.open,
      dialog.onOpenChange,
      dialog.triggerRef,
      rules.rules,
      settings.enabled,
      settings.theme,
      settings.font,
      settings.editorFontSize,
      settings.uiScale,
      settings.settingsLoadNotices,
      settings.appVersion,
      settings.settingsPath,
      settings.shortcutsEnabled,
      settings.shortcutBindings,
      settings.replacements,
      settings.conversion,
      settings.buildCapabilities,
      persistence.settingsStatus,
      persistence.settingsError,
      actions.onToggleRule,
      actions.onSetAll,
      actions.onResetDefaults,
      actions.onThemeChange,
      actions.onFollowSystemChange,
      actions.onFontChange,
      actions.onResetFont,
      actions.onEditorFontSizeChange,
      actions.onUiScaleChange,
      actions.onShortcutsEnabledChange,
      actions.onSaveShortcutBinding,
      actions.onResetShortcuts,
      actions.onReplacementsChange,
      actions.onConversionChange,
      rules.presets,
      actions.onApplyPreset,
      settings.outputMode,
      settings.layoutMode,
      actions.onOutputModeChange,
      actions.onLayoutModeChange,
    ],
  );

  return {
    isDemoMode: !isTauri(),
    rules: rules.rules,
    enabledSet: new Set(settings.enabled),
    output: formatter.output,
    error: formatter.error,
    isFormatting: formatter.isFormatting,
    lastFormatDuration: formatter.lastFormatDuration,
    input: input.input,
    onInputChange: input.onInputChange,
    copied: clipboard.copied,
    copyOutput: clipboard.copy,
    cleared: clear.cleared,
    onClear: clear.clear,
    settingsDialogProps,
    onMinimize: windowControls.onMinimize,
    onToggleMaximize: windowControls.onToggleMaximize,
    onClose: windowControls.onClose,
    onHeaderMouseDown: windowControls.onHeaderMouseDown,
  };
}
