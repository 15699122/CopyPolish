import { useEffect, useMemo, useRef, useState, type MouseEvent } from "react";
import { Check, Copy, Eraser } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { AppTitleBar } from "@/components/AppTitleBar";
import { SettingsDialog } from "@/components/SettingsDialog";
import { FONT_FAMILY_STACKS } from "@/lib/fonts";
import { useShortcuts } from "@/hooks/useShortcuts";
import {
  formatText,
  getAppVersion,
  getEnabledDefaults,
  getRules,
  getSettingsPath,
  getUserSettings,
  isTauri,
  saveUserSettings,
  DEFAULT_SHORTCUT_SETTINGS,
  type Rule,
  type RuleSelection,
  type FontFamily,
  type EditorFontSize,
  type LoadedUserSettings,
  type SettingsLoadNotice,
  type ShortcutAction,
  type ShortcutBindings,
  type ThemeMode,
  type UiScale,
} from "@/lib/tauri";

export const APP_NAME = "文案净排";
const APP_REFERENCE_NAME = "CopyPolish";
const NORMAL_DEBOUNCE_MS = 160;
const LONG_TEXT_DEBOUNCE_MS = 450;
const VERY_LONG_TEXT_DEBOUNCE_MS = 900;
const LONG_TEXT_THRESHOLD = 50_000;
const VERY_LONG_TEXT_THRESHOLD = 200_000;
const SLOW_FORMAT_THRESHOLD_MS = 100;

/**
 * 主界面：左输入 / 右输出（小窗口时上下堆叠）+ 操作栏 + 规则设置对话框。
 * 排版由 Tauri 侧 Rust 引擎完成；浏览器预览时走内置演示回退实现。
 */
export default function App() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [rules, setRules] = useState<Rule[]>([]);
  const [enabled, setEnabled] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isFormatting, setIsFormatting] = useState(false);
  const [lastFormatDuration, setLastFormatDuration] = useState<number | null>(null);
  const [copied, setCopied] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const settingsTriggerRef = useRef<HTMLButtonElement | null>(null);

  const debounceRef = useRef<number | null>(null);
  const settingsDebounceRef = useRef<number | null>(null);
  const seqRef = useRef(0);

  // 规则加载完成后置 true，避免恢复流程把用户设置覆盖为默认值。
  const hydratedRef = useRef(false);

  const enabledSet = useMemo(() => new Set(enabled), [enabled]);

  // 设置持久化状态：saving / saved / error，用于在设置弹窗中给出可见反馈。
  const [settingsStatus, setSettingsStatus] = useState<
    "idle" | "saving" | "saved" | "error"
  >("idle");
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [settingsLoadNotices, setSettingsLoadNotices] = useState<SettingsLoadNotice[]>([]);
  const [settingsPath, setSettingsPath] = useState<string | null>(null);
  const [appVersion, setAppVersion] = useState(__APP_VERSION__);

  // 主题状态：system / light / dark。
  const [theme, setTheme] = useState<ThemeMode>("system");
  const [font, setFont] = useState<FontFamily>("system");
  const [editorFontSize, setEditorFontSize] = useState<EditorFontSize>("normal");
  const [uiScale, setUiScale] = useState<UiScale>("normal");
  // 快捷键：总开关与动作绑定，随用户设置持久化。
  const [shortcutsEnabled, setShortcutsEnabled] = useState(
    DEFAULT_SHORTCUT_SETTINGS.enabled,
  );
  const [shortcutBindings, setShortcutBindings] = useState<ShortcutBindings>(
    DEFAULT_SHORTCUT_SETTINGS.bindings,
  );

  function currentSettings(next: Partial<{
    enabled: string[];
    last_input: string;
    theme: ThemeMode;
    font: FontFamily;
    editor_font_size: EditorFontSize;
    ui_scale: UiScale;
    shortcuts: { enabled: boolean; bindings: ShortcutBindings };
  }> = {}) {
    return {
      enabled,
      last_input: input,
      theme,
      font,
      editor_font_size: editorFontSize,
      ui_scale: uiScale,
      shortcuts: {
        enabled: shortcutsEnabled,
        bindings: shortcutBindings,
      },
      ...next,
    };
  }

  function persistSettings(nextEnabled: string[], nextInput: string) {
    if (!hydratedRef.current) return;
    setSettingsStatus("saving");
    saveUserSettings(currentSettings({ enabled: nextEnabled, last_input: nextInput }))
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        // 持久化失败不打断排版主流程，但必须让用户看到原因。
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function schedulePersist(nextEnabled: string[], nextInput: string) {
    if (settingsDebounceRef.current !== null) window.clearTimeout(settingsDebounceRef.current);
    settingsDebounceRef.current = window.setTimeout(() => {
      persistSettings(nextEnabled, nextInput);
    }, NORMAL_DEBOUNCE_MS);
  }

  // 初始化：加载规则与默认启用集；随后恢复上次保存的用户设置（若有）。
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [ruleList, defaults] = await Promise.all([
          getRules(),
          getEnabledDefaults(),
        ]);
        if (cancelled) return;
        setRules(ruleList);

        let saved: LoadedUserSettings | null = null;
        try {
          saved = await getUserSettings();
        } catch {
          saved = null;
        }
        if (cancelled) return;

        try {
          const path = await getSettingsPath();
          if (!cancelled && path) setSettingsPath(path);
        } catch {
          // 路径获取失败不影响主流程；保存错误会在保存时展示。
        }
        try {
          const version = await getAppVersion();
          if (!cancelled) setAppVersion(version);
        } catch {
          // 读取版本失败时保留构建时注入的浏览器回退版本。
        }
        if (cancelled) return;

        if (saved && Array.isArray(saved.enabled)) {
          const restoredEnabled = saved.enabled.filter((k) =>
            ruleList.some((r) => r.key === k),
          );
          setEnabled(restoredEnabled);
          if (saved.theme !== undefined) {
            setTheme(saved.theme);
          }
          if (saved.font !== undefined) {
            setFont(saved.font);
          }
          setEditorFontSize(saved.editor_font_size ?? "normal");
          setUiScale(saved.ui_scale ?? "normal");
          if (saved.shortcuts) {
            setShortcutsEnabled(saved.shortcuts.enabled);
            setShortcutBindings(saved.shortcuts.bindings);
          }
          setSettingsLoadNotices(saved.notices ?? []);
          if (saved.last_input) {
            setInput(saved.last_input);
            scheduleFormat(saved.last_input, restoredEnabled);
          }
        } else {
          setEnabled(defaults.filter((k) => ruleList.some((r) => r.key === k)));
        }
        hydratedRef.current = true;
      } catch (e) {
        setError(String(e));
        hydratedRef.current = true;
      }
    })();
    return () => {
      cancelled = true;
    };
        // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 主题应用：更新 document.documentElement 的 data-theme 属性。
  // system 模式下跟随 prefers-color-scheme；切换时自动更新。
  useEffect(() => {
    const root = document.documentElement;
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    function applyTheme() {
      const effective = theme === "system" ? (mediaQuery.matches ? "dark" : "light") : theme;
      root.setAttribute("data-theme", effective);
    }

    applyTheme();
    if (theme === "system") {
      mediaQuery.addEventListener("change", applyTheme);
    }
    return () => {
      mediaQuery.removeEventListener("change", applyTheme);
    };
  }, [theme]);

  useEffect(() => {
    document.documentElement.style.setProperty("--app-font-family", FONT_FAMILY_STACKS[font]);
  }, [font]);

  useEffect(() => {
    const sizes: Record<EditorFontSize, [string, string]> = {
      small: ["13px", "1.65"],
      normal: ["14px", "1.7"],
      large: ["16px", "1.75"],
      "x-large": ["18px", "1.8"],
    };
    const [size, lineHeight] = sizes[editorFontSize];
    document.documentElement.style.setProperty("--editor-font-size", size);
    document.documentElement.style.setProperty("--editor-line-height", lineHeight);
  }, [editorFontSize]);

  useEffect(() => {
    const scales: Record<UiScale, string> = {
      compact: "0.8",
      small: "0.9",
      normal: "1",
      large: "1.1",
      "x-large": "1.25",
    };
    document.documentElement.style.setProperty("--app-ui-scale", scales[uiScale]);
  }, [uiScale]);

  // 快捷键监听与分发：总开关关闭时不注册；IME 组合态不触发；
  // 仅在精确匹配绑定时 preventDefault。Esc 仍由 Radix Dialog 原生处理。
  useShortcuts({
    enabled: shortcutsEnabled,
    bindings: shortcutBindings,
    onFormatNow: () => scheduleFormat(input, enabled, 0),
    onCopyOutput: () => {
      void onCopy();
    },
    onOpenSettings: () => setSettingsOpen(true),
  });

  // 实时排版（防抖 + 忽略乱序的旧请求）
  function scheduleFormat(nextInput: string, applyEnabled = enabled, delayOverride?: number) {
    if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
    const debounceMs = getFormatDebounceMs(nextInput);
    debounceRef.current = window.setTimeout(async () => {
      const seq = ++seqRef.current;
      const startedAt = performance.now();
      setIsFormatting(true);
      try {
        const result = await formatText({
          text: nextInput,
          selection: getRuleSelection(applyEnabled),
        });
        if (seqRef.current === seq) {
          setOutput(result);
          setLastFormatDuration(Math.round(performance.now() - startedAt));
          setError(null);
        }
      } catch (e) {
        if (seqRef.current === seq) setError(String(e));
      } finally {
        if (seqRef.current === seq) setIsFormatting(false);
      }
    }, delayOverride ?? debounceMs);
  }

  function getFormatDebounceMs(text: string): number {
    if (text.length >= VERY_LONG_TEXT_THRESHOLD) return VERY_LONG_TEXT_DEBOUNCE_MS;
    if (text.length >= LONG_TEXT_THRESHOLD) return LONG_TEXT_DEBOUNCE_MS;
    return NORMAL_DEBOUNCE_MS;
  }

  function getRuleSelection(selected: string[]): RuleSelection {
    if (selected.length === 0) return { mode: "none" };
    if (selected.length === rules.length && rules.length > 0) return { mode: "all" };
    return { mode: "only", keys: selected };
  }

  function onInputChange(value: string) {
    setInput(value);
    scheduleFormat(value);
    schedulePersist(enabled, value);
  }

  function onToggleRule(key: string) {
    let next: string[];
    if (enabledSet.has(key)) {
      next = enabled.filter((k) => k !== key);
    } else {
      next = [...enabled, key];
    }
    setEnabled(next);
    scheduleFormat(input, next); // 规则变更后立即重排
    persistSettings(next, input);
  }

  function onSetAll(on: boolean) {
    const next = on ? rules.map((r) => r.key) : [];
    setEnabled(next);
    scheduleFormat(input, next);
    persistSettings(next, input);
  }

  function onResetDefaults() {
    const next = rules.filter((r) => r.default).map((r) => r.key);
    setEnabled(next);
    scheduleFormat(input, next);
    persistSettings(next, input);
  }

  async function onCopy() {
    if (!output) return;
    try {
      await navigator.clipboard.writeText(output);
    } catch (e) {
      setError(String(e));
      return;
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  }

  function onSettingsOpenChange(open: boolean) {
    setSettingsOpen(open);
    if (!open) {
      window.setTimeout(() => settingsTriggerRef.current?.focus(), 0);
    }
  }

  function onClear() {
    setInput("");
    setOutput("");
    if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
    setError(null);
    setCleared(true);
    window.setTimeout(() => setCleared(false), 1200);
    persistSettings(enabled, "");
  }

  async function runWindowAction(action: () => Promise<void>) {
    if (!isTauri()) return;

    try {
      await action();
    } catch (error) {
      setError(`窗口操作失败：${String(error)}`);
    }
  }

  function onMinimize() {
    return runWindowAction(() => getCurrentWindow().minimize());
  }

  function onToggleMaximize() {
    return runWindowAction(() => getCurrentWindow().toggleMaximize());
  }

  function onClose() {
    return runWindowAction(() => getCurrentWindow().close());
  }

  function onThemeChange(nextTheme: ThemeMode) {
    setTheme(nextTheme);
    setSettingsStatus("saving");
    saveUserSettings(currentSettings({ theme: nextTheme }))
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  // “跟随系统”勾选框：勾选时进入 system 模式；取消勾选时
  // 立即按当前系统偏好（prefers-color-scheme）切换到显式的 light/dark。
  function onFollowSystemChange(follow: boolean) {
    if (follow) {
      onThemeChange("system");
      return;
    }
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    onThemeChange(prefersDark ? "dark" : "light");
  }

  function onFontChange(nextFont: FontFamily) {
    setFont(nextFont);
    setSettingsStatus("saving");
    saveUserSettings(currentSettings({ font: nextFont }))
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function onResetFont() {
    onFontChange("system");
  }

  function onEditorFontSizeChange(nextSize: EditorFontSize) {
    setEditorFontSize(nextSize);
    setSettingsStatus("saving");
    saveUserSettings(currentSettings({ editor_font_size: nextSize }))
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function onUiScaleChange(nextScale: UiScale) {
    setUiScale(nextScale);
    setSettingsStatus("saving");
    saveUserSettings(currentSettings({ ui_scale: nextScale }))
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function onShortcutsEnabledChange(nextEnabled: boolean) {
    setShortcutsEnabled(nextEnabled);
    setSettingsStatus("saving");
    saveUserSettings(
      currentSettings({ shortcuts: { enabled: nextEnabled, bindings: shortcutBindings } }),
    )
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function onSaveShortcutBinding(action: ShortcutAction, binding: string) {
    const nextBindings = { ...shortcutBindings, [action]: binding };
    setShortcutBindings(nextBindings);
    setSettingsStatus("saving");
    saveUserSettings(
      currentSettings({
        shortcuts: { enabled: shortcutsEnabled, bindings: nextBindings },
      }),
    )
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function onResetShortcuts() {
    const next = {
      enabled: DEFAULT_SHORTCUT_SETTINGS.enabled,
      bindings: { ...DEFAULT_SHORTCUT_SETTINGS.bindings },
    };
    setShortcutsEnabled(next.enabled);
    setShortcutBindings(next.bindings);
    setSettingsStatus("saving");
    saveUserSettings(currentSettings({ shortcuts: next }))
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function settingsNoticeText(notice: SettingsLoadNotice): string {
    const messages: Record<SettingsLoadNotice, string> = {
      legacy_settings_detected: "检测到旧版本设置文件，已迁移至 rules.yaml。",
      legacy_settings_corrupt: "检测到旧版本设置文件，但内容无法读取，已使用默认设置。",
      primary_settings_corrupt_recovered_from_backup: "设置文件损坏，已从 rules.yaml.bak 恢复。",
      primary_settings_corrupt_no_usable_backup: "设置文件损坏，且备份文件也无法读取，已使用默认设置。",
      backup_settings_corrupt: "备份文件损坏，当前 rules.yaml 仍可正常使用。",
    };
    return messages[notice];
  }

  // 标题栏按下时显式启动窗口拖动；控制按钮区域已在容器上 stopPropagation。
  function onHeaderMouseDown(event: MouseEvent<HTMLElement>) {
    if (!isTauri()) return;
    if (event.button !== 0) return;
    if (event.detail > 1) return;

    const target = event.target as HTMLElement;
    if (target.closest("[data-window-control]")) return;

    void getCurrentWindow().startDragging().catch((error) => {
      setError(`窗口拖动失败：${String(error)}`);
    });
  }

return (
    <div className="flex h-full min-h-0 flex-col bg-background text-foreground">
      <AppTitleBar
        appName={APP_NAME}
        referenceName={APP_REFERENCE_NAME}
        tauri={isTauri()}
        onMouseDown={onHeaderMouseDown}
        onDoubleClick={onToggleMaximize}
        onMinimize={onMinimize}
        onToggleMaximize={onToggleMaximize}
        onClose={onClose}
      />

      {settingsLoadNotices.length > 0 && (
        <div
          className="border-b border-amber-200 bg-amber-50 px-6 py-2 text-sm text-amber-900 dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-100"
          role={settingsLoadNotices.some((notice) =>
            notice === "primary_settings_corrupt_no_usable_backup" || notice === "legacy_settings_corrupt"
          ) ? "alert" : "status"}
          aria-live="polite"
          data-testid="settings-load-notices"
        >
          {settingsLoadNotices.map(settingsNoticeText).join(" ")}
        </div>
      )}

      {/* 主体：左右双栏，小窗口上下堆叠 */}
      <main
        className="min-h-0 flex-1"
        style={{ zoom: "var(--app-ui-scale, 1)" }}
        data-testid="scaled-app-content"
      >
        <div className="grid h-full min-h-0 gap-4 p-4 lg:grid-cols-2">
        <Card className="flex h-full min-h-0 min-w-0 flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">原始文案</CardTitle>
            <CardDescription className="text-xs">
              输入或粘贴需要规范化的中文文案，结果会实时生成
            </CardDescription>
          </CardHeader>
          <CardContent className="flex min-h-0 flex-1">
            <Textarea
              className="editor-text min-h-0 flex-1 resize-none placeholder:text-muted-foreground/50"
              placeholder="请在这里粘贴或输入文字"
              aria-label="输入文字"
              data-testid="input-textarea"
              value={input}
              onChange={(e) => onInputChange(e.target.value)}
            />
          </CardContent>
        </Card>

        <Card className="flex h-full min-h-0 min-w-0 flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">规范化结果（实时）</CardTitle>
            <CardDescription className="text-xs">
              {error ? (
                <span className="text-destructive" aria-live="assertive">排版出错：{error}</span>
              ) : isFormatting ? (
                <span data-testid="formatting-status" aria-live="polite">正在排版…</span>
              ) : input.length >= LONG_TEXT_THRESHOLD ? (
                <span data-testid="long-text-status" aria-live="polite">
                  文本较长，处理可能需要更长时间
                  {lastFormatDuration !== null && lastFormatDuration >= SLOW_FORMAT_THRESHOLD_MS
                    ? ` · 最近一次耗时 ${lastFormatDuration} ms`
                    : ""}
                </span>
              ) : lastFormatDuration !== null && lastFormatDuration >= SLOW_FORMAT_THRESHOLD_MS ? (
                <span data-testid="format-duration" aria-live="polite">
                  最近一次排版耗时 {lastFormatDuration} ms
                </span>
              ) : (
                "由规则引擎生成"
              )}
            </CardDescription>
          </CardHeader>
          <CardContent className="flex min-h-0 min-w-0 flex-1 flex-col">
            <div className="relative min-h-0 w-full flex-1 overflow-auto rounded-md border bg-background px-3 py-2">
              {!output && !error && (
                <div
                  className="pointer-events-none absolute inset-0 flex items-center justify-center p-6 text-center text-sm text-muted-foreground"
                  data-testid="output-empty-state"
                >
                  输入内容后，这里将实时显示规范化结果
                </div>
              )}
              <pre
                className="editor-text w-full whitespace-pre-wrap wrap-break-word"
                data-testid="output-text"
              >
                {output}
              </pre>
            </div>
          </CardContent>
        </Card>
        </div>
      </main>

      {/* 操作栏 */}
      <footer className="flex items-center gap-2 border-t px-6 py-4">
        <SettingsDialog
          open={settingsOpen}
          onOpenChange={onSettingsOpenChange}
          triggerRef={settingsTriggerRef}
          rules={rules}
          enabled={enabled}
          enabledSet={enabledSet}
          theme={theme}
          font={font}
          editorFontSize={editorFontSize}
          uiScale={uiScale}
          settingsLoadNotices={settingsLoadNotices}
          appVersion={appVersion}
          settingsStatus={settingsStatus}
          settingsError={settingsError}
          settingsPath={settingsPath}
          onToggleRule={onToggleRule}
          onSetAll={onSetAll}
          onResetDefaults={onResetDefaults}
          onThemeChange={onThemeChange}
          onFollowSystemChange={onFollowSystemChange}
          onFontChange={onFontChange}
          onResetFont={onResetFont}
          onEditorFontSizeChange={onEditorFontSizeChange}
          onUiScaleChange={onUiScaleChange}
          shortcutsEnabled={shortcutsEnabled}
          shortcutBindings={shortcutBindings}
          onShortcutsEnabledChange={onShortcutsEnabledChange}
          onSaveShortcutBinding={onSaveShortcutBinding}
          onResetShortcuts={onResetShortcuts}
        />

        <Button variant="outline" size="sm" data-testid="clear-input" onClick={onClear}>
          {cleared ? (
            <Check className="h-4 w-4 text-green-600" />
          ) : (
            <Eraser className="h-4 w-4" />
          )}
          清除输入
        </Button>

        <div className="ml-auto">
          <Button size="sm" data-testid="copy-output" onClick={onCopy} disabled={!output} aria-label="复制结果">
            {copied ? (
              <>
                <Check className="h-4 w-4 text-green-600" />
                已复制
              </>
            ) : (
              <Copy className="h-4 w-4" />
            )}
            复制结果
          </Button>
          <span className="sr-only" aria-live="polite" data-testid="copy-status">
            {copied ? "已复制" : ""}
          </span>
        </div>
      </footer>
    </div>
  );
}