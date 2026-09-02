import { useCallback, useEffect, useRef, useState } from "react";

import {
  DEFAULT_SHORTCUT_SETTINGS,
  getAppVersion,
  getSettingsPath,
  getUserSettings,
  type EditorFontSize,
  type FontFamily,
  type LoadedUserSettings,
  type CharacterConversion,
  type ReplacementPair,
  type Rule,
  type SettingsLoadNotice,
  type ShortcutBindings,
  type ThemeMode,
  type UiScale,
} from "@/lib/tauri";

export interface UseSettingsLoaderOptions {
  onRestoreInput: (
    input: string,
    enabled: string[],
    replacements: ReplacementPair[],
    conversion: CharacterConversion,
  ) => void;
  onLoadError: (cause: unknown) => void;
}

export interface UseSettingsLoaderResult {
  enabled: string[];
  setEnabled: (enabled: string[]) => void;
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;
  font: FontFamily;
  setFont: (font: FontFamily) => void;
  editorFontSize: EditorFontSize;
  setEditorFontSize: (size: EditorFontSize) => void;
  uiScale: UiScale;
  setUiScale: (scale: UiScale) => void;
  shortcutsEnabled: boolean;
  setShortcutsEnabled: (enabled: boolean) => void;
  shortcutBindings: ShortcutBindings;
  setShortcutBindings: (bindings: ShortcutBindings) => void;
  replacements: ReplacementPair[];
  setReplacements: (replacements: ReplacementPair[]) => void;
  conversion: CharacterConversion;
  setConversion: (conversion: CharacterConversion) => void;
  settingsLoadNotices: SettingsLoadNotice[];
  settingsPath: string | null;
  appVersion: string;
  isHydrated: () => boolean;
  loadSettings: (rules: Rule[], defaults: string[]) => Promise<void>;
}

/** 设置初始化、恢复提醒及设置元数据加载；保存生命周期由 useSettingsPersistence 负责。 */
export function useSettingsLoader({
  onRestoreInput,
  onLoadError,
}: UseSettingsLoaderOptions): UseSettingsLoaderResult {
  const [enabled, setEnabled] = useState<string[]>([]);
  const [theme, setTheme] = useState<ThemeMode>("system");
  const [font, setFont] = useState<FontFamily>("system");
  const [editorFontSize, setEditorFontSize] = useState<EditorFontSize>("normal");
  const [uiScale, setUiScale] = useState<UiScale>("normal");
  const [shortcutsEnabled, setShortcutsEnabled] = useState(DEFAULT_SHORTCUT_SETTINGS.enabled);
  const [shortcutBindings, setShortcutBindings] = useState<ShortcutBindings>(
    DEFAULT_SHORTCUT_SETTINGS.bindings,
  );
  const [replacements, setReplacements] = useState<ReplacementPair[]>([]);
  const [conversion, setConversion] = useState<CharacterConversion>("none");
  const [settingsLoadNotices, setSettingsLoadNotices] = useState<SettingsLoadNotice[]>([]);
  const [settingsPath, setSettingsPath] = useState<string | null>(null);
  const [appVersion, setAppVersion] = useState(__APP_VERSION__);

  const hydratedRef = useRef(false);
  const loadingRef = useRef(false);
  const mountedRef = useRef(true);
  const callbacksRef = useRef({ onRestoreInput, onLoadError });
  callbacksRef.current = { onRestoreInput, onLoadError };

  useEffect(() => {
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const loadSettings = useCallback(async (rules: Rule[], defaults: string[]) => {
    if (loadingRef.current || hydratedRef.current) return;
    loadingRef.current = true;

    try {
      let saved: LoadedUserSettings | null = null;
      try {
        saved = await getUserSettings();
      } catch {
        saved = null;
      }
      if (!mountedRef.current) return;

      try {
        const path = await getSettingsPath();
        if (mountedRef.current && path) setSettingsPath(path);
      } catch {
        // 路径获取失败不影响主流程；保存错误会在保存时展示。
      }
      try {
        const version = await getAppVersion();
        if (mountedRef.current) setAppVersion(version);
      } catch {
        // 读取版本失败时保留构建时注入的浏览器回退版本。
      }
      if (!mountedRef.current) return;

      if (saved && Array.isArray(saved.enabled)) {
        const restoredEnabled = saved.enabled.filter((key) =>
          rules.some((rule) => rule.key === key),
        );
        setEnabled(restoredEnabled);
        setTheme(saved.theme);
        setFont(saved.font);
        setEditorFontSize(saved.editor_font_size ?? "normal");
        setUiScale(saved.ui_scale ?? "normal");
        if (saved.shortcuts) {
          setShortcutsEnabled(saved.shortcuts.enabled);
          setShortcutBindings(saved.shortcuts.bindings);
        }
        setReplacements(saved.replacements ?? []);
        setConversion(saved.conversion ?? "none");
        setSettingsLoadNotices(saved.notices ?? []);
        if (saved.last_input) {
          callbacksRef.current.onRestoreInput(
            saved.last_input,
            restoredEnabled,
            saved.replacements ?? [],
            saved.conversion ?? "none",
          );
        }
      } else {
        setEnabled(defaults.filter((key) => rules.some((rule) => rule.key === key)));
      }
      hydratedRef.current = true;
    } catch (cause) {
      if (mountedRef.current) {
        hydratedRef.current = true;
        callbacksRef.current.onLoadError(cause);
      }
    }
  }, []);

  return {
    enabled,
    setEnabled,
    theme,
    setTheme,
    font,
    setFont,
    editorFontSize,
    setEditorFontSize,
    uiScale,
    setUiScale,
    shortcutsEnabled,
    setShortcutsEnabled,
    shortcutBindings,
    setShortcutBindings,
    replacements,
    setReplacements,
    conversion,
    setConversion,
    settingsLoadNotices,
    settingsPath,
    appVersion,
    isHydrated: () => hydratedRef.current,
    loadSettings,
  };
}