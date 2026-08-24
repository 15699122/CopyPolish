/**
 * 快捷键领域模型：动作、默认绑定、序列化、事件匹配与校验。
 * 组合键使用语义修饰键 `CtrlOrCmd` + `KeyboardEvent.code` 序列化存储，
 * 运行时在 Windows/Linux 上映射为 Ctrl、macOS 上映射为 Cmd；
 * 与 Rust 端 user_settings.rs 的 default_shortcut_bindings 保持一致。
 */

export type ShortcutAction = "format_now" | "copy_output" | "open_settings";

export const SHORTCUT_ACTIONS: ShortcutAction[] = [
  "format_now",
  "copy_output",
  "open_settings",
];

export const SHORTCUT_ACTION_LABELS: Record<ShortcutAction, string> = {
  format_now: "立即排版",
  copy_output: "复制结果",
  open_settings: "打开设置",
};

export type ShortcutBindings = Record<ShortcutAction, string>;

export const DEFAULT_SHORTCUT_BINDINGS: ShortcutBindings = {
  format_now: "CtrlOrCmd+Enter",
  copy_output: "CtrlOrCmd+Shift+KeyC",
  open_settings: "CtrlOrCmd+Comma",
};

/** 高风险系统/窗口组合键黑名单。 */
const SYSTEM_BLACKLIST = new Set([
  "CtrlOrCmd+KeyW",
  "CtrlOrCmd+KeyQ",
  "CtrlOrCmd+KeyT",
  "CtrlOrCmd+KeyN",
]);

/** 自定义录制允许的主键：功能键、字母、数字与导航键；Comma 作为既有默认值的兼容例外。 */
const ALLOWED_CODE_PATTERN =
  /^(F[1-9]|F1[0-2]|Key[A-Z]|Digit[0-9]|Enter|Comma|Arrow(Up|Down|Left|Right)|Home|End|PageUp|PageDown|Insert)$/;

/** 仅作修饰键的 code；单独按下时不构成组合键。 */
const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "MetaLeft",
  "MetaRight",
  "ShiftLeft",
  "ShiftRight",
  "AltLeft",
  "AltRight",
]);

/**
 * 部分环境（如 userEvent/jsdom、个别输入法）对某些按键给出的
 * KeyboardEvent.code 为空或 "Unknown"；此时按 event.key 回退推断。
 */
const KEY_TO_CODE: Record<string, string> = {
  enter: "Enter",
  ",": "Comma",
  ".": "Period",
  "/": "Slash",
  ";": "Semicolon",
  "'": "Quote",
  "[": "BracketLeft",
  "]": "BracketRight",
  "\\": "Backslash",
  "-": "Minus",
  "=": "Equal",
  "`": "Backquote",
};

function effectiveCode(event: KeyboardEvent): string {
  if (event.code && event.code !== "Unknown") return event.code;
  const key = event.key?.toLowerCase() ?? "";
  if (/^[a-z]$/.test(key)) return `Key${key.toUpperCase()}`;
  if (/^[0-9]$/.test(key)) return `Digit${key}`;
  return KEY_TO_CODE[key] ?? "";
}

export interface ParsedCombo {
  ctrlOrCmd: boolean;
  shift: boolean;
  alt: boolean;
  code: string;
}

/** 解析序列化的组合键字符串；格式非法时返回 null。 */
export function parseCombo(binding: string): ParsedCombo | null {
  if (typeof binding !== "string" || binding.length === 0) return null;
  const parts = binding.split("+");
  const code = parts.pop();
  if (!code) return null;
  let ctrlOrCmd = false;
  let shift = false;
  let alt = false;
  for (const part of parts) {
    if (part === "CtrlOrCmd") ctrlOrCmd = true;
    else if (part === "Shift") shift = true;
    else if (part === "Alt") alt = true;
    else return null;
  }
  return { ctrlOrCmd, shift, alt, code };
}

export function serializeCombo(combo: ParsedCombo): string {
  const parts: string[] = [];
  if (combo.ctrlOrCmd) parts.push("CtrlOrCmd");
  if (combo.shift) parts.push("Shift");
  if (combo.alt) parts.push("Alt");
  parts.push(combo.code);
  return parts.join("+");
}

/** 从键盘事件提取组合键；仅按下修饰键或处于 IME 组合态时返回 null。 */
export function comboFromEvent(event: KeyboardEvent): ParsedCombo | null {
  if (isImeComposing(event)) return null;
  const code = effectiveCode(event);
  if (!code || MODIFIER_CODES.has(code)) return null;
  return {
    ctrlOrCmd: event.ctrlKey || event.metaKey,
    shift: event.shiftKey,
    alt: event.altKey,
    code,
  };
}

/** 输入法组合态不触发任何快捷键（isComposing，兼容 keyCode 229）。 */
export function isImeComposing(event: KeyboardEvent): boolean {
  return event.isComposing || event.keyCode === 229;
}

/** 判断事件是否精确匹配某个绑定（不允许出现多余的修饰键）。 */
export function eventMatchesShortcut(
  event: KeyboardEvent,
  binding: string,
): boolean {
  const parsed = parseCombo(binding);
  if (!parsed) return false;
  if (effectiveCode(event) !== parsed.code) return false;
  if ((event.ctrlKey || event.metaKey) !== parsed.ctrlOrCmd) return false;
  if (event.shiftKey !== parsed.shift) return false;
  if (event.altKey !== parsed.alt) return false;
  return true;
}

/**
 * 校验绑定；返回中文错误信息，合法时返回 null。
 * @param binding 待校验的组合键
 * @param others 其它动作当前占用的绑定，用于重复检测
 */
export function validateBinding(
  binding: string,
  others: Partial<Record<ShortcutAction, string>>,
): string | null {
  const parsed = parseCombo(binding);
  if (!parsed) return "无法识别的快捷键组合";
  if (!parsed.ctrlOrCmd) return "快捷键必须包含 Ctrl/Cmd 修饰键";
  for (const [action, other] of Object.entries(others)) {
    if (other && other === binding) {
      const label =
        SHORTCUT_ACTION_LABELS[action as ShortcutAction] ?? action;
      return `该组合键已被「${label}」占用`;
    }
  }
  if (SYSTEM_BLACKLIST.has(binding)) {
    return "该组合键被系统或窗口占用，请换一个";
  }
  if (!ALLOWED_CODE_PATTERN.test(parsed.code)) {
    return "不支持该按键，请使用字母、数字、功能键或方向键等组合";
  }
  return null;
}

const CODE_DISPLAY_LABELS: Record<string, string> = {
  Enter: "Enter",
  Comma: ",",
  Period: ".",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Home: "Home",
  End: "End",
  PageUp: "Page Up",
  PageDown: "Page Down",
  Insert: "Insert",
};

/** 组合键的人类可读展示，如 “Ctrl/Cmd + Shift + C”。 */
export function formatComboForDisplay(binding: string): string {
  const parsed = parseCombo(binding);
  if (!parsed) return binding;
  const parts: string[] = [];
  if (parsed.ctrlOrCmd) parts.push("Ctrl/Cmd");
  if (parsed.alt) parts.push("Alt");
  if (parsed.shift) parts.push("Shift");
  let label = parsed.code;
  if (/^Key[A-Z]$/.test(parsed.code)) {
    label = parsed.code.slice(3);
  } else if (/^Digit[0-9]$/.test(parsed.code)) {
    label = parsed.code.slice(5);
  } else if (parsed.code in CODE_DISPLAY_LABELS) {
    label = CODE_DISPLAY_LABELS[parsed.code];
  }
  parts.push(label);
  return parts.join(" + ");
}
