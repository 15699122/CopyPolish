import { useCallback } from "react";

import {
  DEFAULT_SHORTCUT_SETTINGS,
  type EditorFontSize,
  type FontFamily,
  type CharacterConversion,
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
  replacements?: ReplacementPair[];
  setReplacements?: (replacements: ReplacementPair[]) => void;
  conversion?: CharacterConversion;
  setConversion?: (conversion: CharacterConversion) => void;
  setShortcutsEnabled: (enabled: boolean) => void;
  setShortcutBindings: (bindings: ShortcutBindings) => void;
  shortcutsEnabled: boolean;
  shortcutBindings: ShortcutBindings;
  scheduleFormat: (input: string, enabled: string[], delayOverride?: number, options?: {
    replacements?: ReplacementPair[];
    conversion?: CharacterConversion;
  }) => void;
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
  onShortcutsEnabledChange: (enabled: boolean) => void;
  onSaveShortcutBinding: (action: ShortcutAction, binding: string) => void;
  onResetShortcuts: () => void;
  onReplacementsChange: (replacements: ReplacementPair[]) => void;
  onConversionChange: (conversion: CharacterConversion) => void;
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
  setShortcutsEnabled,
  setShortcutBindings,
  shortcutsEnabled,
  shortcutBindings,
  replacements,
  conversion,
  setReplacements,
  setConversion,
  scheduleFormat,
  persistSettings,
}: UseSettingsActionsOptions): UseSettingsActionsResult {
  const activeReplacements = replacements ?? [];
  const activeConversion = conversion ?? "none";
  const onToggleRule = useCallback(
    (key: string) => {
      const next = enabledSet.has(key)
        ? enabled.filter((current) => current !== key)
        : [...enabled, key];
      setEnabled(next);
      scheduleFormat(input, next, 0, { replacements: activeReplacements, conversion: activeConversion });
      persistSettings({ enabled: next, last_input: input, replacements: activeReplacements, conversion: activeConversion });
    },
    [activeConversion, activeReplacements, enabled, enabledSet, input, persistSettings, scheduleFormat, setEnabled],
  );

  const onSetAll = useCallback(
    (on: boolean) => {
      const next = on ? rules.map((rule) => rule.key) : [];
      setEnabled(next);
      scheduleFormat(input, next, 0, { replacements: activeReplacements, conversion: activeConversion });
      persistSettings({ enabled: next, last_input: input, replacements: activeReplacements, conversion: activeConversion });
    },
    [activeConversion, activeReplacements, input, persistSettings, rules, scheduleFormat, setEnabled],
  );

  const onResetDefaults = useCallback(() => {
    const next = rules.filter((rule) => rule.default).map((rule) => rule.key);
    setEnabled(next);
    scheduleFormat(input, next, 0, { replacements: activeReplacements, conversion: activeConversion });
    persistSettings({ enabled: next, last_input: input, replacements: activeReplacements, conversion: activeConversion });
  }, [activeConversion, activeReplacements, input, persistSettings, rules, scheduleFormat, setEnabled]);

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
      scheduleFormat(input, enabled, 0, { replacements: nextReplacements, conversion: activeConversion });
      persistSettings({ replacements: nextReplacements, conversion: activeConversion, last_input: input });
    },
    [activeConversion, enabled, input, persistSettings, scheduleFormat, setReplacements],
  );

  const onConversionChange = useCallback(
    (nextConversion: CharacterConversion) => {
      setConversion?.(nextConversion);
      scheduleFormat(input, enabled, 0, { replacements: activeReplacements, conversion: nextConversion });
      persistSettings({ conversion: nextConversion, replacements: activeReplacements, last_input: input });
    },
    [activeReplacements, enabled, input, persistSettings, scheduleFormat, setConversion],
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
    onShortcutsEnabledChange,
    onSaveShortcutBinding,
    onResetShortcuts,
    onReplacementsChange,
    onConversionChange,
  };
}