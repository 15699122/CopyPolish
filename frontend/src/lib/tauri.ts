import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

/**
 * Tauri command 薄封装（唯一合法入口）。
 * 与 src-tauri/src/commands.rs 中注册的命令一一对应；
 * 通过受控的 Rust command 访问本地引擎，而不是直接暴露任意后端调用。
 *
 * 浏览器预览回退：当页面运行在普通浏览器（无 Tauri 运行时）时，
 * 用内置的最小 JS 实现兜底，便于脱离 Rust 开发 UI；打包后在 Tauri 内
 * 统一走 invoke 到 Rust 引擎。
 */

export interface Rule {
  key: string;
  section: string;
  name: string;
  disputed: boolean;
  default: boolean;
}

export interface FormatRequest {
  text: string;
  selection: RuleSelection;
}

export type RuleSelection =
  | { mode: "all" }
  | { mode: "defaults" }
  | { mode: "only"; keys: string[] }
  | { mode: "none" };

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 返回当前应用版本；打包应用读取 Tauri manifest，浏览器预览回退到 Vite 注入版本。 */
export async function getAppVersion(): Promise<string> {
  if (!isTauri()) return __APP_VERSION__;
  return getVersion();
}

// 浏览器回退：不维护规则副本。桌面端的规则列表与排版完全由 Rust 注册表
// 驱动；浏览器预览只提供最小化的空格/标点演示效果，用于 UI 开发，
// 不代表桌面端完整引擎行为。
function fallbackFormat(text: string): string {
  const cjk = "[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff]";
  return text
    .replace(new RegExp(`(?<=${cjk})(?=[A-Za-z])`, "g"), " ")
    .replace(new RegExp(`(?<=[A-Za-z])(?=${cjk})`, "g"), " ")
    .replace(new RegExp(`(?<=${cjk})(?=\\d)`, "g"), " ")
    .replace(new RegExp(`(?<=\\d)(?=${cjk})`, "g"), " ")
    .replace(/\s+([，。；：！？、）】》」』])/g, "$1")
    .replace(/([，。；：！？、））】》」』])\s+/g, "$1");
}

export async function formatText(request: FormatRequest): Promise<string> {
  if (!isTauri()) {
    if (request.selection.mode === "none") return request.text;
    return fallbackFormat(request.text);
  }
  return invoke<string>("format_text", { ...request });
}

/** 浏览器预览为演示模式：规则列表由桌面端注册表提供，此处返回空列表。 */
export async function getRules(): Promise<Rule[]> {
  if (!isTauri()) return [];
  return invoke<Rule[]>("get_rules");
}

export async function getEnabledDefaults(): Promise<string[]> {
  if (!isTauri()) return [];
  return invoke<string[]>("get_enabled_defaults");
}


// ---------------------------------------------------------------------------
// 用户设置持久化（由 Rust 端保存在 exe 同目录的 rules.yaml；
// 浏览器预览时回退到 localStorage）。
// ---------------------------------------------------------------------------

export type ThemeMode = "system" | "light" | "dark";
export type FontFamily = "system" | "microsoft-yahei" | "pingfang" | "noto-sans-cjk" | "simsun" | "simhei";

export interface UserSettings {
  enabled: string[];
  last_input: string;
  theme: ThemeMode;
  font: FontFamily;
}

export interface LoadedUserSettings extends UserSettings {
  recovered_from_backup: boolean;
}

const LS_SETTINGS_KEY = "ccw-formatter-settings";

/** 返回设置文件（rules.yaml）的完整路径；浏览器预览返回 null。 */
export async function getSettingsPath(): Promise<string | null> {
  if (!isTauri()) return null;
  return invoke<string>("get_settings_path");
}

export async function getUserSettings(): Promise<LoadedUserSettings | null> {
  if (!isTauri()) {
    try {
      const raw = window.localStorage.getItem(LS_SETTINGS_KEY);
      if (!raw) return null;
      const parsed = JSON.parse(raw) as UserSettings;
      return {
        enabled: parsed.enabled ?? [],
        last_input: parsed.last_input ?? "",
        theme: ensureThemeMode(parsed.theme),
        font: ensureFontFamily(parsed.font),
        recovered_from_backup: false,
      };
    } catch {
      return null;
    }
  }
  const loaded = await invoke<{
    settings: UserSettings;
    recovered_from_backup: boolean;
  } | null>("get_user_settings");
  if (!loaded) return null;
  const settings = loaded.settings;
  return {
    enabled: settings.enabled ?? [],
    last_input: settings.last_input ?? "",
    theme: ensureThemeMode(settings.theme),
    font: ensureFontFamily(settings.font),
    recovered_from_backup: loaded.recovered_from_backup,
  };
}

export async function saveUserSettings(settings: UserSettings): Promise<void> {
  if (!isTauri()) {
    try {
      window.localStorage.setItem(LS_SETTINGS_KEY, JSON.stringify(settings));
    } catch {
      // localStorage 不可用时静默忽略（预览环境）。
    }
    return;
  }
  await invoke("save_user_settings", {
    enabled: settings.enabled,
    lastInput: settings.last_input,
    theme: settings.theme,
    font: settings.font,
  });
}

function ensureThemeMode(value: unknown): ThemeMode {
  if (value === "light" || value === "dark") return value;
  return "system";
}

export function ensureFontFamily(value: unknown): FontFamily {
  if (
    value === "microsoft-yahei" ||
    value === "pingfang" ||
    value === "noto-sans-cjk" ||
    value === "simsun" ||
    value === "simhei"
  ) {
    return value;
  }
  return "system";
}
