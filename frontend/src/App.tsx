import { useEffect, useMemo, useRef, useState, type MouseEvent } from "react";
import { Check, Copy, Eraser, Maximize2, Minus, Settings, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";
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
  type FontFamily,
  type ThemeMode,
  type UserSettings,
} from "@/lib/tauri";

export const APP_NAME = "文案净排";
const APP_REFERENCE_NAME = "CopyPolish";
const DEBOUNCE_MS = 160;

/**
 * 主界面：左输入 / 右输出（小窗口时上下堆叠）+ 操作栏 + 规则设置对话框。
 * 排版由 Tauri 侧 Python 引擎完成；浏览器预览时走内置回退实现。
 */
export default function App() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [rules, setRules] = useState<Rule[]>([]);
  const [enabled, setEnabled] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);

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

  const fontFamilyStack: Record<FontFamily, string> = {
    system: "system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif",
    "microsoft-yahei": "\"Microsoft YaHei\", \"微软雅黑\", system-ui, sans-serif",
    pingfang: "\"PingFang SC\", \"苹方\", system-ui, sans-serif",
    "noto-sans-cjk": "\"Noto Sans CJK SC\", \"Source Han Sans SC\", system-ui, sans-serif",
    simsun: "SimSun, \"宋体\", serif",
    simhei: "SimHei, \"黑体\", system-ui, sans-serif",
  };

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
    }, DEBOUNCE_MS);
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

        let saved: UserSettings | null = null;
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
    document.documentElement.style.setProperty("--app-font-family", fontFamilyStack[font]);
  }, [font]);

  // 实时排版（防抖 + 忽略乱序的旧请求）
  function scheduleFormat(nextInput: string, applyEnabled = enabled) {
    if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
    debounceRef.current = window.setTimeout(async () => {
      const seq = ++seqRef.current;
      try {
        const result = await formatText({ text: nextInput, enabled: applyEnabled });
        if (seqRef.current === seq) setOutput(result);
      } catch (e) {
        if (seqRef.current === seq) setError(String(e));
      } finally {
        if (seqRef.current === seq) setError((prev) => (prev ? prev : null));
      }
    }, DEBOUNCE_MS);
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

  // 按 section 分组规则
  const groups = useMemo(() => {
    const map = new Map<string, Rule[]>();
    for (const r of rules) {
      const list = map.get(r.section) ?? [];
      list.push(r);
      map.set(r.section, list);
    }
    return Array.from(map.entries());
  }, [rules]);
return (
    <div className="flex h-full min-h-0 flex-col bg-background text-foreground">
      {/* 无边框窗口标题栏：按下空白处显式调用 startDragging；右侧为窗口控制。 */}
      <header
        className="flex select-none items-center justify-between border-b px-6 py-3"
        data-testid="title-bar"
        onMouseDown={onHeaderMouseDown}
        onDoubleClick={onToggleMaximize}
      >
        <div className="min-w-0 space-y-1.5">
          <h1 className="text-xl font-bold leading-none">{APP_NAME}</h1>
          <p className="text-xs leading-relaxed text-muted-foreground">
            {isTauri()
              ? `实时保护 LaTeX / Markdown 结构 · ${APP_REFERENCE_NAME}`
              : "浏览器预览模式 · 内置回退排版"}
          </p>
        </div>
        {isTauri() && (
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
                <span className="text-destructive">排版出错：{error}</span>
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
        <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
          <DialogTrigger asChild>
            <Button variant="outline" size="sm" data-testid="open-settings" aria-label="打开设置">
              <Settings className="h-4 w-4" />
              设置
            </Button>
          </DialogTrigger>
          <DialogContent
            data-testid="settings-dialog"
            className="flex h-[min(680px,calc(100vh-2rem))] w-[min(560px,calc(100vw-2rem))] max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)] flex-col gap-0 overflow-hidden p-0 sm:min-h-[520px] sm:min-w-[480px]"
          >
            {/* 固定标题区 */}
            <DialogHeader className="shrink-0 border-b px-6 py-5 pr-12">
              <DialogTitle>设置 — 排版规则</DialogTitle>
              <DialogDescription>
                逐条启用/停用规则。已启用 {enabled.length}/{rules.length} 条
              </DialogDescription>
            </DialogHeader>

            {/* 规则与主题滚动区：仅此区域滚动 */}
            <div
              className="min-h-0 flex-1 overflow-y-auto px-6 py-4"
              data-testid="settings-scroll-area"
            >
              <div className="space-y-6 pb-4">
                {/* 主题设置 */}
                <div className="space-y-2">
                  <h3 className="text-sm font-semibold">主题</h3>
                  <div
                    className="grid grid-cols-1 gap-1 sm:grid-cols-3"
                    data-testid="theme-options"
                  >
                    <label
                      className={cn(
                        "flex min-w-0 cursor-pointer items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors hover:bg-accent",
                        theme === "system" && "bg-accent text-accent-foreground",
                      )}
                    >
                      <input
                        type="radio"
                        name="theme"
                        value="system"
                        checked={theme === "system"}
                        onChange={() => onThemeChange("system")}
                        data-testid="theme-system"
                        className="h-4 w-4 shrink-0"
                      />
                      <span className="truncate text-sm">跟随系统</span>
                    </label>
                    <label
                      className={cn(
                        "flex min-w-0 cursor-pointer items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors hover:bg-accent",
                        theme === "light" && "bg-accent text-accent-foreground",
                      )}
                    >
                      <input
                        type="radio"
                        name="theme"
                        value="light"
                        checked={theme === "light"}
                        onChange={() => onThemeChange("light")}
                        data-testid="theme-light"
                        className="h-4 w-4 shrink-0"
                      />
                      <span className="truncate text-sm">浅色</span>
                    </label>
                    <label
                      className={cn(
                        "flex min-w-0 cursor-pointer items-center gap-1.5 rounded-md px-2 py-1.5 transition-colors hover:bg-accent",
                        theme === "dark" && "bg-accent text-accent-foreground",
                      )}
                    >
                      <input
                        type="radio"
                        name="theme"
                        value="dark"
                        checked={theme === "dark"}
                        onChange={() => onThemeChange("dark")}
                        data-testid="theme-dark"
                        className="h-4 w-4 shrink-0"
                      />
                      <span className="truncate text-sm">深色</span>
                    </label>
                  </div>
                </div>

                <div className="space-y-2" data-testid="font-settings">
                  <div>
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
                </div>

                {groups.map(([section, items]) => (
                  <section key={section}>
                    <h3 className="mb-2 text-sm font-semibold">{section}</h3>
                    <div className="space-y-2">
                      {items.map((r) => (
                        <div key={r.key} className="flex items-start gap-3 rounded-md border p-3">
                          <Checkbox
                            id={`rule-${r.key}`}
                            checked={enabledSet.has(r.key)}
                            onCheckedChange={() => onToggleRule(r.key)}
                            data-testid={`rule-${r.key}`}
                            aria-label={r.name}
                          />
                          <Label
                            htmlFor={`rule-${r.key}`}
                            className="min-w-0 flex-1 text-sm leading-5"
                          >
                            {r.name}
                            {r.disputed && (
                              <span className="ml-1 text-xs text-muted-foreground">
                                （争议，默认关闭）
                              </span>
                            )}
                          </Label>
                        </div>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
            </div>

            {/* 紧凑底部操作区：桌面端单行对齐；小屏或错误信息较长时自动换行。 */}
            <DialogFooter
              className="shrink-0 border-t px-4 py-4 sm:px-6"
              data-testid="settings-footer"
            >
              <div className="flex w-full min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                <div
                  className="min-w-0 flex-1 text-left text-xs leading-5"
                  data-testid="settings-file-info"
                >
                  <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0">
                    <span className="shrink-0 text-muted-foreground" data-testid="settings-version">版本 {appVersion}</span>
                    {settingsStatus === "saving" && (
                      <span className="shrink-0 text-muted-foreground" data-testid="settings-status">正在保存…</span>
                    )}
                    {settingsStatus === "saved" && (
                      <span className="shrink-0 text-green-600" data-testid="settings-status">设置已保存</span>
                    )}
                    {settingsStatus === "error" && (
                      <span className="break-all text-destructive" data-testid="settings-status">
                        设置保存失败：{settingsError}
                      </span>
                    )}
                    {settingsPath && (
                      <span
                        className="group relative min-w-0 truncate text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        tabIndex={0}
                        title={settingsPath}
                        aria-label={`设置文件完整路径：${settingsPath}`}
                        data-testid="settings-path"
                      >
                        设置文件：{settingsPath}
                        {/* 悬停或键盘聚焦时展示完整路径，不受界面宽度截断影响。 */}
                        <span
                          role="tooltip"
                          className="pointer-events-none absolute bottom-full left-0 z-10 mb-1 hidden w-max max-w-[28rem] break-all rounded-md border bg-card px-2 py-1 text-left text-card-foreground shadow-md group-focus-within:block group-hover:block"
                        >
                          {settingsPath}
                        </span>
                      </span>
                    )}
                  </div>
                </div>

                <div className="flex shrink-0 flex-wrap items-center justify-end gap-2" data-testid="settings-actions">
                  <Button variant="outline" size="sm" data-testid="select-all" onClick={() => onSetAll(true)}>
                    全选
                  </Button>
                  <Button variant="outline" size="sm" data-testid="select-none" onClick={() => onSetAll(false)}>
                    全不选
                  </Button>
                  <Button variant="secondary" size="sm" data-testid="reset-defaults" onClick={onResetDefaults}>
                    恢复默认
                  </Button>
                  <Button size="sm" data-testid="settings-done" onClick={() => setSettingsOpen(false)}>
                    完成
                  </Button>
                </div>
              </div>
            </DialogFooter>
          </DialogContent>
        </Dialog>

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
        </div>
      </footer>
    </div>
  );
}