import { useMemo, useState, type RefObject } from "react";
import { Settings } from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
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
import type {
  EditorFontSize,
  FontFamily,
  Rule,
  SettingsLoadNotice,
  ShortcutAction,
  ShortcutBindings,
  ThemeMode,
  UiScale,
} from "@/lib/tauri";

type SettingsStatus = "idle" | "saving" | "saved" | "error";

interface SettingsDialogProps {
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
  settingsLoadNotices: SettingsLoadNotice[];
  appVersion: string;
  settingsStatus: SettingsStatus;
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
}

/** 设置弹窗及规则列表；状态和持久化行为由 App 注入。 */
export function SettingsDialog({
  open,
  onOpenChange,
  triggerRef,
  rules,
  enabled,
  enabledSet,
  theme,
  font,
  editorFontSize,
  uiScale,
  settingsLoadNotices,
  appVersion,
  settingsStatus,
  settingsError,
  settingsPath,
  onToggleRule,
  onSetAll,
  onResetDefaults,
  onThemeChange,
  onFollowSystemChange,
  onFontChange,
  onResetFont,
  onEditorFontSizeChange,
  onUiScaleChange,
  shortcutsEnabled,
  shortcutBindings,
  onShortcutsEnabledChange,
  onSaveShortcutBinding,
  onResetShortcuts,
}: SettingsDialogProps) {
  const groups = useMemo(() => {
    const map = new Map<string, Rule[]>();
    // 仅影响设置窗口的展示顺序：默认开启的规则在上，默认关闭的在下；
    // 同类内部保持后端返回顺序，不影响 Rust pipeline 的实际执行顺序。
    const sorted = [...rules].sort((a, b) => Number(b.default) - Number(a.default));
    for (const rule of sorted) {
      const list = map.get(rule.section) ?? [];
      list.push(rule);
      map.set(rule.section, list);
    }
    return Array.from(map.entries());
  }, [rules]);

  const followingSystem = theme === "system";

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
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button ref={triggerRef} variant="outline" size="sm" data-testid="open-settings" aria-label="打开设置">
          <Settings className="h-4 w-4" />
          设置
        </Button>
      </DialogTrigger>
      <DialogContent
        data-testid="settings-dialog"
        className="flex h-[min(680px,calc(100vh-2rem))] w-[min(560px,calc(100vw-2rem))] max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)] flex-col gap-0 overflow-hidden p-0 sm:min-h-130 sm:min-w-120"
      >
        <DialogHeader className="shrink-0 border-b px-6 py-5 pr-12">
          <DialogTitle>设置 — 排版规则</DialogTitle>
          <DialogDescription>
            逐条启用/停用规则。已启用 {enabled.length}/{rules.length} 条
          </DialogDescription>
        </DialogHeader>

        <div className="min-h-0 flex-1 overflow-y-auto px-6 py-4" data-testid="settings-scroll-area">
          <div className="space-y-6 pb-4">
            <div className="space-y-2">
              <h3 className="text-sm font-semibold">主题</h3>
              <div className="space-y-1.5" data-testid="theme-options">
                <label className="flex w-fit min-w-0 cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-accent">
                  <input
                    type="checkbox"
                    checked={followingSystem}
                    onChange={(event) => onFollowSystemChange(event.target.checked)}
                    data-testid="theme-system"
                    className="h-4 w-4 shrink-0"
                  />
                  <span className="text-sm">跟随系统</span>
                </label>
                <div className="grid grid-cols-1 gap-1 sm:grid-cols-2">
                  {([
                    ["light", "浅色"],
                    ["dark", "深色"],
                  ] as const).map(([value, label]) => (
                    <label
                      key={value}
                      className={cn(
                        "flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors",
                        !followingSystem && "cursor-pointer hover:bg-accent",
                        !followingSystem && theme === value && "bg-accent text-accent-foreground",
                        followingSystem && "cursor-not-allowed opacity-50",
                      )}
                    >
                      <input
                        type="radio"
                        name="theme"
                        value={value}
                        checked={theme === value}
                        disabled={followingSystem}
                        onChange={() => onThemeChange(value)}
                        data-testid={`theme-${value}`}
                        className="h-4 w-4 shrink-0"
                      />
                      <span className="truncate text-sm">{label}</span>
                    </label>
                  ))}
                </div>
              </div>
              <div className="space-y-1.5" data-testid="ui-scale-settings">
                <h4 className="text-xs font-medium text-muted-foreground">缩放</h4>
                <select
                  value={uiScale}
                  onChange={(event) => onUiScaleChange(event.target.value as UiScale)}
                  data-testid="ui-scale-select"
                  aria-label="主界面缩放"
                  className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring sm:max-w-60"
                >
                  <option value="compact">80%</option>
                  <option value="small">90%</option>
                  <option value="normal">100%</option>
                  <option value="large">110%</option>
                  <option value="x-large">125%</option>
                </select>
              </div>
            </div>

            <div className="space-y-2" data-testid="font-settings">
              <div>
                <h3 className="text-sm font-semibold">字体</h3>
                <p className="text-xs text-muted-foreground">选择界面显示字体；未安装的字体会自动使用系统回退字体。</p>
              </div>
              <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                <select
                  value={font}
                  onChange={(event) => onFontChange(event.target.value as FontFamily)}
                  data-testid="font-select"
                  aria-label="界面字体"
                  className="h-9 min-w-0 flex-1 rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <option value="system">系统默认（推荐）</option>
                  <option value="microsoft-yahei">微软雅黑</option>
                  <option value="pingfang">苹方</option>
                  <option value="noto-sans-cjk">思源黑体 / Noto Sans CJK SC</option>
                  <option value="simsun">宋体</option>
                  <option value="simhei">黑体</option>
                </select>
                <Button variant="ghost" size="sm" data-testid="reset-font" onClick={onResetFont}>
                  恢复默认字体
                </Button>
              </div>
              <div className="space-y-1.5" data-testid="editor-font-size-settings">
                <h4 className="text-xs font-medium text-muted-foreground">字号</h4>
                <select
                  value={editorFontSize}
                  onChange={(event) => onEditorFontSizeChange(event.target.value as EditorFontSize)}
                  data-testid="editor-font-size-select"
                  aria-label="编辑器字号"
                  className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring sm:max-w-60"
                >
                  <option value="small">小</option>
                  <option value="normal">标准</option>
                  <option value="large">大</option>
                  <option value="x-large">特大</option>
                </select>
              </div>
            </div>

            <div className="space-y-2" data-testid="shortcut-settings">
              <div>
                <h3 className="text-sm font-semibold">快捷键</h3>
                <p className="text-xs text-muted-foreground">
                  关闭后应用不再处理任何自定义组合键；设置窗口仍可用 Esc 关闭。
                </p>
              </div>
              <label className="flex w-fit cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-accent">
                <input
                  type="checkbox"
                  checked={shortcutsEnabled}
                  onChange={(event) => onShortcutsEnabledChange(event.target.checked)}
                  data-testid="shortcuts-toggle"
                  className="h-4 w-4 shrink-0"
                />
                <span className="text-sm">启用应用快捷键</span>
              </label>
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
                variant="secondary"
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

            {groups.map(([section, items]) => (
              <section key={section}>
                <h3 className="mb-2 text-sm font-semibold">{section}</h3>
                <div className="space-y-2">
                  {items.map((rule) => (
                    <div key={rule.key} className="flex items-start gap-3 rounded-md border p-3">
                      <Checkbox
                        id={`rule-${rule.key}`}
                        checked={enabledSet.has(rule.key)}
                        onCheckedChange={() => onToggleRule(rule.key)}
                        data-testid={`rule-${rule.key}`}
                        aria-label={rule.name}
                      />
                      <Label htmlFor={`rule-${rule.key}`} className="min-w-0 flex-1 text-sm leading-5">
                        {rule.name}
                        {rule.disputed && (
                          <span className="ml-1 text-xs text-muted-foreground">（争议，默认关闭）</span>
                        )}
                      </Label>
                    </div>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </div>

        <DialogFooter className="shrink-0 border-t px-4 py-4 sm:px-6" data-testid="settings-footer">
          <div className="flex w-full min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div className="min-w-0 flex-1 text-left text-xs leading-5" data-testid="settings-file-info">
              <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0">
                <span className="shrink-0 text-muted-foreground" data-testid="settings-version">版本 {appVersion}</span>
                {settingsStatus === "saving" && <span className="shrink-0 text-muted-foreground" data-testid="settings-status" aria-live="polite">正在保存…</span>}
                {settingsStatus === "saved" && <span className="shrink-0 text-green-600" data-testid="settings-status" aria-live="polite">设置已保存</span>}
                {settingsStatus === "error" && (
                  <span className="break-all text-destructive" data-testid="settings-status" aria-live="assertive">设置保存失败：{settingsError}</span>
                )}
                {settingsLoadNotices.map((notice) => (
                  <span
                    key={notice}
                    className="break-all text-amber-700 dark:text-amber-300"
                    data-testid={`settings-load-notice-${notice}`}
                    role={notice === "primary_settings_corrupt_no_usable_backup" || notice === "legacy_settings_corrupt" ? "alert" : "status"}
                    aria-live={notice === "primary_settings_corrupt_no_usable_backup" || notice === "legacy_settings_corrupt" ? "assertive" : "polite"}
                  >
                    {notice === "legacy_settings_detected" && "检测到旧版本设置文件，已迁移至 rules.yaml。"}
                    {notice === "legacy_settings_corrupt" && "检测到旧版本设置文件，但内容无法读取，已使用默认设置。"}
                    {notice === "primary_settings_corrupt_recovered_from_backup" && "设置文件损坏，已从 rules.yaml.bak 恢复。"}
                    {notice === "primary_settings_corrupt_no_usable_backup" && "设置文件损坏，且备份文件也无法读取，已使用默认设置。"}
                    {notice === "backup_settings_corrupt" && "备份文件损坏，当前 rules.yaml 仍可正常使用。"}
                  </span>
                ))}
                {settingsPath && (
                  <span
                    className="min-w-0 text-muted-foreground"
                    data-testid="settings-path-label"
                  >
                    设置文件：
                    <span
                      className="relative inline max-w-full truncate align-bottom underline decoration-dotted decoration-muted-foreground/60 underline-offset-4 outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      tabIndex={0}
                      title={settingsPath}
                      aria-label={`设置文件完整路径：${settingsPath}`}
                      data-testid="settings-path"
                    >
                      {settingsPath}
                    </span>
                  </span>
                )}
              </div>
            </div>

            <div className="flex shrink-0 flex-wrap items-center justify-end gap-2" data-testid="settings-actions">
              <Button variant="outline" size="sm" data-testid="select-all" onClick={() => onSetAll(true)}>全选</Button>
              <Button variant="outline" size="sm" data-testid="select-none" onClick={() => onSetAll(false)}>全不选</Button>
              <Button variant="secondary" size="sm" data-testid="reset-defaults" onClick={onResetDefaults}>恢复默认</Button>
              <Button size="sm" data-testid="settings-done" onClick={() => onOpenChange(false)}>完成</Button>
            </div>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}