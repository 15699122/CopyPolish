import { Maximize2, Minus, X } from "lucide-react";

import { Button } from "@/components/ui/button";

interface AppTitleBarProps {
  appName: string;
  referenceName: string;
  tauri: boolean;
  onMouseDown: (event: React.MouseEvent<HTMLElement>) => void;
  onDoubleClick: () => void;
  onMinimize: () => void;
  onToggleMaximize: () => void;
  onClose: () => void;
}

/** 无边框窗口标题栏；窗口行为由 App 注入，组件只负责呈现和事件转发。 */
export function AppTitleBar({
  appName,
  referenceName,
  tauri,
  onMouseDown,
  onDoubleClick,
  onMinimize,
  onToggleMaximize,
  onClose,
}: AppTitleBarProps) {
  return (
    <header
      className="flex select-none items-center justify-between border-b px-6 py-3"
      data-testid="title-bar"
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
    >
      <div className="min-w-0 space-y-1.5">
        <h1 className="text-xl font-bold leading-none">{appName}</h1>
        <p className="text-xs leading-relaxed text-muted-foreground">
          {tauri
            ? `实时保护 LaTeX / Markdown 结构 · ${referenceName}`
            : "浏览器预览模式 · 内置回退排版"}
        </p>
      </div>
      {tauri && (
        <div
          className="flex items-center gap-1"
          data-window-control
          onMouseDown={(event) => event.stopPropagation()}
        >
          <Button variant="ghost" size="icon" onClick={onMinimize} aria-label="最小化" data-window-control>
            <Minus className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon" onClick={onToggleMaximize} aria-label="最大化或还原" data-window-control>
            <Maximize2 className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon" onClick={onClose} aria-label="关闭" data-window-control>
            <X className="h-4 w-4" />
          </Button>
        </div>
      )}
    </header>
  );
}