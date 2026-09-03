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

use super::model::{RuleKind, RuleMeta, RuleRisk};
use super::rule_impls;
use std::collections::{HashMap, HashSet};

/// 稳定规则 key 常量。
pub mod keys {
    pub const CLEANUP_REFERENCE_SQUARE: &str = "cleanup.reference-square";
    pub const CLEANUP_COLLAPSE_HORIZONTAL_SPACES: &str = "cleanup.collapse-horizontal-spaces";
    pub const CLEANUP_LIMIT_BLANK_LINES: &str = "cleanup.limit-blank-lines";
    pub const SPACING_CJK_LATIN: &str = "spacing.cjk-latin";
    pub const SPACING_CJK_NUMBER: &str = "spacing.cjk-number";
    pub const SPACING_NUMBER_UNIT: &str = "spacing.number-unit";
    pub const SPACING_NUMERIC_PUNCTUATION: &str = "spacing.numeric-punctuation";
    pub const SPACING_TEMPERATURE_CJK: &str = "spacing.temperature-cjk";
    pub const SPACING_NO_SPACE_AROUND_FW_PUNCT: &str = "spacing.no-space-around-fw-punct";
    pub const PUNCT_NO_REPETITION: &str = "punctuation.no-repetition";
    pub const PUNCT_FULLWIDTH_CJK: &str = "punctuation.fullwidth-cjk";
    pub const TEXT_HALFWIDTH_DIGITS: &str = "text.halfwidth-digits";
    pub const TEXT_ASCII_PUNCT_IN_LATIN: &str = "text.ascii-punct-in-latin";
    pub const TEXT_UNICODE_EQUIVALENTS: &str = "text.unicode-equivalents";
    pub const NAMING_PROPER_NOUNS: &str = "naming.proper-nouns";
    pub const NAMING_EXPAND_ABBREVIATIONS: &str = "naming.expand-abbreviations";
    pub const SPACING_AROUND_LINKS: &str = "spacing.around-links";
    pub const PUNCT_CORNER_QUOTES: &str = "punctuation.corner-quotes";
}

/// 规则执行阶段。
///
/// 同一阶段内保持注册表原顺序，以便这次重构不改变既有输出；不同阶段
/// 的顺序由 `execution_rules` 显式决定，而不是由调用方自行推断。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RulePhase {
    Cleanup,
    PunctuationNormalization,
    NamingNormalization,
    StructureBoundary,
    TextBoundary,
    FinalCleanup,
}

/// 一条已注册规则：元数据 + 执行阶段 + 处理函数 + 历史 key 别名。
#[derive(Clone)]
pub struct RuleDef {
    pub meta: RuleMeta,
    pub phase: RulePhase,
    /// 当前规则必须排在这些规则之前。
    pub before: &'static [&'static str],
    /// 当前规则必须排在这些规则之后。
    pub after: &'static [&'static str],
    /// 旧版设置文件中的等价 key（中文名）；为空表示无历史别名。
    pub legacy: &'static [&'static str],
    #[allow(clippy::type_complexity)]
    pub apply: fn(&str) -> String,
}

fn phase_for_key(key: &str) -> RulePhase {
    use keys::*;
    match key {
        CLEANUP_REFERENCE_SQUARE
        | CLEANUP_COLLAPSE_HORIZONTAL_SPACES
        | CLEANUP_LIMIT_BLANK_LINES => RulePhase::Cleanup,
        PUNCT_NO_REPETITION
        | PUNCT_FULLWIDTH_CJK
        | TEXT_HALFWIDTH_DIGITS
        | TEXT_ASCII_PUNCT_IN_LATIN => RulePhase::PunctuationNormalization,
        NAMING_PROPER_NOUNS | NAMING_EXPAND_ABBREVIATIONS => RulePhase::NamingNormalization,
        SPACING_AROUND_LINKS | PUNCT_CORNER_QUOTES | TEXT_UNICODE_EQUIVALENTS => {
            RulePhase::StructureBoundary
        }
        SPACING_CJK_LATIN
        | SPACING_CJK_NUMBER
        | SPACING_NUMBER_UNIT
        | SPACING_NUMERIC_PUNCTUATION
        | SPACING_TEMPERATURE_CJK => RulePhase::TextBoundary,
        SPACING_NO_SPACE_AROUND_FW_PUNCT => RulePhase::FinalCleanup,
        _ => RulePhase::FinalCleanup,
    }
}

fn dependencies_for_key(key: &str) -> (&'static [&'static str], &'static [&'static str]) {
    use keys::*;
    match key {
        CLEANUP_REFERENCE_SQUARE
        | CLEANUP_COLLAPSE_HORIZONTAL_SPACES
        | CLEANUP_LIMIT_BLANK_LINES => (&[], &[]),
        PUNCT_FULLWIDTH_CJK => (&[], &[PUNCT_NO_REPETITION][..]),
        TEXT_HALFWIDTH_DIGITS => (&[], &[PUNCT_FULLWIDTH_CJK][..]),
        TEXT_ASCII_PUNCT_IN_LATIN => (&[], &[TEXT_HALFWIDTH_DIGITS][..]),
        NAMING_EXPAND_ABBREVIATIONS => (&[], &[NAMING_PROPER_NOUNS][..]),
        PUNCT_CORNER_QUOTES => (&[], &[SPACING_AROUND_LINKS][..]),
        SPACING_CJK_NUMBER => (&[], &[SPACING_CJK_LATIN][..]),
        SPACING_NUMBER_UNIT => (&[], &[SPACING_CJK_NUMBER][..]),
        SPACING_NUMERIC_PUNCTUATION => (&[], &[SPACING_NUMBER_UNIT][..]),
        SPACING_TEMPERATURE_CJK => (&[], &[SPACING_NUMERIC_PUNCTUATION][..]),
        SPACING_NO_SPACE_AROUND_FW_PUNCT => (&[], &[SPACING_TEMPERATURE_CJK][..]),
        TEXT_UNICODE_EQUIVALENTS => (&[], &[PUNCT_CORNER_QUOTES][..]),
        _ => (&[], &[]),
    }
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
    let (description, kind, risk) = metadata_for_key(key, name, disputed);
    RuleDef {
        meta: RuleMeta {
            key: key.to_string(),
            section: section.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            kind,
            risk,
            disputed,
            default,
        },
        phase: phase_for_key(key),
        before: dependencies_for_key(key).0,
        after: dependencies_for_key(key).1,
        legacy,
        apply,
    }
}

/// 为现有静态规则集中生成面向用户的说明和风险分类。
///
/// 规则调用点继续保持原有参数形态，避免在元数据扩展时遗漏 stable key、
/// 默认状态或 legacy alias；后续新增规则必须在这里补充明确的分类和描述。
fn metadata_for_key(key: &str, _name: &str, disputed: bool) -> (&'static str, RuleKind, RuleRisk) {
    use keys::*;
    match key {
        CLEANUP_REFERENCE_SQUARE => (
            "删除普通文本中的数字引用角标，如 [1]、[2, 3]、[4-7] 和对应的中文方括号形式。",
            RuleKind::Cleanup,
            RuleRisk::Contextual,
        ),
        CLEANUP_COLLAPSE_HORIZONTAL_SPACES => (
            "将普通可编辑文本中的连续 ASCII 空格折叠为一个，不改写代码、链接、公式和其他保护结构。",
            RuleKind::Cleanup,
            RuleRisk::Contextual,
        ),
        CLEANUP_LIMIT_BLANK_LINES => (
            "将普通文本中的连续空行限制为一个空行，跳过受保护的 Markdown、代码、公式和 HTML 结构。",
            RuleKind::Cleanup,
            RuleRisk::Contextual,
        ),
        PUNCT_NO_REPETITION => (
            "折叠连续重复标点，并规范连续叹号和问号的组合。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        PUNCT_FULLWIDTH_CJK => (
            "在中文语境中将适合的半角标点转换为中文全角标点，不处理 URL、代码和公式。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        TEXT_HALFWIDTH_DIGITS => (
            "仅将全角数字 ０–９ 转换为 ASCII 半角数字。",
            RuleKind::Conversion,
            RuleRisk::Safe,
        ),
        TEXT_ASCII_PUNCT_IN_LATIN => (
            "在可识别的英文片段中恢复半角标点，不对全文执行无上下文标点互转。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        TEXT_UNICODE_EQUIVALENTS => (
            "仅转换有限的等价 Unicode 单位字符，不执行全文 NFKC。",
            RuleKind::Conversion,
            RuleRisk::Contextual,
        ),
        NAMING_PROPER_NOUNS => (
            "将有限词典中的专有名词统一为约定大小写。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        NAMING_EXPAND_ABBREVIATIONS => (
            "将有限词典中的不推荐缩写替换为约定写法。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        SPACING_AROUND_LINKS => (
            "在链接与相邻中文之间增加空格；属于可争议的排版偏好。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        PUNCT_CORNER_QUOTES => (
            "在中文语境中将符合条件的直引号转换为直角引号。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        SPACING_CJK_LATIN => (
            "在中文与拉丁字母之间增加空格，并尊重 grapheme 和保护边界。",
            RuleKind::Typography,
            RuleRisk::Safe,
        ),
        SPACING_CJK_NUMBER => (
            "在中文与数字之间增加空格，并尊重单位和结构保护边界。",
            RuleKind::Typography,
            RuleRisk::Safe,
        ),
        SPACING_NUMBER_UNIT => (
            "在数字与已识别的单位之间增加空格。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        SPACING_NUMERIC_PUNCTUATION => (
            "移除小数点、时间/比例冒号、数字分组逗号和数字斜线两侧的异常 ASCII 空格，并保留版本号/IP 等连续点号数字链。",
            RuleKind::Cleanup,
            RuleRisk::Contextual,
        ),
        SPACING_TEMPERATURE_CJK => (
            "在摄氏度或华氏度符号与中文之间增加空格。",
            RuleKind::Typography,
            RuleRisk::Contextual,
        ),
        SPACING_NO_SPACE_AROUND_FW_PUNCT => (
            "移除全角标点与相邻字符之间不必要的空格。",
            RuleKind::Typography,
            RuleRisk::Safe,
        ),
        _ => (
            "未分类规则。",
            RuleKind::Typography,
            if disputed {
                RuleRisk::Contextual
            } else {
                RuleRisk::Safe
            },
        ),
    }
}

/// 规则注册表。数组顺序是 UI 展示和同阶段规则的稳定 tie-breaker。
/// 历史规则已全部迁移；争议规则、Unicode 输出规范化和两个名词规则默认关闭。
static RULES: std::sync::LazyLock<Vec<RuleDef>> = std::sync::LazyLock::new(|| {
    use keys::*;
    vec![
        def(
            CLEANUP_REFERENCE_SQUARE,
            "文本清洗",
            "删除方括号引用角标",
            false,
            false,
            &["删除方括号引用角标"],
            rule_impls::remove_square_reference_badges,
        ),
        def(
            CLEANUP_COLLAPSE_HORIZONTAL_SPACES,
            "文本清洗",
            "折叠连续空格",
            false,
            false,
            &["折叠连续空格"],
            rule_impls::collapse_horizontal_spaces,
        ),
        def(
            CLEANUP_LIMIT_BLANK_LINES,
            "文本清洗",
            "限制连续空行",
            false,
            false,
            &["限制连续空行"],
            rule_impls::limit_blank_lines,
        ),
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
            TEXT_UNICODE_EQUIVALENTS,
            "全角和半角",
            "统一等价 Unicode 单位字符",
            false,
            false,
            &[],
            rule_impls::unicode_equivalents,
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
            SPACING_NUMERIC_PUNCTUATION,
            "空格",
            "修复数值标点异常空格",
            false,
            false,
            &["修复数值标点异常空格"],
            rule_impls::numeric_punctuation_space,
        ),
        def(
            SPACING_TEMPERATURE_CJK,
            "空格",
            "摄氏度/华氏度符号与中文之间加空格",
            false,
            true,
            &[],
            rule_impls::temperature_cjk_space,
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

/// 返回 pipeline 使用的显式阶段顺序。
///
/// 排序是稳定的：相同阶段保留注册表顺序，从而在引入 phase 元数据时
/// 不改变已有规则组合的输出。
pub fn execution_rules() -> Vec<&'static RuleDef> {
    resolve_execution_order(&RULES).expect("invalid rule dependency graph")
}

/// 按 phase 与注册表顺序做稳定拓扑排序，并校验依赖图完整性。
pub fn resolve_execution_order(defs: &[RuleDef]) -> Result<Vec<&RuleDef>, String> {
    let mut indices = HashMap::with_capacity(defs.len());
    for (index, rule) in defs.iter().enumerate() {
        if indices.insert(rule.key(), index).is_some() {
            return Err(format!("duplicate rule key: {}", rule.key()));
        }
    }

    let mut edges: Vec<HashSet<usize>> = vec![HashSet::new(); defs.len()];
    let mut indegree = vec![0usize; defs.len()];
    for (index, rule) in defs.iter().enumerate() {
        for dependency in rule.before.iter().chain(rule.after.iter()) {
            let Some(&target) = indices.get(dependency) else {
                return Err(format!("unknown rule dependency: {dependency}"));
            };
            let (from, to) = if rule.before.contains(dependency) {
                (index, target)
            } else {
                (target, index)
            };
            if edges[from].insert(to) {
                indegree[to] += 1;
            }
        }
    }

    let mut available: Vec<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect();
    let mut ordered = Vec::with_capacity(defs.len());
    while !available.is_empty() {
        available.sort_by_key(|&index| (defs[index].phase, index));
        let index = available.remove(0);
        ordered.push(index);
        for target in edges[index].clone() {
            indegree[target] -= 1;
            if indegree[target] == 0 {
                available.push(target);
            }
        }
    }

    if ordered.len() != defs.len() {
        return Err("cyclic rule dependency graph".to_string());
    }
    Ok(ordered.into_iter().map(|index| &defs[index]).collect())
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
                .find(|r| r.legacy.contains(&k.as_str()))
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
