import { useCallback, useEffect, useRef, useState, type RefObject } from "react";

export interface UseSettingsDialogResult {
  open: boolean;
  triggerRef: RefObject<HTMLButtonElement | null>;
  onOpenChange: (open: boolean) => void;
}

/** 设置弹窗生命周期：open 状态、触发按钮引用和关闭后的焦点恢复。 */
export function useSettingsDialog(): UseSettingsDialogResult {
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const focusTimerRef = useRef<number | null>(null);

  const onOpenChange = useCallback((nextOpen: boolean) => {
    setOpen(nextOpen);
    if (!nextOpen) {
      if (focusTimerRef.current !== null) window.clearTimeout(focusTimerRef.current);
      focusTimerRef.current = window.setTimeout(() => {
        focusTimerRef.current = null;
        triggerRef.current?.focus();
      }, 0);
    }
  }, []);

  useEffect(() => {
    return () => {
      if (focusTimerRef.current !== null) window.clearTimeout(focusTimerRef.current);
    };
  }, []);

  return { open, triggerRef, onOpenChange };
}