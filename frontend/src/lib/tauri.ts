import { invoke } from "@tauri-apps/api/core";

/**
 * Tauri command 薄封装（唯一合法入口）。
 * 与 src-tauri/src/commands.rs 中注册的命令一一对应；
 * 通过受控的 Rust command 访问 Python，而不是直接暴露任意 Python 调用。
 *
 * 浏览器预览回退：当页面运行在普通浏览器（无 Tauri 运行时）时，
 * 用内置的最小 JS 实现兜底，便于脱离 Rust 开发 UI；打包后在 Tauri 内
 * 统一走 invoke 到 Python 引擎。
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
  enabled: string[];
}

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

const FALLBACK_RULES: Rule[] = [
  { key: "中英文之间需要增加空格", section: "空格", name: "中英文之间需要增加空格", disputed: false, default: true },
  { key: "中文与数字之间需要增加空格", section: "空格", name: "中文与数字之间需要增加空格", disputed: false, default: true },
  { key: "数字与单位之间需要增加空格", section: "空格", name: "数字与单位之间需要增加空格", disputed: false, default: true },
  { key: "全角标点与其他字符之间不加空格", section: "空格", name: "全角标点与其他字符之间不加空格", disputed: false, default: true },
  { key: "不重复使用标点符号", section: "标点符号", name: "不重复使用标点符号", disputed: false, default: true },
  { key: "使用全角中文标点", section: "全角和半角", name: "使用全角中文标点", disputed: false, default: true },
  { key: "数字使用半角字符", section: "全角和半角", name: "数字使用半角字符", disputed: false, default: true },
  { key: "遇到完整的英文整句、特殊名词，其内容使用半角标点", section: "全角和半角", name: "遇到完整的英文整句、特殊名词，其内容使用半角标点", disputed: false, default: true },
  { key: "专有名词使用正确的大小写", section: "名词", name: "专有名词使用正确的大小写", disputed: false, default: false },
  { key: "不要使用不地道的缩写", section: "名词", name: "不要使用不地道的缩写", disputed: false, default: false },
  { key: "链接之间增加空格", section: "争议", name: "链接之间增加空格", disputed: true, default: false },
  { key: "简体中文使用直角引号", section: "争议", name: "简体中文使用直角引号", disputed: true, default: false },
];

// 浏览器回退实现（仅中英文/中文与数字空格 + 全角标点去空格），用于 UI 预览，非真实引擎。
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
  if (!isTauri()) return fallbackFormat(request.text);
  return invoke<string>("format_text", { ...request });
}

export async function getRules(): Promise<Rule[]> {
  if (!isTauri()) return FALLBACK_RULES;
  return invoke<Rule[]>("get_rules");
}

export async function getEnabledDefaults(): Promise<string[]> {
  if (!isTauri()) return FALLBACK_RULES.filter((r) => r.default).map((r) => r.key);
  return invoke<string[]>("get_enabled_defaults");
}

// ---------------------------------------------------------------------------
// 用户设置持久化（由 Rust 端保存在 exe 同目录的 rules.yaml；
// 浏览器预览时回退到 localStorage）。
// ---------------------------------------------------------------------------

export type ThemeMode = "system" | "light" | "dark";

export interface UserSettings {
  enabled: string[];
  last_input: string;
  theme: ThemeMode;
}

const LS_SETTINGS_KEY = "ccw-formatter-settings";

/** 返回设置文件（rules.yaml）的完整路径；浏览器预览返回 null。 */
export async function getSettingsPath(): Promise<string | null> {
  if (!isTauri()) return null;
  return invoke<string>("get_settings_path");
}

export async function getUserSettings(): Promise<UserSettings | null> {
  if (!isTauri()) {
    try {
      const raw = window.localStorage.getItem(LS_SETTINGS_KEY);
      if (!raw) return null;
      const parsed = JSON.parse(raw) as UserSettings;
      return {
        enabled: parsed.enabled ?? [],
        last_input: parsed.last_input ?? "",
        theme: ensureThemeMode(parsed.theme),
      };
    } catch {
      return null;
    }
  }
  const settings = await invoke<UserSettings | null>("get_user_settings");
  if (!settings) return null;
  return {
    enabled: settings.enabled ?? [],
    last_input: settings.last_input ?? "",
    theme: ensureThemeMode(settings.theme),
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
  });
}

function ensureThemeMode(value: unknown): ThemeMode {
  if (value === "light" || value === "dark") return value;
  return "system";
}
