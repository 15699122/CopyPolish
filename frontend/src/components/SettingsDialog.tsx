import { useMemo, type RefObject } from "react";
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
import type { FontFamily, Rule, ThemeMode } from "@/lib/tauri";

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
  appVersion: string;
  settingsStatus: SettingsStatus;
  settingsError: string | null;
  settingsPath: string | null;
  onToggleRule: (key: string) => void;
  onSetAll: (on: boolean) => void;
  onResetDefaults: () => void;
  onThemeChange: (theme: ThemeMode) => void;
  onFontChange: (font: FontFamily) => void;
  onResetFont: () => void;
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
  appVersion,
  settingsStatus,
  settingsError,
  settingsPath,
  onToggleRule,
  onSetAll,
  onResetDefaults,
  onThemeChange,
  onFontChange,
  onResetFont,
}: SettingsDialogProps) {
  const groups = useMemo(() => {
    const map = new Map<string, Rule[]>();
    for (const rule of rules) {
      const list = map.get(rule.section) ?? [];
      list.push(rule);
      map.set(rule.section, list);
    }
    return Array.from(map.entries());
  }, [rules]);

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
              <div className="grid grid-cols-1 gap-1 sm:grid-cols-3" data-testid="theme-options">
                {([
                  ["system", "跟随系统"],
                  ["light", "浅色"],
                  ["dark", "深色"],
                ] as const).map(([value, label]) => (
                  <label
                    key={value}
                    className={cn(
                      "flex min-w-0 cursor-pointer items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors hover:bg-accent",
                      theme === value && "bg-accent text-accent-foreground",
                    )}
                  >
                    <input
                      type="radio"
                      name="theme"
                      value={value}
                      checked={theme === value}
                      onChange={() => onThemeChange(value)}
                      data-testid={`theme-${value}`}
                      className="h-4 w-4 shrink-0"
                    />
                    <span className="truncate text-sm">{label}</span>
                  </label>
                ))}
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
                {settingsPath && (
                  <span
                    className="group relative min-w-0 truncate text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    tabIndex={0}
                    title={settingsPath}
                    aria-label={`设置文件完整路径：${settingsPath}`}
                    data-testid="settings-path"
                  >
                    设置文件：{settingsPath}
                    <span role="tooltip" className="pointer-events-none absolute bottom-full left-0 z-10 mb-1 hidden w-max max-w-md break-all rounded-md border bg-card px-2 py-1 text-left text-card-foreground shadow-md group-focus-within:block group-hover:block">
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