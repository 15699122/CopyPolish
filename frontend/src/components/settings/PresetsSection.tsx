import { Button } from "@/components/ui/button";
import type { Preset } from "@/lib/tauri";

interface PresetsSectionProps {
  presets?: Preset[];
  onApplyPreset: (preset: Preset) => void;
}

/** 内置工作流预设；预设只展开为统一请求模型字段。 */
export function PresetsSection({ presets = [], onApplyPreset }: PresetsSectionProps) {
  return (
    <section className="space-y-3" data-testid="presets-settings">
      <div>
        <h3 className="text-sm font-semibold">工作流预设</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          预设只组合规则、替换和字符转换设置，不解析或修改 PDF/DOCX 文件本体。
        </p>
      </div>
      {presets.length === 0 ? (
        <p className="text-xs text-muted-foreground" data-testid="presets-empty">
          浏览器演示模式不加载 Rust 预设；请在桌面版中使用。
        </p>
      ) : (
        <div className="space-y-2" data-testid="preset-list">
          {presets.map((preset) => (
            <div key={preset.key} className="flex items-start gap-3 rounded-md border p-3">
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium">{preset.name}</div>
                <p className="mt-1 text-xs text-muted-foreground">{preset.description}</p>
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onApplyPreset(preset)}
                data-testid={`preset-apply-${preset.key}`}
              >
                应用
              </Button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}