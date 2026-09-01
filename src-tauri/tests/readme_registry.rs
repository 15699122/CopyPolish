// README 规则表与 Rust 注册表一致性检查。
//
// 注册表（registry.rs）是规则的唯一事实来源；README 的规则表必须与之同步。
// 本测试防止两份数据漂移：每条注册规则都能在 README 表格中找到对应行，
// 行内的展示名、默认状态后缀（「，默认关闭」）和 stable key 与注册表一致，
// 且表格行数与注册表规则数相同。

use std::path::PathBuf;
use std::sync::LazyLock;

use chinese_copywriting_formatter_lib::engine;

static README: LazyLock<String> = LazyLock::new(|| {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("README.md");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("无法读取 README.md（{}）：{error}", path.display()))
});

/// 提取 README 规则表数据行：`| 分类 | 规则名 | \`stable.key\` |`。
fn readme_rule_rows() -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    for line in README.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 3 {
            continue;
        }
        let (section, name, key_cell) = (cells[0], cells[1], cells[2]);
        if section == "分类" || section.contains("---") {
            continue;
        }
        // 规则 key 必须以反引号包裹的 stable key 形式出现。
        if !(key_cell.starts_with('`') && key_cell.ends_with('`')) {
            continue;
        }
        rows.push((
            section.to_string(),
            name.to_string(),
            key_cell.trim_matches('`').to_string(),
        ));
    }
    rows
}

#[test]
fn readme_rule_table_matches_registry() {
    let rules = engine::rules();
    assert!(!rules.is_empty(), "注册表不应为空");
    let rows = readme_rule_rows();

    // 表格行数与注册表规则数一致，防止漏写或多余。
    assert_eq!(
        rows.len(),
        rules.len(),
        "README 规则表行数（{}）与注册表规则数（{}）不一致",
        rows.len(),
        rules.len()
    );

    for rule in rules.iter() {
        let meta = &rule.meta;
        let row = rows
            .iter()
            .find(|(_, _, key)| key == &meta.key)
            .unwrap_or_else(|| panic!("README 规则表缺少 stable key `{}`", meta.key));

        let (section, name, _) = row;
        assert_eq!(
            section, &meta.section,
            "规则 `{}` 的分类与注册表不一致",
            meta.key
        );

        // 默认关闭的规则在 README 中以「，默认关闭」后缀标注。
        let expected_name = if meta.default {
            meta.name.clone()
        } else {
            format!("{}，默认关闭", meta.name)
        };
        assert_eq!(
            name, &expected_name,
            "规则 `{}` 的展示名/默认状态标注与注册表不一致",
            meta.key
        );
    }
}

#[test]
fn readme_claims_consistent_with_defaults() {
    // README 声明的默认关闭规则集合应与注册表推导结果一致。
    let defaults = engine::enabled_defaults();
    let rules = engine::rules();
    assert!(!defaults.is_empty(), "默认规则集不应为空");
    assert!(
        defaults.len() < rules.len(),
        "注册表应存在默认关闭的规则（争议/Unicode 等价/名词），否则 README 的「默认关闭」说明失真"
    );
    // 默认规则集与 default 标记一致。
    for rule in rules.iter() {
        let is_default = defaults.iter().any(|key| key == &rule.meta.key);
        assert_eq!(
            is_default, rule.meta.default,
            "规则 `{}` 的 default 标记与 enabled_defaults 不一致",
            rule.meta.key
        );
    }
}
