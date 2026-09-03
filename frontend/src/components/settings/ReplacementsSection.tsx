import { X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import type { BuildCapabilities, CharacterConversion, ReplacementPair } from "@/lib/tauri";

interface ReplacementsSectionProps {
  replacements: ReplacementPair[];
  conversion: CharacterConversion;
  buildCapabilities: BuildCapabilities;
  onReplacementsChange: (replacements: ReplacementPair[]) => void;
  onConversionChange: (conversion: CharacterConversion) => void;
}

/** 自定义字面量替换与可选简繁转换；顺序即执行顺序。 */
export function ReplacementsSection({
  replacements,
  conversion,
  buildCapabilities,
  onReplacementsChange,
  onConversionChange,
}: ReplacementsSectionProps) {
  const updateReplacement = (index: number, patch: Partial<ReplacementPair>) => {
    onReplacementsChange(
      replacements.map((replacement, currentIndex) =>
        currentIndex === index ? { ...replacement, ...patch } : replacement,
      ),
    );
  };

  return (
    <section className="space-y-3" data-testid="text-transform-settings">
      <div>
        <h3 className="text-sm font-semibold">文本替换与转换</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          替换按列表顺序执行，仅支持字面量；空来源会被忽略。
        </p>
      </div>

      <div className="space-y-2" data-testid="replacement-list">
        {replacements.map((replacement, index) => (
          <div key={index} className="grid grid-cols-[auto_minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-2">
            <Checkbox
              checked={replacement.active}
              onCheckedChange={(checked) => updateReplacement(index, { active: checked === true })}
              aria-label={`启用替换 ${index + 1}`}
              data-testid={`replacement-active-${index}`}
            />
            <input
              value={replacement.from}
              onChange={(event) => updateReplacement(index, { from: event.target.value })}
              placeholder="来源"
              aria-label={`替换 ${index + 1} 来源`}
              data-testid={`replacement-from-${index}`}
              className="h-9 min-w-0 rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
            <input
              value={replacement.to}
              onChange={(event) => updateReplacement(index, { to: event.target.value })}
              placeholder="目标"
              aria-label={`替换 ${index + 1} 目标`}
              data-testid={`replacement-to-${index}`}
              className="h-9 min-w-0 rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              onClick={() => onReplacementsChange(replacements.filter((_, currentIndex) => currentIndex !== index))}
              aria-label={`删除替换 ${index + 1}`}
              data-testid={`replacement-remove-${index}`}
            >
              <X />
            </Button>
          </div>
        ))}
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => onReplacementsChange([...replacements, { from: "", to: "", active: true }])}
          data-testid="replacement-add"
        >
          添加替换
        </Button>
      </div>

      <div className="space-y-1.5">
        <label htmlFor="conversion-select" className="text-xs font-medium text-muted-foreground">
          简繁转换
        </label>
        <select
          id="conversion-select"
          value={conversion}
          onChange={(event) => onConversionChange(event.target.value as CharacterConversion)}
          data-testid="conversion-select"
          aria-label="简繁转换"
          className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring sm:max-w-60"
        >
          <option value="none">不转换</option>
          <option value="t2s" disabled={!buildCapabilities.simplifiedTradConversion}>繁体转简体</option>
          <option value="s2t" disabled={!buildCapabilities.simplifiedTradConversion}>简体转繁体</option>
        </select>
        <p className="text-xs text-muted-foreground">
          {buildCapabilities.simplifiedTradConversion
            ? "当前构建已包含简繁转换能力。"
            : "当前构建未包含简繁转换能力；请使用 simplified-trad-conversion 构建。"}
        </p>
      </div>
    </section>
  );
}