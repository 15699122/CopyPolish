import { Button } from "@/components/ui/button";
import type { EditorFontSize, FontFamily } from "@/lib/tauri";

interface DisplaySectionProps {
  font: FontFamily;
  onFontChange: (font: FontFamily) => void;
  onResetFont: () => void;
  editorFontSize: EditorFontSize;
  onEditorFontSizeChange: (size: EditorFontSize) => void;
}

/** 界面字体、字体回退与编辑器字号设置。 */
export function DisplaySection({
  font,
  onFontChange,
  onResetFont,
  editorFontSize,
  onEditorFontSizeChange,
}: DisplaySectionProps) {
  return (
    <div className="space-y-2" data-testid="font-settings">
      <div className="space-y-1.5">
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
  );
}