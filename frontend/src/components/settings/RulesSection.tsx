import { useMemo } from "react";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import type { Rule } from "@/lib/tauri";

interface RulesSectionProps {
  rules: Rule[];
  enabledSet: Set<string>;
  onToggleRule: (key: string) => void;
}

const kindLabels = {
  cleanup: "清洗",
  conversion: "转换",
  typography: "排版",
} as const;

const riskLabels = {
  safe: "低风险",
  contextual: "需复核",
  destructive: "高风险",
} as const;

/** 生成规则的“修改前 → 修改后”示例文案，用于悬停提示与辅助技术描述。 */
function formatRuleExample(rule: Rule): string {
  return `示例：“${rule.example.before}” → “${rule.example.after}”`;
}

/** 规则分组列表；分组仅影响展示顺序，不改变执行顺序。 */
export function RulesSection({ rules, enabledSet, onToggleRule }: RulesSectionProps) {
  const groups = useMemo(() => {
    const map = new Map<string, Rule[]>();
    // 仅影响设置窗口的展示顺序：默认开启的规则在上，默认关闭的在下；
    // 同类内部保持后端返回顺序，不影响 Rust pipeline 的实际执行顺序。
    const sorted = [...rules].sort((a, b) => Number(b.default) - Number(a.default));
    for (const rule of sorted) {
      const list = map.get(rule.section) ?? [];
      list.push(rule);
      map.set(rule.section, list);
    }
    return Array.from(map.entries());
  }, [rules]);

  return (
    <>
      {groups.map(([section, items]) => (
        <section key={section}>
          <h3 className="mb-2 text-sm font-semibold">{section}</h3>
          <div className="space-y-2">
            {items.map((rule) => (
              <div
                key={rule.key}
                className="flex items-start gap-3 rounded-md border p-3"
                title={formatRuleExample(rule)}
                data-testid={`rule-card-${rule.key}`}
              >
                <Checkbox
                  id={`rule-${rule.key}`}
                  checked={enabledSet.has(rule.key)}
                  onCheckedChange={() => onToggleRule(rule.key)}
                  data-testid={`rule-${rule.key}`}
                  aria-label={rule.name}
                  aria-describedby={`rule-example-${rule.key}`}
                />
                <Label htmlFor={`rule-${rule.key}`} className="min-w-0 flex-1 text-sm leading-5">
                  <span className="flex flex-wrap items-center gap-x-1.5 gap-y-0.5">
                    <span>{rule.name}</span>
                    {rule.kind && (
                      <span className="text-xs text-muted-foreground">[{kindLabels[rule.kind]}]</span>
                    )}
                    {rule.risk && (
                      <span className="text-xs text-muted-foreground">· {riskLabels[rule.risk]}</span>
                    )}
                    {rule.disputed && (
                      <span className="text-xs text-muted-foreground">（争议，默认关闭）</span>
                    )}
                  </span>
                  {rule.description && (
                    <span className="mt-1 block text-xs font-normal text-muted-foreground">
                      {rule.description}
                    </span>
                  )}
                  <span id={`rule-example-${rule.key}`} className="sr-only">
                    {formatRuleExample(rule)}
                  </span>
                </Label>
              </div>
            ))}
          </div>
        </section>
      ))}
    </>
  );
}