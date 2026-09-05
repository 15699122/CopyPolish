import { cn } from "@/lib/utils";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import type { ThemeMode, UiScale } from "@/lib/tauri";

interface ThemeSectionProps {
  theme: ThemeMode;
  onThemeChange: (theme: ThemeMode) => void;
  onFollowSystemChange: (follow: boolean) => void;
  uiScale: UiScale;
  onUiScaleChange: (scale: UiScale) => void;
}

/** 主题选择（跟随系统 / 浅色 / 深色）与主界面缩放。 */
export function ThemeSection({
  theme,
  onThemeChange,
  onFollowSystemChange,
  uiScale,
  onUiScaleChange,
}: ThemeSectionProps) {
  const followingSystem = theme === "system";

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-semibold">主题</h3>
      <div className="grid w-full grid-cols-3 gap-2" data-testid="theme-options">
        <div className="flex w-full min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors hover:bg-accent">
          <Checkbox
            id="theme-system"
            checked={followingSystem}
            onCheckedChange={(checked) => onFollowSystemChange(checked === true)}
            data-testid="theme-system"
          />
          <Label htmlFor="theme-system" className="cursor-pointer text-sm">跟随系统</Label>
        </div>
        {([
          ["light", "浅色"],
          ["dark", "深色"],
        ] as const).map(([value, label]) => (
          <label
            key={value}
            className={cn(
              "flex w-full min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors",
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
  );
}