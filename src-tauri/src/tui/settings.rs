//! TUI 与桌面 GUI 共享设置的桥接层。
//!
//! 共享存储就是现有 GUI 使用的 `rules.yaml`（见 `crate::user_settings`）；
//! TUI 只消费与其相关的两个字段：
//!
//! - `enabled`：启用规则 key 列表 —— 映射为 `RuleSelection`；
//! - `last_input`：最近一次编辑的原文 —— 启动时恢复到输入框。
//!
//! GUI 专属字段（theme、font、ui_scale 等）在此模块中被原样保留，TUI 不解释
//! 也不覆盖它们。测试只覆盖纯映射函数，绝不触碰真实文件系统中的设置文件。

use std::collections::BTreeSet;

use crate::engine::{RuleKind, RuleMeta, RuleRisk, RuleSelection};
use crate::user_settings;

/// 从共享设置加载出的、TUI 关心的配置子集。
pub struct SharedConfig {
    pub selection: RuleSelection,
    pub last_input: String,
}

/// 注册表中全部规则 key 的集合。
fn all_rule_keys(rules: &[RuleMeta]) -> BTreeSet<String> {
    rules.iter().map(|rule| rule.key.clone()).collect()
}

/// 默认启用的规则 key 集合。
fn default_rule_keys(rules: &[RuleMeta]) -> BTreeSet<String> {
    rules
        .iter()
        .filter(|rule| rule.default)
        .map(|rule| rule.key.clone())
        .collect()
}

/// 展开任意 `RuleSelection` 为显式 key 集合（行为与引擎语义一致）。
pub fn expand_selection(selection: &RuleSelection, rules: &[RuleMeta]) -> BTreeSet<String> {
    match selection {
        RuleSelection::All => all_rule_keys(rules),
        RuleSelection::Defaults => default_rule_keys(rules),
        RuleSelection::Only { keys } => keys
            .iter()
            .filter(|key| rules.iter().any(|rule| &rule.key == *key))
            .cloned()
            .collect(),
        RuleSelection::None => BTreeSet::new(),
    }
}

/// 把显式 key 集合规约为最简 `RuleSelection` 表示。
///
/// 与 `Only` 行为等价的特殊集合（空集、全集、恰好等于默认集）会归一化，
/// 使持久化的设置和 CLI 输出保持稳定形式。
pub fn canonical_selection(keys: &BTreeSet<String>, rules: &[RuleMeta]) -> RuleSelection {
    if keys.is_empty() {
        return RuleSelection::None;
    }
    if *keys == all_rule_keys(rules) {
        return RuleSelection::All;
    }
    if *keys == default_rule_keys(rules) {
        return RuleSelection::Defaults;
    }
    RuleSelection::Only {
        keys: keys.iter().cloned().collect(),
    }
}

/// 由 `rules.yaml` 中的 `enabled` 列表推导规则选择；未知 key 安全忽略。
pub fn selection_from_enabled(enabled: &[String], rules: &[RuleMeta]) -> RuleSelection {
    let known = all_rule_keys(rules);
    let keys: BTreeSet<String> = enabled
        .iter()
        .filter(|key| known.contains(*key))
        .cloned()
        .collect();
    canonical_selection(&keys, rules)
}

/// 由规则选择反推 `rules.yaml` 的 `enabled` 列表（排序保证写入稳定）。
pub fn enabled_from_selection(selection: &RuleSelection, rules: &[RuleMeta]) -> Vec<String> {
    expand_selection(selection, rules).into_iter().collect()
}

/// 读取共享设置；无配置文件或 `--no-config` 时返回 None。
///
/// 注意：会访问真实文件系统（exe 目录下的 `rules.yaml`），单测不要调用。
pub fn load_shared(no_config: bool) -> Option<SharedConfig> {
    if no_config {
        return None;
    }
    let loaded = user_settings::load_with_status()?;
    let rules = crate::engine::default_rules();
    Some(SharedConfig {
        selection: selection_from_enabled(&loaded.settings.enabled, &rules),
        last_input: loaded.settings.last_input,
    })
}

/// 将当前规则选择与最近输入写回共享设置。
///
/// 采用“读改写”：先读取现有设置以保留 GUI 专属字段，再更新
/// `enabled` 与 `last_input`。注意：会写入真实文件系统，单测不要调用。
pub fn persist(selection: &RuleSelection, last_input: &str) -> Result<(), String> {
    let rules = crate::engine::default_rules();
    let mut settings = user_settings::load_with_status()
        .map(|loaded| loaded.settings)
        .unwrap_or_default();
    settings.enabled = enabled_from_selection(selection, &rules);
    settings.last_input = last_input.to_string();
    user_settings::save(&settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造包含 3 条规则的迷你注册表：其中两条默认启用。
    fn mini_registry() -> Vec<RuleMeta> {
        ["r1", "r2", "r3"]
            .into_iter()
            .enumerate()
            .map(|(index, key)| RuleMeta {
                key: key.to_string(),
                section: "测试".to_string(),
                name: format!("规则 {key}"),
                description: format!("测试规则 {key}"),
                kind: RuleKind::Typography,
                risk: RuleRisk::Safe,
                disputed: false,
                default: index < 2,
            })
            .collect()
    }

    #[test]
    fn expand_covers_all_modes() {
        let rules = mini_registry();
        assert_eq!(
            expand_selection(&RuleSelection::All, &rules),
            BTreeSet::from(["r1".to_string(), "r2".to_string(), "r3".to_string()])
        );
        assert_eq!(
            expand_selection(&RuleSelection::Defaults, &rules),
            BTreeSet::from(["r1".to_string(), "r2".to_string()])
        );
        assert!(expand_selection(&RuleSelection::None, &rules).is_empty());
        assert_eq!(
            expand_selection(
                &RuleSelection::Only {
                    keys: vec!["r3".to_string()]
                },
                &rules
            ),
            BTreeSet::from(["r3".to_string()])
        );
    }

    #[test]
    fn expand_only_filters_unknown_keys() {
        let rules = mini_registry();
        let expanded = expand_selection(
            &RuleSelection::Only {
                keys: vec!["r1".to_string(), "ghost".to_string()],
            },
            &rules,
        );
        assert_eq!(expanded, BTreeSet::from(["r1".to_string()]));
    }

    #[test]
    fn canonical_maps_special_sets_back_to_named_forms() {
        let rules = mini_registry();
        let all = BTreeSet::from(["r1".to_string(), "r2".to_string(), "r3".to_string()]);
        assert!(matches!(
            canonical_selection(&all, &rules),
            RuleSelection::All
        ));
        let defaults = BTreeSet::from(["r1".to_string(), "r2".to_string()]);
        assert!(matches!(
            canonical_selection(&defaults, &rules),
            RuleSelection::Defaults
        ));
        assert!(matches!(
            canonical_selection(&BTreeSet::new(), &rules),
            RuleSelection::None
        ));
        let partial = BTreeSet::from(["r1".to_string(), "r3".to_string()]);
        match canonical_selection(&partial, &rules) {
            RuleSelection::Only { keys } => {
                assert_eq!(keys, vec!["r1".to_string(), "r3".to_string()])
            }
            other => panic!("expected Only, got {other:?}"),
        }
    }

    #[test]
    fn enabled_list_roundtrip_stays_stable() {
        let rules = mini_registry();
        let original = RuleSelection::Only {
            keys: vec!["r2".to_string(), "r3".to_string()],
        };
        let enabled = enabled_from_selection(&original, &rules);
        assert_eq!(enabled, vec!["r2".to_string(), "r3".to_string()]);
        match selection_from_enabled(&enabled, &rules) {
            RuleSelection::Only { keys } => {
                assert_eq!(keys, vec!["r2".to_string(), "r3".to_string()])
            }
            other => panic!("expected Only, got {other:?}"),
        }
    }

    #[test]
    fn enabled_defaults_map_to_defaults_selection() {
        let rules = mini_registry();
        assert!(matches!(
            selection_from_enabled(&["r1".to_string(), "r2".to_string()], &rules),
            RuleSelection::Defaults
        ));
    }

    #[test]
    fn empty_and_unknown_enabled_lists_degrade_safely() {
        let rules = mini_registry();
        // 空列表 → None。
        assert!(matches!(
            selection_from_enabled(&[], &rules),
            RuleSelection::None
        ));
        // 仅含未知 key → 等价于空集 → None。
        assert!(matches!(
            selection_from_enabled(&["ghost".to_string()], &rules),
            RuleSelection::None
        ));
    }
}
