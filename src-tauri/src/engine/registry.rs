// engine/registry.rs
// =============================================================================
// 规则注册表：规则的唯一事实来源。
//
// - `key` 是稳定的机器标识（英文点分），不再使用中文展示名或 Python slug；
//   展示名 / 分组只存在于 RuleMeta，供前端渲染。
// - `legacy` 保存历史设置文件（rules.yaml）中可能出现的中文名 key，
//   `normalize_rule_keys` 负责把旧 key 映射为新 key、丢弃未知 key。
// - 新增规则：在 RULES 表追加一个 RuleDef 即可；command 层、pipeline、
//   前端均无需改动。
// =============================================================================

use super::model::RuleMeta;
use super::rule_impls;

/// 稳定规则 key 常量。
pub mod keys {
    pub const SPACING_CJK_LATIN: &str = "spacing.cjk-latin";
    pub const SPACING_CJK_NUMBER: &str = "spacing.cjk-number";
    pub const SPACING_NUMBER_UNIT: &str = "spacing.number-unit";
    pub const SPACING_NO_SPACE_AROUND_FW_PUNCT: &str = "spacing.no-space-around-fw-punct";
    pub const PUNCT_NO_REPETITION: &str = "punctuation.no-repetition";
    pub const PUNCT_FULLWIDTH_CJK: &str = "punctuation.fullwidth-cjk";
    pub const TEXT_HALFWIDTH_DIGITS: &str = "text.halfwidth-digits";
    pub const TEXT_ASCII_PUNCT_IN_LATIN: &str = "text.ascii-punct-in-latin";
    pub const NAMING_PROPER_NOUNS: &str = "naming.proper-nouns";
    pub const NAMING_EXPAND_ABBREVIATIONS: &str = "naming.expand-abbreviations";
    pub const SPACING_AROUND_LINKS: &str = "spacing.around-links";
    pub const PUNCT_CORNER_QUOTES: &str = "punctuation.corner-quotes";
}

/// 一条已注册规则：元数据 + 处理函数 + 历史 key 别名。
#[derive(Clone)]
pub struct RuleDef {
    pub meta: RuleMeta,
    /// 旧版设置文件中的等价 key（中文名）；为空表示无历史别名。
    pub legacy: &'static [&'static str],
    #[allow(clippy::type_complexity)]
    pub apply: fn(&str) -> String,
}

impl RuleDef {
    pub fn key(&self) -> &str {
        &self.meta.key
    }
}

fn def(
    key: &'static str,
    section: &'static str,
    name: &'static str,
    disputed: bool,
    default: bool,
    legacy: &'static [&'static str],
    apply: fn(&str) -> String,
) -> RuleDef {
    RuleDef {
        meta: RuleMeta {
            key: key.to_string(),
            section: section.to_string(),
            name: name.to_string(),
            disputed,
            default,
        },
        legacy,
        apply,
    }
}

/// 规则注册表。数组顺序即 pipeline 的执行顺序。
/// 历史 12 条规则已全部迁移；争议规则与两个名词规则默认关闭。
static RULES: std::sync::LazyLock<Vec<RuleDef>> = std::sync::LazyLock::new(|| {
    use keys::*;
    vec![
        def(
            PUNCT_NO_REPETITION,
            "标点符号",
            "不重复使用标点符号",
            false,
            true,
            &["不重复使用标点符号"],
            rule_impls::no_repeat_punct,
        ),
        def(
            PUNCT_FULLWIDTH_CJK,
            "全角和半角",
            "使用全角中文标点",
            false,
            true,
            &["使用全角中文标点"],
            rule_impls::fullwidth_chinese_punct,
        ),
        def(
            TEXT_HALFWIDTH_DIGITS,
            "全角和半角",
            "数字使用半角字符",
            false,
            true,
            &["数字使用半角字符"],
            rule_impls::fullwidth_digits,
        ),
        def(
            TEXT_ASCII_PUNCT_IN_LATIN,
            "全角和半角",
            "遇到完整的英文整句、特殊名词，其内容使用半角标点",
            false,
            true,
            &["遇到完整的英文整句_特殊名词_其内容使用半角标点"],
            rule_impls::halfwidth_in_english,
        ),
        def(
            NAMING_PROPER_NOUNS,
            "名词",
            "专有名词使用正确的大小写",
            false,
            false,
            &["专有名词使用正确的大小写"],
            rule_impls::proper_nouns,
        ),
        def(
            NAMING_EXPAND_ABBREVIATIONS,
            "名词",
            "不要使用不地道的缩写",
            false,
            false,
            &["不要使用不地道的缩写"],
            rule_impls::no_abbr,
        ),
        def(
            SPACING_AROUND_LINKS,
            "争议",
            "链接之间增加空格",
            true,
            false,
            &["链接之间增加空格"],
            rule_impls::around_links,
        ),
        def(
            PUNCT_CORNER_QUOTES,
            "争议",
            "简体中文使用直角引号",
            true,
            false,
            &["简体中文使用直角引号"],
            rule_impls::corner_quotes,
        ),
        def(
            SPACING_CJK_LATIN,
            "空格",
            "中英文之间需要增加空格",
            false,
            true,
            &["中英文之间需要增加空格"],
            rule_impls::cn_en_space,
        ),
        def(
            SPACING_CJK_NUMBER,
            "空格",
            "中文与数字之间需要增加空格",
            false,
            true,
            &["中文与数字之间需要增加空格"],
            rule_impls::cn_digit_space,
        ),
        def(
            SPACING_NUMBER_UNIT,
            "空格",
            "数字与单位之间需要增加空格",
            false,
            true,
            &["数字与单位之间需要增加空格"],
            rule_impls::digit_unit_space,
        ),
        def(
            SPACING_NO_SPACE_AROUND_FW_PUNCT,
            "空格",
            "全角标点与其他字符之间不加空格",
            false,
            true,
            &["全角标点与其他字符之间不加空格"],
            rule_impls::fw_punct_no_space,
        ),
    ]
});

/// 全部已注册规则（按执行顺序）。
pub fn rules() -> &'static [RuleDef] {
    &RULES
}

/// 返回全部规则元数据（供 Tauri command 序列化给前端）。
pub fn default_rules() -> Vec<RuleMeta> {
    RULES.iter().map(|r| r.meta.clone()).collect()
}

/// 默认启用的规则 key 列表（新稳定 key）。
pub fn enabled_defaults() -> Vec<String> {
    RULES
        .iter()
        .filter(|r| r.meta.default)
        .map(|r| r.meta.key.clone())
        .collect()
}

/// 按 key 查找规则。
pub fn find_rule(key: &str) -> Option<&'static RuleDef> {
    RULES.iter().find(|r| r.meta.key == key)
}

/// 归一化规则 key 列表：
/// - 新 key 原样保留（去重，保持首次出现顺序）；
/// - 历史 key 映射为对应新 key；
/// - 未知 key 安全丢弃。
pub fn normalize_rule_keys(keys: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(keys.len());
    for k in keys {
        let normalized = if RULES.iter().any(|r| r.meta.key == *k) {
            Some(k.clone())
        } else {
            RULES
                .iter()
                .find(|r| r.legacy.iter().any(|l| *l == k.as_str()))
                .map(|r| r.meta.key.clone())
        };
        if let Some(n) = normalized {
            if !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}
