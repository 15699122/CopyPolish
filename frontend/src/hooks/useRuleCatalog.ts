import { useEffect, useState } from "react";

import { getEnabledDefaults, getRules, type Rule } from "@/lib/tauri";

export interface UseRuleCatalogOptions {
  loadSettings: (rules: Rule[], defaults: string[]) => Promise<void>;
  onError: (cause: unknown) => void;
}

export interface UseRuleCatalogResult {
  rules: Rule[];
}

/** 加载规则元数据和默认启用集，并在完成后触发设置恢复。 */
export function useRuleCatalog({ loadSettings, onError }: UseRuleCatalogOptions): UseRuleCatalogResult {
  const [rules, setRules] = useState<Rule[]>([]);

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
      } catch (cause) {
        if (!cancelled) onError(cause);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [loadSettings, onError]);

  return { rules };
}