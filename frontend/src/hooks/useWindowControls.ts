import { useCallback } from "react";

import { getCurrentWindow } from "@tauri-apps/api/window";

import { isTauri } from "@/lib/tauri";

export interface UseWindowControlsOptions {
  onError: (message: string) => void;
}

export interface UseWindowControlsResult {
  onMinimize: () => Promise<void>;
  onToggleMaximize: () => Promise<void>;
  onClose: () => Promise<void>;
  onHeaderMouseDown: (event: React.MouseEvent<HTMLElement>) => void;
}

/** 管理无边框窗口的控制、拖动和错误转换；浏览器预览模式下保持 no-op。 */
export function useWindowControls({ onError }: UseWindowControlsOptions): UseWindowControlsResult {
  const runWindowAction = useCallback(
    async (action: () => Promise<void>) => {
      if (!isTauri()) return;

      try {
        await action();
      } catch (error) {
        onError(`窗口操作失败：${String(error)}`);
      }
    },
    [onError],
  );

  const onMinimize = useCallback(
    () => runWindowAction(() => getCurrentWindow().minimize()),
    [runWindowAction],
  );

  const onToggleMaximize = useCallback(
    () => runWindowAction(() => getCurrentWindow().toggleMaximize()),
    [runWindowAction],
  );

  const onClose = useCallback(
    () => runWindowAction(() => getCurrentWindow().close()),
    [runWindowAction],
  );

  const onHeaderMouseDown = useCallback(
    (event: React.MouseEvent<HTMLElement>) => {
      if (!isTauri()) return;
      if (event.button !== 0 || event.detail > 1) return;

      const target = event.target as HTMLElement;
      if (target.closest("[data-window-control]")) return;

      void getCurrentWindow()
        .startDragging()
        .catch((error) => onError(`窗口拖动失败：${String(error)}`));
    },
    [onError],
  );

  return { onMinimize, onToggleMaximize, onClose, onHeaderMouseDown };
}