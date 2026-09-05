import { useCallback, useState } from "react";

export const FIRST_RUN_NOTICE_STORAGE_KEY = "copypolish.first-run-notice-seen";

function hasSeenNotice(): boolean {
  try {
    return window.localStorage.getItem(FIRST_RUN_NOTICE_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

/** 管理仅在当前浏览器/用户配置中展示一次的首次使用提示。 */
export function useFirstRunNotice() {
  const [visible, setVisible] = useState(() => !hasSeenNotice());

  const dismiss = useCallback(() => {
    setVisible(false);
    try {
      window.localStorage.setItem(FIRST_RUN_NOTICE_STORAGE_KEY, "1");
    } catch {
      // localStorage 不可用时仍允许本次运行关闭提示。
    }
  }, []);

  return { visible, dismiss };
}