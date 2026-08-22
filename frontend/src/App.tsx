import { useEffect, useMemo, useRef, useState } from "react";
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
import { ScrollArea } from "@/components/ui/scroll-area";
import { Textarea } from "@/components/ui/textarea";
import {
  formatText,
  getEnabledDefaults,
  getRules,
  getUserSettings,
  isTauri,
  saveUserSettings,
  type Rule,
  type UserSettings,
} from "@/lib/tauri";

const APP_NAME = "文案净排";
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

  function persistSettings(nextEnabled: string[], nextInput: string) {
    if (!hydratedRef.current) return;
    saveUserSettings({ enabled: nextEnabled, last_input: nextInput }).catch(() => {
      // 持久化失败不打断排版主流程。
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

        if (saved && Array.isArray(saved.enabled)) {
          const restoredEnabled = saved.enabled.filter((k) =>
            ruleList.some((r) => r.key === k),
          );
          setEnabled(restoredEnabled);
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

  async function onMinimize() {
    if (isTauri()) await getCurrentWindow().minimize();
  }

  async function onToggleMaximize() {
    if (isTauri()) await getCurrentWindow().toggleMaximize();
  }

  async function onClose() {
    if (isTauri()) await getCurrentWindow().close();
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
    <div className="flex min-h-svh flex-col bg-background text-foreground">
      {/* 无边框窗口标题栏：标题区可拖动，右侧提供窗口控制。 */}
      <header
        className="flex select-none items-center justify-between border-b px-6 py-3"
        data-tauri-drag-region
        onDoubleClick={onToggleMaximize}
      >
        <div data-tauri-drag-region>
          <h1 className="text-xl font-bold leading-tight">{APP_NAME}</h1>
          <p className="text-xs text-muted-foreground">
            {isTauri()
              ? `实时保护 LaTeX / Markdown 结构 · ${APP_REFERENCE_NAME}`
              : "浏览器预览模式 · 内置回退排版"}
          </p>
        </div>
        {isTauri() && (
          <div className="flex items-center gap-1" data-tauri-drag-region="false">
            <Button variant="ghost" size="icon" onClick={onMinimize} aria-label="最小化">
              <Minus className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" onClick={onToggleMaximize} aria-label="最大化或还原">
              <Maximize2 className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" onClick={onClose} aria-label="关闭">
              <X className="h-4 w-4" />
            </Button>
          </div>
        )}
      </header>

      {/* 主体：左右双栏，小窗口上下堆叠 */}
      <main className="grid min-h-0 flex-1 gap-4 p-4 md:grid-cols-2">
        <Card className="flex min-h-0 flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">输入文字</CardTitle>
            <CardDescription className="text-xs">粘贴后自动排版</CardDescription>
          </CardHeader>
          <CardContent className="flex min-h-0 flex-1">
            <Textarea
              className="min-h-0 flex-1 resize-none"
              placeholder="在LeanCloud上，花了5000元"
              aria-label="输入文字"
              data-testid="input-textarea"
              value={input}
              onChange={(e) => onInputChange(e.target.value)}
            />
          </CardContent>
        </Card>

        <Card className="flex min-h-0 flex-col">
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
          <CardContent className="flex min-h-0 flex-1">
            <ScrollArea className="h-full w-full">
              <pre
                className="min-h-full whitespace-pre-wrap break-words font-sans text-sm"
                data-testid="output-text"
              >
                {output}
              </pre>
            </ScrollArea>
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
          <DialogContent className="max-h-[80vh] max-w-md">
            <DialogHeader>
              <DialogTitle>设置 — 排版规则</DialogTitle>
              <DialogDescription>
                逐条启用/停用规则。已启用 {enabled.length}/{rules.length} 条
              </DialogDescription>
            </DialogHeader>

            <ScrollArea className="h-[50vh] pr-4">
              <div className="space-y-4">
                {groups.map(([section, items]) => (
                  <div key={section}>
                    <h3 className="mb-2 text-sm font-semibold">{section}</h3>
                    <div className="space-y-2">
                      {items.map((r) => (
                        <div key={r.key} className="flex items-start gap-3">
                          <Checkbox
                            id={`rule-${r.key}`}
                            checked={enabledSet.has(r.key)}
                            onCheckedChange={() => onToggleRule(r.key)}
                            data-testid={`rule-${r.key}`}
                            aria-label={r.name}
                          />
                          <Label
                            htmlFor={`rule-${r.key}`}
                            className="text-sm leading-snug"
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
                  </div>
                ))}
              </div>
            </ScrollArea>

            <DialogFooter className="flex items-center justify-between gap-2">
              <div className="flex gap-2">
                <Button variant="outline" size="sm" data-testid="select-all" onClick={() => onSetAll(true)}>
                  全选
                </Button>
                <Button variant="outline" size="sm" data-testid="select-none" onClick={() => onSetAll(false)}>
                  全不选
                </Button>
                <Button variant="outline" size="sm" data-testid="reset-defaults" onClick={onResetDefaults}>
                  恢复默认
                </Button>
              </div>
              <Button size="sm" data-testid="settings-done" onClick={() => setSettingsOpen(false)}>
                完成
              </Button>
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