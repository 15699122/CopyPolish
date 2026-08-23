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
import {
  formatText,
  getAppVersion,
  getEnabledDefaults,
  getRules,
  getSettingsPath,
  getUserSettings,
  isTauri,
  saveUserSettings,
  type Rule,
  type RuleSelection,
  type FontFamily,
  type LoadedUserSettings,
  type ThemeMode,
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
  const [settingsPath, setSettingsPath] = useState<string | null>(null);
  const [appVersion, setAppVersion] = useState(__APP_VERSION__);

  // 主题状态：system / light / dark。
  const [theme, setTheme] = useState<ThemeMode>("system");
  const [font, setFont] = useState<FontFamily>("system");

  function persistSettings(nextEnabled: string[], nextInput: string) {
    if (!hydratedRef.current) return;
    setSettingsStatus("saving");
    saveUserSettings({ enabled: nextEnabled, last_input: nextInput, theme, font })
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
          if (saved.recovered_from_backup) {
            setSettingsStatus("error");
            setSettingsError("主设置文件损坏，已从 rules.yaml.bak 恢复；下次保存后会生成新的主文件。");
          }
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
    function onKeyDown(event: KeyboardEvent) {
      const modifier = event.ctrlKey || event.metaKey;
      if (!modifier || event.altKey) return;

      if (event.key === "Enter") {
        event.preventDefault();
        scheduleFormat(input, enabled, 0);
      } else if (event.shiftKey && event.key.toLowerCase() === "c") {
        event.preventDefault();
        void onCopy();
      } else if (!event.shiftKey && event.key === ",") {
        event.preventDefault();
        setSettingsOpen(true);
      }
    }

    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [enabled, input, output]);

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
    saveUserSettings({ enabled, last_input: input, theme: nextTheme, font })
      .then(() => {
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((e) => {
        setSettingsStatus("error");
        setSettingsError(String(e));
      });
  }

  function onFontChange(nextFont: FontFamily) {
    setFont(nextFont);
    setSettingsStatus("saving");
    saveUserSettings({ enabled, last_input: input, theme, font: nextFont })
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

      {/* 主体：左右双栏，小窗口上下堆叠 */}
      <main className="grid min-h-0 flex-1 gap-4 p-4 lg:grid-cols-2">
        <Card className="flex min-h-0 flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">原始文案</CardTitle>
            <CardDescription className="text-xs">
              输入或粘贴需要规范化的中文文案，结果会实时生成
            </CardDescription>
          </CardHeader>
          <CardContent className="flex min-h-0 flex-1">
            <Textarea
              className="min-h-0 flex-1 resize-none placeholder:text-sm placeholder:text-muted-foreground/50"
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
            <div className="relative min-h-0 w-full flex-1 overflow-auto rounded-md border bg-background p-3">
              {!output && !error && (
                <div
                  className="pointer-events-none absolute inset-0 flex items-center justify-center p-6 text-center text-sm text-muted-foreground"
                  data-testid="output-empty-state"
                >
                  输入内容后，这里将实时显示规范化结果
                </div>
              )}
              <pre
                className="w-full whitespace-pre-wrap wrap-break-word text-sm"
                data-testid="output-text"
              >
                {output}
              </pre>
            </div>
          </CardContent>
        </Card>
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
          appVersion={appVersion}
          settingsStatus={settingsStatus}
          settingsError={settingsError}
          settingsPath={settingsPath}
          onToggleRule={onToggleRule}
          onSetAll={onSetAll}
          onResetDefaults={onResetDefaults}
          onThemeChange={onThemeChange}
          onFontChange={onFontChange}
          onResetFont={onResetFont}
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