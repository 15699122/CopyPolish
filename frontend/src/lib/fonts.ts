// frontend/src/lib/fonts.ts
// =============================================================================
// 统一字体令牌：全应用唯一的字体栈定义。
//
// 用户选择的字体预设写入 CSS 变量 `--app-font-family`（documentElement），
// index.css 中 html/body 与所有原生表单控件均从该变量继承，保证标题、
// 输入框、输出区、按钮与设置弹窗使用同一种字体。
// =============================================================================

import type { FontFamily } from "./tauri";

export const FONT_FAMILY_STACKS: Record<FontFamily, string> = {
  system: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  "microsoft-yahei": '"Microsoft YaHei", "微软雅黑", system-ui, sans-serif',
  pingfang: '"PingFang SC", "苹方", system-ui, sans-serif',
  "noto-sans-cjk": '"Noto Sans CJK SC", "Source Han Sans SC", system-ui, sans-serif',
  simsun: 'SimSun, "宋体", serif',
  simhei: 'SimHei, "黑体", system-ui, sans-serif',
};
