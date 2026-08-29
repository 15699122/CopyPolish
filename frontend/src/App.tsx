import { useEffect, useMemo, useRef, useState } from "react";
import { Check, Copy, Eraser } from "lucide-react";

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
import { useFormatter } from "@/hooks/useFormatter";
import { useSettingsActions } from "@/hooks/useSettingsActions";
import { useSettingsPersistence } from "@/hooks/useSettingsPersistence";
import { useSettingsLoader } from "@/hooks/useSettingsLoader";
import { useShortcuts } from "@/hooks/useShortcuts";
import { useThemeAndFont } from "@/hooks/useThemeAndFont";
import { useWindowControls } from "@/hooks/useWindowControls";
import {
  getEnabledDefaults,
  getRules,
  isTauri,
  type Rule,
  type RuleSelection,
  type SettingsLoadNotice,
  type UserSettings,
} from "@/lib/tauri";

export const APP_NAME = "文案净排";
const APP_REFERENCE_NAME = "CopyPolish";
const NORMAL_DEBOUNCE_MS = 160;
const LONG_TEXT_THRESHOLD = 50_000;
const SLOW_FORMAT_THRESHOLD_MS = 100;

/**
 * 主界面：左输入 / 右输出（小窗口时上下堆叠）+ 操作栏 + 规则设置对话框。
 * 排版由 Tauri 侧 Rust 引擎完成；浏览器预览时走内置演示回退实现。
 */
export default function App() {
  const [input, setInput] = useState("");
  const [rules, setRules] = useState<Rule[]>([]);
  const [copied, setCopied] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const settingsTriggerRef = useRef<HTMLButtonElement | null>(null);

  const getRuleSelection = useMemo(
    () => (selected: string[]): RuleSelection => {
      if (selected.length === 0) return { mode: "none" };
      if (selected.length === rules.length && rules.length > 0) return { mode: "all" };
      return { mode: "only", keys: selected };
    },
    [rules.length],
  );
  const {
    output,
    error,
    isFormatting,
    lastFormatDuration,
    scheduleFormat,
    cancelFormat,
    clearOutput,
    clearError,
    reportError,
  } = useFormatter({ getSelection: getRuleSelection });

  const {
    enabled,
    setEnabled,
    theme,
    setTheme,
    font,
    setFont,
    editorFontSize,
    setEditorFontSize,
    uiScale,
    setUiScale,
    shortcutsEnabled,
    setShortcutsEnabled,
    shortcutBindings,
    setShortcutBindings,
    settingsLoadNotices,
    settingsPath,
    appVersion,
    isHydrated,
    loadSettings,
  } = useSettingsLoader({
    onRestoreInput: (restoredInput, restoredEnabled) => {
      setInput(restoredInput);
      scheduleFormat(restoredInput, restoredEnabled);
    },
    onLoadError: reportError,
  });

  const enabledSet = useMemo(() => new Set(enabled), [enabled]);

  function currentSettings(next: Partial<UserSettings> = {}): UserSettings {
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

  const { settingsStatus, settingsError, persistSettings, schedulePersist } = useSettingsPersistence({
    getSettings: currentSettings,
    isHydrated,
    debounceMs: NORMAL_DEBOUNCE_MS,
  });

  const {
    onToggleRule,
    onSetAll,
    onResetDefaults,
    onThemeChange,
    onFollowSystemChange,
    onFontChange,
    onResetFont,
    onEditorFontSizeChange,
    onUiScaleChange,
    onShortcutsEnabledChange,
    onSaveShortcutBinding,
    onResetShortcuts,
  } = useSettingsActions({
    rules,
    enabled,
    enabledSet,
    input,
    setEnabled,
    setTheme,
    setFont,
    setEditorFontSize,
    setUiScale,
    setShortcutsEnabled,
    setShortcutBindings,
    shortcutsEnabled,
    shortcutBindings,
    scheduleFormat,
    persistSettings,
  });

  // 初始化：加载规则与默认启用集；随后由 useSettingsLoader 恢复用户设置。
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
        await loadSettings(ruleList, defaults);
      } catch (e) {
        if (!cancelled) reportError(e);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

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

  useThemeAndFont({ theme, font, editorFontSize, uiScale });

  const { onMinimize, onToggleMaximize, onClose, onHeaderMouseDown } = useWindowControls({
    onError: reportError,
  });

  function onInputChange(value: string) {
    setInput(value);
    scheduleFormat(value, enabled);
    schedulePersist({ enabled, last_input: value });
  }

  async function onCopy() {
    if (!output) return;
    try {
      await navigator.clipboard.writeText(output);
    } catch (e) {
      reportError(e);
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
    clearOutput();
    cancelFormat();
    clearError();
    setCleared(true);
    window.setTimeout(() => setCleared(false), 1200);
    persistSettings({ enabled, last_input: "" });
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
        <div className="grid h-full min-h-0 grid-rows-2 gap-4 p-4 lg:grid-cols-2 lg:grid-rows-1">
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
            <div
              className="relative min-h-0 w-full flex-1 overflow-auto rounded-md border bg-background px-3 py-2"
              data-testid="output-scroller"
            >
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