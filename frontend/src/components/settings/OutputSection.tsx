import type { LayoutMode, OutputMode } from "@/lib/tauri";

interface OutputSectionProps {
  outputMode: OutputMode;
  layoutMode: LayoutMode;
  onOutputModeChange: (mode: OutputMode) => void;
  onLayoutModeChange: (mode: LayoutMode) => void;
}

/** 输出更新策略与主界面布局。 */
export function OutputSection({
  outputMode,
  layoutMode,
  onOutputModeChange,
  onLayoutModeChange,
}: OutputSectionProps) {
  return (
    <section className="space-y-3" data-testid="output-settings">
      <div>
        <h3 className="text-sm font-semibold">输出与布局</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          手动模式不会因输入变化自动刷新输出，可使用“立即排版”快捷键更新。
        </p>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <label className="space-y-1.5">
          <span className="text-xs font-medium text-muted-foreground">输出模式</span>
          <select
            value={outputMode}
            onChange={(event) => onOutputModeChange(event.target.value as OutputMode)}
            data-testid="output-mode-select"
            aria-label="输出模式"
            className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="realtime">实时输出</option>
            <option value="manual">手动输出</option>
          </select>
        </label>
        <label className="space-y-1.5">
          <span className="text-xs font-medium text-muted-foreground">输入/输出布局</span>
          <select
            value={layoutMode}
            onChange={(event) => onLayoutModeChange(event.target.value as LayoutMode)}
            data-testid="layout-mode-select"
            aria-label="输入输出布局"
            className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="auto">自动（宽屏左右，小屏上下）</option>
            <option value="horizontal">左右布局</option>
            <option value="vertical">上下布局</option>
          </select>
        </label>
      </div>
    </section>
  );
}