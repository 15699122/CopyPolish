import type { RefObject } from "react";
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
        className="flex h-[min(680px,calc(100vh-2rem))] w-[min(560px,calc(100vw-2rem))] max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)] flex-col gap-0 overflow-hidden p-0 sm:min-h-130 sm:min-w-120"
      >
        <DialogHeader className="shrink-0 border-b px-6 py-5 pr-12">
          <DialogTitle>设置 — 排版规则</DialogTitle>
          <DialogDescription>
            逐条启用/停用规则。已启用 {enabled.length}/{rules.length} 条
          </DialogDescription>
        </DialogHeader>

        <div className="min-h-0 flex-1 overflow-y-auto px-6 py-4" data-testid="settings-scroll-area">
          <div className="space-y-6 pb-4">
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
            <ShortcutsSection
              shortcutsEnabled={shortcutsEnabled}
              shortcutBindings={shortcutBindings}
              onShortcutsEnabledChange={onShortcutsEnabledChange}
              onSaveShortcutBinding={onSaveShortcutBinding}
              onResetShortcuts={onResetShortcuts}
            />
            <ReplacementsSection
              replacements={replacements}
              conversion={conversion}
              buildCapabilities={buildCapabilities}
              onReplacementsChange={onReplacementsChange}
              onConversionChange={onConversionChange}
            />
            <PresetsSection presets={presets} onApplyPreset={onApplyPreset} />
            <OutputSection
              outputMode={outputMode}
              layoutMode={layoutMode}
              onOutputModeChange={onOutputModeChange}
              onLayoutModeChange={onLayoutModeChange}
            />
            <PrivacySection
              restoreLastInput={restoreLastInput}
              onRestoreLastInputChange={onRestoreLastInputChange}
              onClearSavedInput={onClearSavedInput}
            />
            <RulesSection rules={rules} enabledSet={enabledSet} onToggleRule={onToggleRule} />
          </div>
        </div>

        <SettingsFooter
          appVersion={appVersion}
          settingsStatus={settingsStatus}
          settingsError={settingsError}
          settingsLoadNotices={settingsLoadNotices}
          settingsPath={settingsPath}
          onSetAll={onSetAll}
          onResetDefaults={onResetDefaults}
          onOpenChange={onOpenChange}
        />
      </DialogContent>
    </Dialog>
  );
}