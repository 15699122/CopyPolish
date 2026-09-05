import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import {
  SHORTCUT_ACTIONS,
  SHORTCUT_ACTION_LABELS,
  comboFromEvent,
  formatComboForDisplay,
  serializeCombo,
  validateBinding,
} from "@/lib/shortcuts";
import type { ShortcutAction, ShortcutBindings } from "@/lib/tauri";

interface ShortcutsSectionProps {
  shortcutsEnabled: boolean;
  shortcutBindings: ShortcutBindings;
  onShortcutsEnabledChange: (enabled: boolean) => void;
  onSaveShortcutBinding: (action: ShortcutAction, binding: string) => void;
  onResetShortcuts: () => void;
}

/** 快捷键总开关、绑定列表与录制反馈；录制状态仅属于本分区。 */
export function ShortcutsSection({
  shortcutsEnabled,
  shortcutBindings,
  onShortcutsEnabledChange,
  onSaveShortcutBinding,
  onResetShortcuts,
}: ShortcutsSectionProps) {
  // 录制中的动作与最近一次快捷键反馈（aria-live）。
  const [recordingAction, setRecordingAction] = useState<ShortcutAction | null>(null);
  const [shortcutMessage, setShortcutMessage] = useState<string | null>(null);

  function onRecordKeyDown(action: ShortcutAction, event: React.KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "Escape") {
      event.stopPropagation();
      setRecordingAction(null);
      setShortcutMessage(null);
      return;
    }
    const combo = comboFromEvent(event.nativeEvent);
    // 仅修饰键或 IME 组合态：等待完整组合键。
    if (!combo) return;
    event.preventDefault();
    event.stopPropagation();
    const binding = serializeCombo(combo);
    const others: Partial<Record<ShortcutAction, string>> = {};
    for (const other of SHORTCUT_ACTIONS) {
      if (other !== action) others[other] = shortcutBindings[other];
    }
    const error = validateBinding(binding, others);
    if (error) {
      setShortcutMessage(error);
      return;
    }
    onSaveShortcutBinding(action, binding);
    setRecordingAction(null);
    setShortcutMessage(
      `已将「${SHORTCUT_ACTION_LABELS[action]}」设为 ${formatComboForDisplay(binding)}`,
    );
  }

  return (
    <div className="space-y-2" data-testid="shortcut-settings">
      <div className="space-y-1.5">
        <h3 className="text-sm font-semibold">快捷键</h3>
        <p className="text-xs text-muted-foreground">
          关闭后应用不再处理任何自定义组合键；设置窗口仍可用 Esc 关闭。
        </p>
      </div>
      <div className="flex w-fit items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-accent">
        <Checkbox
          id="shortcuts-toggle"
          checked={shortcutsEnabled}
          onCheckedChange={(checked) => onShortcutsEnabledChange(checked === true)}
          data-testid="shortcuts-toggle"
        />
        <Label htmlFor="shortcuts-toggle" className="cursor-pointer text-sm">启用应用快捷键</Label>
      </div>
      <div className={cn("space-y-1.5", !shortcutsEnabled && "opacity-50")}>
        {SHORTCUT_ACTIONS.map((action) => (
          <div
            key={action}
            className="flex items-center justify-between gap-3 rounded-md border p-2.5"
            data-testid={`shortcut-row-${action}`}
          >
            <span className="min-w-0 truncate text-sm">{SHORTCUT_ACTION_LABELS[action]}</span>
            <span className="flex shrink-0 items-center gap-2">
              <kbd
                className="rounded border bg-muted px-2 py-0.5 font-mono text-xs"
                data-testid={`shortcut-value-${action}`}
              >
                {formatComboForDisplay(shortcutBindings[action])}
              </kbd>
              <Button
                variant="ghost"
                size="sm"
                disabled={!shortcutsEnabled}
                data-testid={`shortcut-edit-${action}`}
                onKeyDown={(event) => onRecordKeyDown(action, event)}
                onClick={() => {
                  setRecordingAction(action);
                  setShortcutMessage(null);
                }}
              >
                {recordingAction === action ? "请按下新组合键（Esc 取消）" : "修改"}
              </Button>
            </span>
          </div>
        ))}
      </div>
      <span
        className="block text-xs text-muted-foreground"
        data-testid="shortcut-status"
        aria-live="polite"
      >
        {shortcutMessage ?? ""}
      </span>
      <Button
        variant="ghost"
        size="sm"
        disabled={!shortcutsEnabled}
        data-testid="reset-shortcuts"
        onClick={() => {
          setRecordingAction(null);
          setShortcutMessage("已恢复默认快捷键");
          onResetShortcuts();
        }}
      >
        恢复默认快捷键
      </Button>
    </div>
  );
}