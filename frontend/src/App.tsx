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
          <div className="px-6 pb-3 text-xs text-muted-foreground" data-testid="input-stats">
            输入：{Array.from(input).length} 字符
          </div>
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
              className="relative min-h-0 w-full flex-1 overflow-auto rounded-md border border-input bg-background px-3 py-2 shadow-sm"
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
          <div className="px-6 pb-3 text-xs text-muted-foreground" data-testid="output-stats">
            输出：{Array.from(output).length} 字符
          </div>
        </Card>
        </div>
      </main>

      {/* 操作栏 */}
      <footer className="flex items-center gap-2 border-t px-6 py-4">
        <SettingsDialog {...settingsDialogProps} />
        <HelpDialog open={helpOpen} onOpenChange={setHelpOpen} />

        <Button variant="outline" size="sm" data-testid="clear-input" onClick={onClear}>
          {cleared ? (
            <Check className="h-4 w-4 text-green-600" />
          ) : (
            <Eraser className="h-4 w-4" />
          )}
          清除输入
        </Button>

        <div className="ml-auto">
          <Button size="sm" data-testid="copy-output" onClick={copyOutput} disabled={!output} aria-label="复制结果">
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
          <span className="sr-only" aria-live="polite" data-testid="copy-status">
            {copied ? "已复制" : ""}
          </span>
        </div>
      </footer>
    </div>
  );
}