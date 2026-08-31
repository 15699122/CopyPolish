import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { isSettingsLoadNoticeAlert, settingsLoadNoticeText } from "@/lib/settingsLoadNotices";
import type { SettingsLoadNotice } from "@/lib/tauri";

function abbreviatePath(path: string, maxLength = 56): string {
  if (path.length <= maxLength) return path;

  const separator = path.includes("\\") ? "\\" : "/";
  const parts = path.split(separator);
  const isWindowsDrive = /^[A-Za-z]:$/.test(parts[0] ?? "");
  const prefix = isWindowsDrive ? `${parts[0]}${separator}` : path.startsWith(separator) ? separator : "";
  const remainder = isWindowsDrive || prefix ? parts.slice(1) : parts;
  const suffixParts = remainder.slice(-2);
  const suffix = suffixParts.join(separator);
  const available = Math.max(8, maxLength - prefix.length - suffix.length - 2);
  const leading = remainder.slice(0, -2).join(separator);
  return `${prefix}${leading.slice(0, available)}…${separator}${suffix}`;
}

/** 设置保存状态；由 App 中的 useSettingsPersistence 提供。 */
export type SettingsStatus = "idle" | "saving" | "saved" | "error";

interface SettingsFooterProps {
  appVersion: string;
  settingsStatus: SettingsStatus;
  settingsError: string | null;
  settingsLoadNotices: SettingsLoadNotice[];
  settingsPath: string | null;
  onSetAll: (on: boolean) => void;
  onResetDefaults: () => void;
  onOpenChange: (open: boolean) => void;
}

/** 设置弹窗底部：版本、保存状态、设置路径与规则操作/完成按钮。 */
export function SettingsFooter({
  appVersion,
  settingsStatus,
  settingsError,
  settingsLoadNotices,
  settingsPath,
  onSetAll,
  onResetDefaults,
  onOpenChange,
}: SettingsFooterProps) {
  return (
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
                role={isSettingsLoadNoticeAlert(notice) ? "alert" : "status"}
                aria-live={isSettingsLoadNoticeAlert(notice) ? "assertive" : "polite"}
              >
                {settingsLoadNoticeText(notice)}
              </span>
            ))}
            {settingsPath && (
              <span
                className="min-w-0 text-muted-foreground"
                data-testid="settings-path-label"
              >
                设置文件：
                <span
                  className="inline-block max-w-full min-w-0 align-bottom underline decoration-dotted decoration-muted-foreground/60 underline-offset-4 outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  tabIndex={0}
                  title={settingsPath}
                  aria-label={`设置文件完整路径：${settingsPath}`}
                  data-testid="settings-path"
                >
                  {abbreviatePath(settingsPath)}
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
  );
}