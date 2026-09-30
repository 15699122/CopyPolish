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
  const subtitle = tauri
    ? `本地排版 · 保护 LaTeX / Markdown 结构 · ${referenceName}`
    : "浏览器预览 · 内置回退排版";

  return (
    <header
      className="flex select-none items-center justify-between gap-4 border-b px-5 py-2.5"
      data-testid="title-bar"
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
    >
      <div className="flex min-w-0 items-baseline gap-3">
        <h1 className="shrink-0 text-lg font-bold leading-none tracking-wide">{appName}</h1>
        <p className="prose-helper min-w-0 truncate text-muted-foreground">{subtitle}</p>
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