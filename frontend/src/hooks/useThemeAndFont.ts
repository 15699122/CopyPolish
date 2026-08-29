import { useEffect } from "react";

import { FONT_FAMILY_STACKS } from "@/lib/fonts";
import type { EditorFontSize, FontFamily, ThemeMode, UiScale } from "@/lib/tauri";

const EDITOR_SIZES: Record<EditorFontSize, [string, string]> = {
  small: ["13px", "1.65"],
  normal: ["14px", "1.7"],
  large: ["16px", "1.75"],
  "x-large": ["18px", "1.8"],
};

const UI_SCALES: Record<UiScale, string> = {
  compact: "0.8",
  small: "0.9",
  normal: "1",
  large: "1.1",
  "x-large": "1.25",
};

export interface UseThemeAndFontOptions {
  theme: ThemeMode;
  font: FontFamily;
  editorFontSize: EditorFontSize;
  uiScale: UiScale;
}

/** 将主题、字体、字号和缩放设置应用到 document 根节点。 */
export function useThemeAndFont({
  theme,
  font,
  editorFontSize,
  uiScale,
}: UseThemeAndFontOptions): void {
  useEffect(() => {
    const root = document.documentElement;
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    function applyTheme() {
      const effective = theme === "system" ? (mediaQuery.matches ? "dark" : "light") : theme;
      root.setAttribute("data-theme", effective);
    }

    applyTheme();
    if (theme === "system") mediaQuery.addEventListener("change", applyTheme);
    return () => mediaQuery.removeEventListener("change", applyTheme);
  }, [theme]);

  useEffect(() => {
    document.documentElement.style.setProperty("--app-font-family", FONT_FAMILY_STACKS[font]);
  }, [font]);

  useEffect(() => {
    const [size, lineHeight] = EDITOR_SIZES[editorFontSize];
    document.documentElement.style.setProperty("--editor-font-size", size);
    document.documentElement.style.setProperty("--editor-line-height", lineHeight);
  }, [editorFontSize]);

  useEffect(() => {
    document.documentElement.style.setProperty("--app-ui-scale", UI_SCALES[uiScale]);
  }, [uiScale]);
}