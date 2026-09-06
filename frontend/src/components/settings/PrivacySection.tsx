import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";

interface PrivacySectionProps {
  restoreLastInput: boolean;
  onRestoreLastInputChange: (enabled: boolean) => void;
  onClearSavedInput: () => void;
}

/**
 * 隐私设置：控制应用是否将输入正文持久化到本地设置文件。
 *
 * 默认关闭"启动时恢复上次输入"，用户正文不会写入 rules.yaml；
 * 仅在用户显式开启后才保存，并可一键清除已保存的正文。
 */
export function PrivacySection({
  restoreLastInput,
  onRestoreLastInputChange,
  onClearSavedInput,
}: PrivacySectionProps) {
  return (
    <div className="space-y-3">
      <h3 className="text-sm font-semibold">隐私</h3>
      <div className="space-y-3 rounded-md border border-border bg-card/40 p-3">
        <div className="flex items-start gap-2">
          <Checkbox
            id="restore-last-input"
            checked={restoreLastInput}
            onCheckedChange={(checked) => onRestoreLastInputChange(checked === true)}
            data-testid="restore-last-input"
            className="mt-0.5"
          />
          <div className="space-y-1">
            <Label htmlFor="restore-last-input" className="cursor-pointer text-sm font-medium">
              启动时恢复上次输入
            </Label>
            <p className="text-xs text-muted-foreground">
              开启后，应用退出时会保存当前输入框的正文，下次启动自动恢复。
              正文将以明文保存在本地设置文件（rules.yaml）中。
              关闭时正文不会被保存，也不会恢复。
            </p>
          </div>
        </div>
        {restoreLastInput && (
          <div className="flex items-center justify-between border-t border-border pt-2">
            <span className="text-xs text-muted-foreground">
              已保存的明文正文可随时清除，关闭本开关也会在下次保存时清除。
            </span>
            <button
              type="button"
              onClick={onClearSavedInput}
              data-testid="clear-saved-input"
              className="shrink-0 rounded-md border border-destructive/40 bg-background px-3 py-1 text-xs text-destructive transition-colors hover:bg-destructive/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive"
            >
              清除已保存正文
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
