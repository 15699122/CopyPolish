import { useEffect, useRef } from "react";

import { eventMatchesShortcut, isImeComposing, type ShortcutAction, type ShortcutBindings } from "@/lib/shortcuts";

export interface UseShortcutsOptions {
  /** 总开关：false 时不注册任何应用快捷键监听。 */
  enabled: boolean;
  /** 动作 → 序列化组合键。 */
  bindings: ShortcutBindings;
  onFormatNow: () => void;
  onCopyOutput: () => void;
  onOpenSettings: () => void;
}

/**
 * 全局 keydown 监听与动作分发。
 * - 仅在窗口聚焦且事件精确匹配已启用绑定时 preventDefault；
 * - IME 组合态（isComposing / keyCode 229）不触发；
 * - Esc 交给 Radix Dialog 原生处理，不在此拦截。
 */
export function useShortcuts({
  enabled,
  bindings,
  onFormatNow,
  onCopyOutput,
  onOpenSettings,
}: UseShortcutsOptions): void {
  const callbacksRef = useRef({ onFormatNow, onCopyOutput, onOpenSettings });
  callbacksRef.current = { onFormatNow, onCopyOutput, onOpenSettings };

  useEffect(() => {
    if (!enabled) return;

    function dispatch(action: ShortcutAction) {
      const cb = callbacksRef.current;
      if (action === "format_now") cb.onFormatNow();
      else if (action === "copy_output") cb.onCopyOutput();
      else if (action === "open_settings") cb.onOpenSettings();
    }

    function onKeyDown(event: KeyboardEvent) {
      if (isImeComposing(event)) return;
      // 一个按键最多分发一个动作；未匹配时不阻止默认行为。
      for (const action of Object.keys(bindings) as ShortcutAction[]) {
        const binding = bindings[action];
        if (binding && eventMatchesShortcut(event, binding)) {
          event.preventDefault();
          dispatch(action);
          return;
        }
      }
    }

    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [enabled, bindings]);
}
