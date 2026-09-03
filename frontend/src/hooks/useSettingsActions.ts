import { useCallback } from "react";

import {
  DEFAULT_SHORTCUT_SETTINGS,
  type BuildCapabilities,
  type CharacterConversion,
  type EditorFontSize,
  type FontFamily,
  type LayoutMode,
  type OutputMode,
  type Preset,
  type ReplacementPair,
  type Rule,
  type ShortcutAction,
  type ShortcutBindings,
  type ThemeMode,
  type UiScale,
  type UserSettings,
} from "@/lib/tauri";

export interface UseSettingsActionsOptions {
  rules: Rule[];
  enabled: string[];
  enabledSet: Set<string>;
  input: string;
  setEnabled: (enabled: string[]) => void;
  setTheme: (theme: ThemeMode) => void;
  setFont: (font: FontFamily) => void;
  setEditorFontSize: (size: EditorFontSize) => void;
  setUiScale: (scale: UiScale) => void;
  outputMode?: OutputMode;
  setOutputMode?: (mode: OutputMode) => void;
  setLayoutMode?: (mode: LayoutMode) => void;
  replacements?: ReplacementPair[];
  buildCapabilities?: BuildCapabilities;
  setReplacements?: (replacements: ReplacementPair[]) => void;
  conversion?: CharacterConversion;
  setConversion?: (conversion: CharacterConversion) => void;
  setShortcutsEnabled: (enabled: boolean) => void;
  setShortcutBindings: (bindings: ShortcutBindings) => void;
  shortcutsEnabled: boolean;
  shortcutBindings: ShortcutBindings;
  scheduleFormat: (
    input: string,
    enabled: string[],
    delayOverride?: number,
    options?: { replacements?: ReplacementPair[]; conversion?: CharacterConversion },
  ) => void;
  persistSettings: (patch?: Partial<UserSettings>) => void;
}

export interface UseSettingsActionsResult {
  onToggleRule: (key: string) => void;
  onSetAll: (on: boolean) => void;
  onResetDefaults: () => void;
  onThemeChange: (theme: ThemeMode) => void;
  onFollowSystemChange: (follow: boolean) => void;
  onFontChange: (font: FontFamily) => void;
  onResetFont: () => void;
  onEditorFontSizeChange: (size: EditorFontSize) => void;
  onUiScaleChange: (scale: UiScale) => void;
  onOutputModeChange: (mode: OutputMode) => void;
  onLayoutModeChange: (mode: LayoutMode) => void;
  onShortcutsEnabledChange: (enabled: boolean) => void;
  onSaveShortcutBinding: (action: ShortcutAction, binding: string) => void;
  onResetShortcuts: () => void;
  onReplacementsChange: (replacements: ReplacementPair[]) => void;
  onConversionChange: (conversion: CharacterConversion) => void;
  onApplyPreset: (preset: Preset) => void;
}

/** 设置窗口动作：同步更新本地状态、触发重排并提交对应设置 patch。 */
export function useSettingsActions({
  rules,
  enabled,
  enabledSet,
  input,
  setEnabled,
  setTheme,
  setFont,
  setEditorFontSize,
  setUiScale,
  outputMode = "realtime",
  setOutputMode,
  setLayoutMode,
  replacements,
  buildCapabilities,
  setReplacements,
  conversion,
  setConversion,
  setShortcutsEnabled,
  setShortcutBindings,
  shortcutsEnabled,
  shortcutBindings,
  scheduleFormat,
  persistSettings,
}: UseSettingsActionsOptions): UseSettingsActionsResult {
  const activeReplacements = replacements ?? [];
  const activeConversion = conversion ?? "none";
  const conversionAvailable = buildCapabilities?.simplifiedTradConversion ?? false;
  const effectiveConversion = conversionAvailable ? activeConversion : "none";

  const scheduleIfRealtime = useCallback(
    (
      nextInput: string,
      nextEnabled: string[],
      options: { replacements?: ReplacementPair[]; conversion?: CharacterConversion },
    ) => {
      if (outputMode === "realtime") scheduleFormat(nextInput, nextEnabled, 0, options);
    },
    [outputMode, scheduleFormat],
  );

  const onToggleRule = useCallback(
    (key: string) => {
      const next = enabledSet.has(key)
        ? enabled.filter((current) => current !== key)
        : [...enabled, key];
      setEnabled(next);
      scheduleIfRealtime(input, next, {
        replacements: activeReplacements,
        conversion: effectiveConversion,
      });
      persistSettings({
        enabled: next,
        last_input: input,
        replacements: activeReplacements,
        conversion: effectiveConversion,
      });
    },
    [
      activeReplacements,
      effectiveConversion,
      enabled,
      enabledSet,
      input,
      persistSettings,
      scheduleIfRealtime,
      setEnabled,
    ],
  );

  const onSetAll = useCallback(
    (on: boolean) => {
      const next = on ? rules.map((rule) => rule.key) : [];
      setEnabled(next);
      scheduleIfRealtime(input, next, {
        replacements: activeReplacements,
        conversion: effectiveConversion,
      });
      persistSettings({
        enabled: next,
        last_input: input,
        replacements: activeReplacements,
        conversion: effectiveConversion,
      });
    },
    [
      activeReplacements,
      effectiveConversion,
      input,
      persistSettings,
      rules,
      scheduleIfRealtime,
      setEnabled,
    ],
  );

  const onResetDefaults = useCallback(() => {
    const next = rules.filter((rule) => rule.default).map((rule) => rule.key);
    setEnabled(next);
    scheduleIfRealtime(input, next, {
      replacements: activeReplacements,
      conversion: effectiveConversion,
    });
    persistSettings({
      enabled: next,
      last_input: input,
      replacements: activeReplacements,
      conversion: effectiveConversion,
    });
  }, [
    activeReplacements,
    effectiveConversion,
    input,
    persistSettings,
    rules,
    scheduleIfRealtime,
    setEnabled,
  ]);

  const onThemeChange = useCallback(
    (nextTheme: ThemeMode) => {
      setTheme(nextTheme);
      persistSettings({ theme: nextTheme });
    },
    [persistSettings, setTheme],
  );

  const onFollowSystemChange = useCallback(
    (follow: boolean) => {
      if (follow) {
        onThemeChange("system");
        return;
      }
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      onThemeChange(prefersDark ? "dark" : "light");
    },
    [onThemeChange],
  );

  const onFontChange = useCallback(
    (nextFont: FontFamily) => {
      setFont(nextFont);
      persistSettings({ font: nextFont });
    },
    [persistSettings, setFont],
  );

  const onResetFont = useCallback(() => onFontChange("system"), [onFontChange]);

  const onEditorFontSizeChange = useCallback(
    (nextSize: EditorFontSize) => {
      setEditorFontSize(nextSize);
      persistSettings({ editor_font_size: nextSize });
    },
    [persistSettings, setEditorFontSize],
  );

  const onUiScaleChange = useCallback(
    (nextScale: UiScale) => {
      setUiScale(nextScale);
      persistSettings({ ui_scale: nextScale });
    },
    [persistSettings, setUiScale],
  );

  const onOutputModeChange = useCallback(
    (nextMode: OutputMode) => {
      setOutputMode?.(nextMode);
      persistSettings({ output_mode: nextMode });
      if (nextMode === "realtime") {
        scheduleFormat(input, enabled, 0, {
          replacements: activeReplacements,
          conversion: effectiveConversion,
        });
      }
    },
    [
      activeReplacements,
      effectiveConversion,
      enabled,
      input,
      persistSettings,
      scheduleFormat,
      setOutputMode,
    ],
  );

  const onLayoutModeChange = useCallback(
    (nextMode: LayoutMode) => {
      setLayoutMode?.(nextMode);
      persistSettings({ layout_mode: nextMode });
    },
    [persistSettings, setLayoutMode],
  );

  const onShortcutsEnabledChange = useCallback(
    (nextEnabled: boolean) => {
      setShortcutsEnabled(nextEnabled);
      persistSettings({ shortcuts: { enabled: nextEnabled, bindings: shortcutBindings } });
    },
    [persistSettings, setShortcutsEnabled, shortcutBindings],
  );

  const onSaveShortcutBinding = useCallback(
    (action: ShortcutAction, binding: string) => {
      const nextBindings = { ...shortcutBindings, [action]: binding };
      setShortcutBindings(nextBindings);
      persistSettings({ shortcuts: { enabled: shortcutsEnabled, bindings: nextBindings } });
    },
    [persistSettings, setShortcutBindings, shortcutBindings, shortcutsEnabled],
  );

  const onResetShortcuts = useCallback(() => {
    const next = {
      enabled: DEFAULT_SHORTCUT_SETTINGS.enabled,
      bindings: { ...DEFAULT_SHORTCUT_SETTINGS.bindings },
    } satisfies UserSettings["shortcuts"];
    setShortcutsEnabled(next.enabled);
    setShortcutBindings(next.bindings);
    persistSettings({ shortcuts: next });
  }, [persistSettings, setShortcutBindings, setShortcutsEnabled]);

  const onReplacementsChange = useCallback(
    (nextReplacements: ReplacementPair[]) => {
      setReplacements?.(nextReplacements);
      scheduleIfRealtime(input, enabled, {
        replacements: nextReplacements,
        conversion: effectiveConversion,
      });
      persistSettings({
        replacements: nextReplacements,
        conversion: effectiveConversion,
        last_input: input,
      });
    },
    [effectiveConversion, enabled, input, persistSettings, scheduleIfRealtime, setReplacements],
  );

  const onConversionChange = useCallback(
    (nextConversion: CharacterConversion) => {
      const effectiveNextConversion = conversionAvailable ? nextConversion : "none";
      setConversion?.(effectiveNextConversion);
      scheduleIfRealtime(input, enabled, {
        replacements: activeReplacements,
        conversion: effectiveNextConversion,
      });
      persistSettings({
        conversion: effectiveNextConversion,
        replacements: activeReplacements,
        last_input: input,
      });
    },
    [
      activeReplacements,
      conversionAvailable,
      enabled,
      input,
      persistSettings,
      scheduleIfRealtime,
      setConversion,
    ],
  );

  const onApplyPreset = useCallback(
    (preset: Preset) => {
      const nextEnabled = preset.selection.mode === "all"
        ? rules.map((rule) => rule.key)
        : preset.selection.mode === "defaults"
          ? rules.filter((rule) => rule.default).map((rule) => rule.key)
          : preset.selection.mode === "only"
            ? preset.selection.keys.filter((key) => rules.some((rule) => rule.key === key))
            : [];
      const nextConversion = conversionAvailable ? preset.conversion : "none";
      setEnabled(nextEnabled);
      setReplacements?.(preset.replacements);
      setConversion?.(nextConversion);
      scheduleIfRealtime(input, nextEnabled, {
        replacements: preset.replacements,
        conversion: nextConversion,
      });
      persistSettings({
        enabled: nextEnabled,
        last_input: input,
        replacements: preset.replacements,
        conversion: nextConversion,
      });
    },
    [
      conversionAvailable,
      input,
      persistSettings,
      rules,
      scheduleIfRealtime,
      setConversion,
      setEnabled,
      setReplacements,
    ],
  );

  return {
    onToggleRule,
    onSetAll,
    onResetDefaults,
    onThemeChange,
    onFollowSystemChange,
    onFontChange,
    onResetFont,
    onEditorFontSizeChange,
    onUiScaleChange,
    onOutputModeChange,
    onLayoutModeChange,
    onShortcutsEnabledChange,
    onSaveShortcutBinding,
    onResetShortcuts,
    onReplacementsChange,
    onConversionChange,
    onApplyPreset,
  };
}