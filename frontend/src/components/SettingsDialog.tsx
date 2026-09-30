import { useState, type RefObject } from "react";
import { Settings } from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ThemeSection } from "@/components/settings/ThemeSection";
import { DisplaySection } from "@/components/settings/DisplaySection";
import { ShortcutsSection } from "@/components/settings/ShortcutsSection";
import { RulesSection } from "@/components/settings/RulesSection";
import { ReplacementsSection } from "@/components/settings/ReplacementsSection";
import { PresetsSection } from "@/components/settings/PresetsSection";
import { OutputSection } from "@/components/settings/OutputSection";
import { PrivacySection } from "@/components/settings/PrivacySection";
import { SettingsFooter, type SettingsStatus } from "@/components/settings/SettingsFooter";
import { cn } from "@/lib/utils";
import type {
  EditorFontSize,
  FontFamily,
  CharacterConversion,
  BuildCapabilities,
  ReplacementPair,
  Preset,
  Rule,
  SettingsLoadNotice,
  ShortcutAction,
  ShortcutBindings,
  ThemeMode,
  UiScale,
  OutputMode,
  LayoutMode,
} from "@/lib/tauri";

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  triggerRef: RefObject<HTMLButtonElement | null>;
  rules: Rule[];
  enabled: string[];
  enabledSet: Set<string>;
  theme: ThemeMode;
  font: FontFamily;
  editorFontSize: EditorFontSize;
  uiScale: UiScale;
  replacements: ReplacementPair[];
  conversion: CharacterConversion;
  buildCapabilities: BuildCapabilities;
  settingsLoadNotices: SettingsLoadNotice[];
  appVersion: string;
  settingsStatus: SettingsStatus;
  settingsError: string | null;
  settingsPath: string | null;
  onToggleRule: (key: string) => void;
  onSetAll: (on: boolean) => void;
  onResetDefaults: () => void;
  onThemeChange: (theme: ThemeMode) => void;
  onFollowSystemChange: (follow: boolean) => void;
  onFontChange: (font: FontFamily) => void;
  onResetFont: () => void;
  onEditorFontSizeChange: (size: EditorFontSize) => void;
  onUiScaleChange: (scale: UiScale) => void;
  shortcutsEnabled: boolean;
  shortcutBindings: ShortcutBindings;
  onShortcutsEnabledChange: (enabled: boolean) => void;
  onSaveShortcutBinding: (action: ShortcutAction, binding: string) => void;
  onResetShortcuts: () => void;
  onReplacementsChange: (replacements: ReplacementPair[]) => void;
  onConversionChange: (conversion: CharacterConversion) => void;
  restoreLastInput: boolean;
  onRestoreLastInputChange: (enabled: boolean) => void;
  onClearSavedInput: () => void;
  presets: Preset[];
  onApplyPreset: (preset: Preset) => void;
  outputMode: OutputMode;
  layoutMode: LayoutMode;
  onOutputModeChange: (mode: OutputMode) => void;
  onLayoutModeChange: (mode: LayoutMode) => void;
}

/** 设置弹窗编排容器；各分区与状态/持久化行为由 App 注入。 */
type SettingsCategoryId = "rules" | "transform" | "output" | "appearance" | "shortcuts" | "privacy";

const SETTINGS_CATEGORIES: { id: SettingsCategoryId; title: string; description: string }[] = [
  { id: "rules", title: "排版规则", description: "先选工作流预设，再逐条微调规则" },
  { id: "transform", title: "替换与转换", description: "按顺序管理字面量替换与简繁转换" },
  { id: "output", title: "编辑与输出", description: "选择输出刷新方式与输入输出布局" },
  { id: "appearance", title: "外观显示", description: "切换主题、字体、字号与界面缩放" },
  { id: "shortcuts", title: "快捷键", description: "启用、修改或恢复默认组合键" },
  { id: "privacy", title: "隐私与存储", description: "控制是否保存上次输入正文" },
];

export function SettingsDialog({
  open,
  onOpenChange,
  triggerRef,
  rules,
  enabled,
  enabledSet,
  theme,
  font,
  editorFontSize,
  uiScale,
  replacements,
  conversion,
  buildCapabilities,
  settingsLoadNotices,
  appVersion,
  settingsStatus,
  settingsError,
  settingsPath,
  onToggleRule,
  onSetAll,
  onResetDefaults,
  onThemeChange,
  onFollowSystemChange,
  onFontChange,
  onResetFont,
  onEditorFontSizeChange,
  onUiScaleChange,
  shortcutsEnabled,
  shortcutBindings,
  onShortcutsEnabledChange,
  onSaveShortcutBinding,
  onResetShortcuts,
  onReplacementsChange,
  onConversionChange,
  restoreLastInput,
  onRestoreLastInputChange,
  onClearSavedInput,
  presets,
  onApplyPreset,
  outputMode,
  layoutMode,
  onOutputModeChange,
  onLayoutModeChange,
}: SettingsDialogProps) {
  const [activeCategory, setActiveCategory] = useState<SettingsCategoryId>("rules");
  const activeCategoryMeta = SETTINGS_CATEGORIES.find((category) => category.id === activeCategory)
    ?? SETTINGS_CATEGORIES[0];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button ref={triggerRef} variant="outline" size="sm" data-testid="open-settings" aria-label="打开设置">
          <Settings className="h-4 w-4" />
          设置
        </Button>
      </DialogTrigger>
      <DialogContent
        data-testid="settings-dialog"
        className="flex h-[min(720px,calc(100vh-2rem))] w-[min(760px,calc(100vw-2rem))] max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)] flex-col gap-0 overflow-hidden p-0 sm:min-h-130 sm:min-w-150"
      >
        <DialogHeader className="shrink-0 border-b px-6 py-4 pr-12">
          <DialogTitle>设置</DialogTitle>
          <DialogDescription>
            {activeCategoryMeta.title}：{activeCategoryMeta.description}
          </DialogDescription>
        </DialogHeader>

        <div
          className="flex min-h-0 flex-1 flex-col sm:flex-row"
          data-testid="settings-category-layout"
        >
          <nav
            aria-label="设置分类"
            data-testid="settings-category-nav"
            className="flex shrink-0 gap-1 overflow-x-auto border-b px-3 py-2 sm:w-44 sm:flex-col sm:overflow-y-auto sm:border-b-0 sm:border-r sm:px-2 sm:py-3"
          >
            {SETTINGS_CATEGORIES.map((category) => (
              <button
                key={category.id}
                type="button"
                onClick={() => setActiveCategory(category.id)}
                data-testid={`settings-category-${category.id}`}
                aria-current={activeCategory === category.id ? "true" : undefined}
                className={cn(
                  "shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-left text-sm outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring sm:whitespace-normal",
                  activeCategory === category.id
                    ? "bg-accent font-medium text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                )}
              >
                {category.title}
              </button>
            ))}
          </nav>

          <div
            className="min-h-0 min-w-0 flex-1 overflow-y-auto px-6 py-4"
            data-testid="settings-scroll-area"
            role="tabpanel"
            aria-label={activeCategoryMeta.title}
          >
            <div className="space-y-6 pb-4" key={activeCategory}>
              {activeCategory === "rules" && (
                <section className="space-y-4" aria-label="排版规则">
                  <PresetsSection presets={presets} onApplyPreset={onApplyPreset} />
                  <RulesSection rules={rules} enabledSet={enabledSet} onToggleRule={onToggleRule} />
                </section>
              )}
              {activeCategory === "transform" && (
                <ReplacementsSection
                  replacements={replacements}
                  conversion={conversion}
                  buildCapabilities={buildCapabilities}
                  onReplacementsChange={onReplacementsChange}
                  onConversionChange={onConversionChange}
                />
              )}
              {activeCategory === "output" && (
                <OutputSection
                  outputMode={outputMode}
                  layoutMode={layoutMode}
                  onOutputModeChange={onOutputModeChange}
                  onLayoutModeChange={onLayoutModeChange}
                />
              )}
              {activeCategory === "appearance" && (
                <>
                  <ThemeSection
                    theme={theme}
                    onThemeChange={onThemeChange}
                    onFollowSystemChange={onFollowSystemChange}
                    uiScale={uiScale}
                    onUiScaleChange={onUiScaleChange}
                  />
                  <DisplaySection
                    font={font}
                    onFontChange={onFontChange}
                    onResetFont={onResetFont}
                    editorFontSize={editorFontSize}
                    onEditorFontSizeChange={onEditorFontSizeChange}
                  />
                </>
              )}
              {activeCategory === "shortcuts" && (
                <ShortcutsSection
                  shortcutsEnabled={shortcutsEnabled}
                  shortcutBindings={shortcutBindings}
                  onShortcutsEnabledChange={onShortcutsEnabledChange}
                  onSaveShortcutBinding={onSaveShortcutBinding}
                  onResetShortcuts={onResetShortcuts}
                />
              )}
              {activeCategory === "privacy" && (
                <PrivacySection
                  restoreLastInput={restoreLastInput}
                  onRestoreLastInputChange={onRestoreLastInputChange}
                  onClearSavedInput={onClearSavedInput}
                />
              )}
            </div>
          </div>
        </div>

        <SettingsFooter
          appVersion={appVersion}
          settingsStatus={settingsStatus}
          settingsError={settingsError}
          settingsLoadNotices={settingsLoadNotices}
          settingsPath={settingsPath}
          enabledCount={enabled.length}
          rulesCount={rules.length}
          showRuleActions={activeCategory === "rules"}
          onSetAll={onSetAll}
          onResetDefaults={onResetDefaults}
          onOpenChange={onOpenChange}
        />
      </DialogContent>
    </Dialog>
  );
}