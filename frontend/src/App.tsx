import { useState } from "react";
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
import { HelpDialog } from "@/components/HelpDialog";
import { useAppController } from "@/hooks/useAppController";
import { useFirstRunNotice } from "@/hooks/useFirstRunNotice";
import { isSettingsLoadNoticeAlert, settingsLoadNoticeText } from "@/lib/settingsLoadNotices";

export const APP_NAME = "文案净排";
const APP_REFERENCE_NAME = "CopyPolish";
const LONG_TEXT_THRESHOLD = 50_000;
const SLOW_FORMAT_THRESHOLD_MS = 100;

/**
 * 主界面：左输入 / 右输出（小窗口时上下堆叠）+ 操作栏 + 规则设置对话框。
 * 排版由 Tauri 侧 Rust 引擎完成；浏览器预览时走内置演示回退实现。
 */
export default function App() {
  const [helpOpen, setHelpOpen] = useState(false);
  const firstRunNotice = useFirstRunNotice();
  const {
    isDemoMode,
    output,
    error,
    isFormatting,
    lastFormatDuration,
    input,
    onInputChange,
    onFormatNow,
    outputMode,
    layoutMode,
    onOutputModeChange,
    onLayoutModeChange,
    copied,
    copyOutput,
    copyAndClear,
    cleared,
    onClear,
    settingsDialogProps,
    onMinimize,
    onToggleMaximize,
    onClose,
    onHeaderMouseDown,
  } = useAppController();
  const manualMode = outputMode === "manual";
  const inputChars = Array.from(input).length;
  const outputChars = Array.from(output).length;
  const resultTitle = manualMode ? "排版结果（手动）" : "排版结果（实时）";

  return (
    <div className="flex h-full min-h-0 flex-col bg-background text-foreground">
      <AppTitleBar
        appName={APP_NAME}
        referenceName={APP_REFERENCE_NAME}
        tauri={!isDemoMode}
        onMouseDown={onHeaderMouseDown}
        onDoubleClick={onToggleMaximize}
        onMinimize={onMinimize}
        onToggleMaximize={onToggleMaximize}
        onClose={onClose}
      />

      {settingsDialogProps.settingsLoadNotices.length > 0 && (
        <div
          className="border-b border-amber-200 bg-amber-50 px-6 py-2 text-sm text-amber-900 dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-100"
          role={settingsDialogProps.settingsLoadNotices.some(isSettingsLoadNoticeAlert) ? "alert" : "status"}
          aria-live="polite"
          data-testid="settings-load-notices"
        >
          {settingsDialogProps.settingsLoadNotices.map(settingsLoadNoticeText).join(" ")}
        </div>
      )}

      {isDemoMode && (
        <div
          className="border-b border-sky-200 bg-sky-50 px-6 py-2 text-sm text-sky-900 dark:border-sky-900/60 dark:bg-sky-950/40 dark:text-sky-100"
          role="status"
          data-testid="demo-mode-banner"
        >
          演示模式：当前运行在浏览器预览中，排版结果使用最小化回退实现，不代表桌面版 Rust 引擎的完整行为。
        </div>
      )}

      {firstRunNotice.visible && (
        <div
          className="flex items-start gap-3 border-b border-amber-200 bg-amber-50 px-6 py-3 text-sm text-amber-950 dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-100"
          role="status"
          aria-live="polite"
          data-testid="first-run-notice"
        >
          <span className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full border border-current text-[10px] font-semibold" aria-hidden="true">
            ?
          </span>
          <div className="min-w-0 flex-1">
            <p className="font-medium">第一次使用？先了解规则风险和演示模式边界。</p>
            <p className="mt-0.5 text-xs text-amber-900/75 dark:text-amber-100/75">
              输出适合复核，不替代人工检查；高风险清洗规则请谨慎启用。
            </p>
          </div>
          <Button
            variant="ghost"
            size="sm"
            data-testid="first-run-help"
            onClick={() => {
              firstRunNotice.dismiss();
              setHelpOpen(true);
            }}
          >
            查看说明
          </Button>
          <Button variant="ghost" size="sm" data-testid="first-run-dismiss" onClick={firstRunNotice.dismiss}>
            知道了
          </Button>
        </div>
      )}

      {/* 操作模式与布局：紧凑工具条，内容区保持主导 */}
      <div
        className="flex flex-wrap items-center gap-x-5 gap-y-2 border-b px-5 py-2"
        data-testid="workspace-toolbar"
      >
        <label className="prose-helper flex items-center gap-2 text-muted-foreground">
          <span className="shrink-0 font-medium">输出模式</span>
          <select
            value={outputMode}
            onChange={(event) => onOutputModeChange(event.target.value as typeof outputMode)}
            data-testid="output-mode-toolbar"
            aria-label="输出模式"
            className="h-8 rounded-md border border-input bg-background px-2 text-sm text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="realtime">实时输出</option>
            <option value="manual">手动输出</option>
          </select>
        </label>
        <label className="prose-helper flex items-center gap-2 text-muted-foreground">
          <span className="shrink-0 font-medium">编辑区布局</span>
          <select
            value={layoutMode}
            onChange={(event) => onLayoutModeChange(event.target.value as typeof layoutMode)}
            data-testid="layout-mode-toolbar"
            aria-label="输入输出布局"
            className="h-8 rounded-md border border-input bg-background px-2 text-sm text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="auto">自动布局</option>
            <option value="horizontal">左右对照</option>
            <option value="vertical">上下排列</option>
          </select>
        </label>
        <p className="prose-helper min-w-40 flex-1 text-muted-foreground">
          {manualMode ? "手动模式不会自动刷新，请点击结果区“立即排版”。" : "实时模式会在输入或设置变化后自动排版。"}
        </p>
      </div>

      {/* 主体：左右双栏，小窗口上下堆叠 */}
      <main
        className="min-h-0 flex-1"
        style={{ zoom: "var(--app-ui-scale, 1)" }}
        data-testid="scaled-app-content"
      >
        <div
          className={[
            "grid h-full min-h-0 gap-4 p-4",
            settingsDialogProps.layoutMode === "vertical"
              ? "grid-rows-2"
              : settingsDialogProps.layoutMode === "horizontal"
                ? "grid-cols-2"
                : "grid-rows-2 lg:grid-cols-2 lg:grid-rows-1",
          ].join(" ")}
          data-testid="editor-layout"
        >
        <Card className="flex h-full min-h-0 min-w-0 flex-col shadow-none">
          <CardHeader className="flex flex-row items-start justify-between gap-3 space-y-0 px-5 pb-2 pt-4">
            <div className="min-w-0 space-y-1">
              <CardTitle className="prose-title">原始文本</CardTitle>
              <CardDescription className="prose-helper">
                输入或粘贴中文原文，{manualMode ? "手动模式下需要主动触发排版" : "排版结果会跟随输入自动更新"}
              </CardDescription>
            </div>
            <Button
              variant="ghost"
              size="sm"
              data-testid="clear-input"
              onClick={onClear}
              className="shrink-0"
            >
              {cleared ? (
                <Check className="h-4 w-4 text-green-600" />
              ) : (
                <Eraser className="h-4 w-4" />
              )}
              清空输入
            </Button>
          </CardHeader>
          <CardContent className="flex min-h-0 flex-1 px-5">
            <Textarea
              className="editor-text min-h-0 flex-1 resize-none placeholder:text-muted-foreground/50"
              placeholder="请在这里粘贴或输入文字"
              aria-label="输入文字"
              data-testid="input-textarea"
              value={input}
              onChange={(e) => onInputChange(e.target.value)}
            />
          </CardContent>
          <div className="prose-stats px-5 pb-3 pt-2 text-muted-foreground" data-testid="input-stats">
            输入：{inputChars} 字符
          </div>
        </Card>

        <Card className="flex h-full min-h-0 min-w-0 flex-col shadow-none">
          <CardHeader className="flex flex-row items-start justify-between gap-3 space-y-0 px-5 pb-2 pt-4">
            <div className="min-w-0 space-y-1">
              <CardTitle className="prose-title">{resultTitle}</CardTitle>
              <CardDescription className="prose-helper">
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
                ) : manualMode ? (
                  "手动模式需要主动触发排版"
                ) : (
                  "由规则引擎生成"
                )}
              </CardDescription>
            </div>
            {manualMode && (
              <Button
                variant="outline"
                size="sm"
                data-testid="format-now"
                onClick={onFormatNow}
                disabled={!input}
                className="shrink-0"
              >
                立即排版
              </Button>
            )}
          </CardHeader>
          <CardContent className="flex min-h-0 min-w-0 flex-1 flex-col px-5">
            <div
              className="relative min-h-0 w-full flex-1 overflow-auto rounded-md border border-input bg-background px-3 py-2 shadow-sm"
              data-testid="output-scroller"
            >
              {!output && !error && (
                <div
                  className="prose-helper pointer-events-none absolute inset-0 flex items-center justify-center p-6 text-center text-muted-foreground"
                  data-testid="output-empty-state"
                >
                  {manualMode ? "输入内容后，点击右上角“立即排版”生成结果" : "输入内容后，这里会自动显示排版结果"}
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
          <div className="prose-stats px-5 pb-3 pt-2 text-muted-foreground" data-testid="output-stats">
            输出：{outputChars} 字符
          </div>
        </Card>
        </div>
      </main>

      {/* 操作栏：设置与说明在左，复制动作在右并保留原语义 */}
      <footer className="flex flex-wrap items-center gap-2 border-t px-5 py-3">
        <SettingsDialog {...settingsDialogProps} />
        <HelpDialog open={helpOpen} onOpenChange={setHelpOpen} />

        <div className="ml-auto flex flex-wrap items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            data-testid="copy-and-clear"
            onClick={copyAndClear}
            disabled={!output}
            aria-label="复制并清空"
          >
            <Copy className="h-4 w-4" />
            复制并清空
          </Button>
          <Button size="sm" data-testid="copy-output" onClick={copyOutput} disabled={!output}>
            {copied ? (
              <>
                <Check className="h-4 w-4" />
                已复制
              </>
            ) : (
              <>
                <Copy className="h-4 w-4" />
                复制结果
              </>
            )}
          </Button>
          <span className="sr-only" aria-live="polite" data-testid="copy-status">
            {copied ? "已复制" : ""}
          </span>
        </div>
      </footer>
    </div>
  );
}