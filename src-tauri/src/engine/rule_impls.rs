// engine/rule_impls.rs
// =============================================================================
// 规则实现：每条规则是一个 `fn(&str) -> String` 纯函数，由 registry 注册。
// 迁移自旧 rust_engine.rs，行为保持不变（含默认启用集下的输出）。
// =============================================================================

use regex::{Captures, Regex};
use std::sync::OnceLock;

use super::semantic_tokens::{scan_semantic_tokens, SemanticTokenKind};
use super::tokenizer::contains_cjk;
use super::unicode_boundaries::{for_each_adjacent_unit, BoundaryStrategy, ScriptClass, TextUnit};

// ---------------------------------------------------------------------------
// 基础工具
// ---------------------------------------------------------------------------

pub fn normalize_spaces(text: &str) -> String {
    static MULTI: OnceLock<Regex> = OnceLock::new();
    MULTI
        .get_or_init(|| Regex::new(r" {2,}").unwrap())
        .replace_all(text, " ")
        .to_string()
}

/// cleanup.collapse-horizontal-spaces：折叠普通可编辑文本中的连续 ASCII 空格。
pub fn collapse_horizontal_spaces(text: &str) -> String {
    normalize_spaces(text)
}

/// cleanup.reference-square：删除数字方括号引用角标。
///
/// Markdown 链接、引用定义、代码和其他结构由上游 span 层排除；此函数本身
/// 只负责普通可编辑片段中的字面模式，圆括号引用不在本轮范围内。
pub fn remove_square_reference_badges(text: &str) -> String {
    static ASCII: OnceLock<Regex> = OnceLock::new();
    static CJK: OnceLock<Regex> = OnceLock::new();
    let ascii = ASCII.get_or_init(|| Regex::new(r"\[[0-9]+(?:\s*[,\-–—]\s*[0-9]+)*\]").unwrap());
    let cjk = CJK.get_or_init(|| Regex::new(r"【[0-9]+(?:\s*[,，\-–—]\s*[0-9]+)*】").unwrap());
    let text = ascii.replace_all(text, "");
    cjk.replace_all(&text, "").to_string()
}

/// cleanup.limit-blank-lines 的单行回退函数。
/// 跨行压缩由 `edit_plan` 在结构 span 边界上完成；这里保留纯函数入口，
/// 供注册表和后续预设共享同一规则元数据。
pub fn limit_blank_lines(text: &str) -> String {
    text.to_string()
}

/// text.unicode-equivalents：将已确认等价的 Unicode 单位字符统一为推荐写法。
///
/// 该规则只处理有限映射，不执行全文 NFKC；默认关闭，避免未经用户选择
/// 改写数学字母、兼容字符或其他文本。
pub fn unicode_equivalents(text: &str) -> String {
    text.replace('µ', "μ").replace('Å', "Å")
}

fn is_ascii_alnum(ch: Option<char>) -> bool {
    ch.map(|c| c.is_ascii_alphanumeric()).unwrap_or(false)
}

fn is_ascii_alpha(ch: Option<char>) -> bool {
    ch.map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
}

fn is_closing_quote(ch: Option<char>) -> bool {
    matches!(ch, Some('」' | '』' | '》' | '’' | '”' | '）'))
}

/// 折叠同一字符的连续重复。
fn collapse_repeated_runs(text: &str, set: &[char]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev: Option<char> = None;
    for ch in text.chars() {
        if prev == Some(ch) && set.contains(&ch) {
            continue;
        }
        out.push(ch);
        prev = Some(ch);
    }
    out
}

fn word_pattern(wrong: &str) -> Regex {
    Regex::new(&format!(
        r"(?i)(^|[^A-Za-z0-9]){}([^A-Za-z0-9]|$)",
        regex::escape(wrong)
    ))
    .unwrap()
}

/// 词级替换必须使用预编译缓存：TextEdit 迁移后规则按可编辑片段高频调用，
/// 每次重新编译正则会成为数量级热点（roadmap §8 性能基线实测）。
fn replace_word_case_insensitive(text: &str, pattern: &Regex, right: &str) -> String {
    pattern
        .replace_all(text, |caps: &Captures| {
            format!("{}{}{}", &caps[1], right, &caps[2])
        })
        .to_string()
}

/// 仅转换直引号（全角中文标点规则内部的行为）。
fn straight_corner_quotes(text: &str) -> String {
    static DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = DOUBLE
        .get_or_init(|| Regex::new(&format!(r#""([^"\n]*?{cjk}[^"\n]*?)""#)).unwrap())
        .replace_all(text, "「$1」");
    SINGLE
        .get_or_init(|| Regex::new(&format!(r"'([^'\n]*?{cjk}[^'\n]*?)'")).unwrap())
        .replace_all(&text, "『$1』")
        .to_string()
}

// ---------------------------------------------------------------------------
// 规则实现
// ---------------------------------------------------------------------------

/// 按判定单位在相邻类别之间插空；单位以完整 &str 输出，
/// 保证 emoji ZWJ / 组合附加符等 grapheme 不会被拆开。
fn insert_space_between_units<F>(text: &str, strategy: BoundaryStrategy, should_insert: F) -> String
where
    F: Fn(&TextUnit<'_>, &TextUnit<'_>) -> bool,
{
    let mut out = String::with_capacity(text.len());
    for_each_adjacent_unit(text, strategy, |previous, script, unit_text| {
        if let Some((previous_script, previous_text)) = previous {
            let previous_unit = TextUnit {
                text: previous_text,
                byte_start: 0,
                script: previous_script,
            };
            let unit = TextUnit {
                text: unit_text,
                byte_start: 0,
                script,
            };
            if should_insert(&previous_unit, &unit) {
                out.push(' ');
            }
        }
        out.push_str(unit_text);
    });
    out
}

/// 在 [SPACING_CJK_LATIN] 基础上，额外处理 Markdown 行内单星强调片段（如 *t*、*p*）
/// 与中文或比较运算符相邻的边界，以及以 Unicode 上标结尾的科学单位片段与中文的边界。
///
/// 边界判定使用 grapheme cluster 策略（`Graphemes`）；`_with` 变体仅供新旧策略对比
/// 测试（`LegacyChars`），生产固定使用 Graphemes。
pub(crate) fn cn_en_space_with(text: &str, strategy: BoundaryStrategy) -> String {
    let base = insert_space_between_units(text, strategy, |a, b| {
        matches!(
            (a.script, b.script),
            (ScriptClass::Han, ScriptClass::Latin) | (ScriptClass::Latin, ScriptClass::Han)
        )
    });
    let text = break_emphasis_boundaries(&base);
    break_superscript_unit_boundaries(&text)
}

/// 在 `cn_en_space` 结果上，补齐「CJK / 比较运算符 ↔ Markdown 单星强调片段」间的空格。
///
/// 匹配形式：`*word*`（word 为纯 ASCII 字母，如 *t*、*p*）；视为原子片段，
/// 与邻接的 CJK 字符或比较运算符 `<`/`>`/`=` 之间插入空格，例如
/// `中*t*` → `中 *t*`、`*p*<0.05` → `*p* <0.05`。
/// 不拆分与相邻英文字母/数字（如 `a*b*c` 保持原样），且不影响 `**粗体**`。
fn break_emphasis_boundaries(text: &str) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    // 边界：CJK 字 或 比较运算符 < > =（排除 ASCII 字母/数字/空格/星号），
    // 仅当 `*word*` 与上述字符相邻时加空格，避免伤及 `a*b*c` 及 `**粗体**`。
    let boundary = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}<>=]";
    let before = BEFORE.get_or_init(|| {
        Regex::new(&format!(r"({boundary})(\*[A-Za-z]+\*)")).expect("invalid before-emphasis regex")
    });
    let after = AFTER.get_or_init(|| {
        Regex::new(&format!(r"(\*[A-Za-z]+\*)({boundary})")).expect("invalid after-emphasis regex")
    });
    let text = before.replace_all(text, "$1 $2");
    after.replace_all(&text, "$1 $2").to_string()
}

/// 补齐「以 Unicode 上标结尾的科学单位片段（如 mg·mL⁻¹）与相邻中文」间的空格。
///
/// 片段形如 `mg·mL⁻¹`（字母开头，可含字母/数字与 `·` 连接段，以上标字符结尾）。
/// tokenizer 将上标归类为独立类别，直接 CJK↔Latin/Digit 规则无法覆盖其尾部边界；
/// 此处仅在片段整体与中文相邻时补空格，不改动片段内部。化学式（如 Fe²⁺）在保护层
/// 已转为占位符，不会进入本规则。
fn break_superscript_unit_boundaries(text: &str) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    let unit = r"[A-Za-z][A-Za-z0-9]*(?:[·⋅][A-Za-z0-9]+)*[⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾]+";
    let cjk = r"\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}";
    let before = BEFORE.get_or_init(|| {
        Regex::new(&format!(r"([{cjk}])({unit})")).expect("invalid before-unit regex")
    });
    let after = AFTER.get_or_init(|| {
        Regex::new(&format!(r"({unit})([{cjk}])")).expect("invalid after-unit regex")
    });
    let text = before.replace_all(text, "$1 $2");
    after.replace_all(&text, "$1 $2").to_string()
}

fn break_han_math_boundaries(text: &str) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    let operator = r"[∂±×≈≤≥]";
    let cjk = r"\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}";
    let before = BEFORE.get_or_init(|| {
        Regex::new(&format!(r"([{cjk}])({operator})")).expect("invalid Han-math boundary regex")
    });
    let after = AFTER.get_or_init(|| {
        Regex::new(&format!(r"({operator})([{cjk}])")).expect("invalid math-Han boundary regex")
    });
    let text = before.replace_all(text, "$1 $2");
    after.replace_all(&text, "$1 $2").to_string()
}

pub fn cn_en_space(text: &str) -> String {
    cn_en_space_with(text, BoundaryStrategy::Graphemes)
}

/// spacing.cjk-number：中文与数字之间增加空格。
pub(crate) fn cn_digit_space_with(text: &str, strategy: BoundaryStrategy) -> String {
    let text = insert_space_between_units(text, strategy, |a, b| {
        matches!(
            (a.script, b.script),
            (ScriptClass::Han, ScriptClass::Digit) | (ScriptClass::Digit, ScriptClass::Han)
        )
    });
    break_han_math_boundaries(&text)
}

pub fn cn_digit_space(text: &str) -> String {
    cn_digit_space_with(text, BoundaryStrategy::Graphemes)
}

/// spacing.number-unit：数字与单位之间增加空格。
pub fn digit_unit_space(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    let mut cursor = 0;

    for token in scan_semantic_tokens(text) {
        if token.start < cursor {
            continue;
        }
        out.push_str(&text[cursor..token.unit_start]);

        let unit = &text[token.unit_start..token.end];
        let is_temperature = token.kind == SemanticTokenKind::Temperature;
        let has_space = text[token.number_end..token.unit_start]
            .chars()
            .any(char::is_whitespace);

        let is_percent_like = matches!(unit, "%" | "％" | "‰");
        if !is_temperature && !is_percent_like && !has_space {
            out.push(' ');
        }
        out.push_str(unit);
        cursor = token.end;
    }

    out.push_str(&text[cursor..]);

    // 兼容历史规则：角度/百分号在数字前的空格会被移除，百分号包括全角写法。
    static DEG_PERCENT: OnceLock<Regex> = OnceLock::new();
    static PERMILLE_CJK: OnceLock<Regex> = OnceLock::new();
    let out = DEG_PERCENT
        .get_or_init(|| Regex::new(r"(\d)\s+([°%％‰])").unwrap())
        .replace_all(&out, "$1$2");
    PERMILLE_CJK
        .get_or_init(|| {
            Regex::new(r"(‰)([\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}])").unwrap()
        })
        .replace_all(&out, "$1 $2")
        .to_string()
}

/// spacing.temperature-cjk：摄氏度/华氏度符号与紧随的中文之间加空格。
/// 不调整数字与温标符号之间的写法，例如保留 `4℃` 或 `-20 ℃`。
pub fn temperature_cjk_space(text: &str) -> String {
    static TEMPERATURE_BEFORE_CJK: OnceLock<Regex> = OnceLock::new();
    TEMPERATURE_BEFORE_CJK
        .get_or_init(|| {
            Regex::new(r"([℃℉])([\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}])").unwrap()
        })
        .replace_all(text, "$1 $2")
        .to_string()
}

/// spacing.no-space-around-fw-punct：全角标点与其他字符之间不加空格。
pub fn fw_punct_no_space(text: &str) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    let text = BEFORE
        .get_or_init(|| Regex::new(r"\s+([，。；：！？、（）《》【】「」『』…—])").unwrap())
        .replace_all(text, "$1");
    AFTER
        .get_or_init(|| Regex::new(r"([，。；：！？、（）《》【】「」『』…—])\s+").unwrap())
        .replace_all(&text, "$1")
        .to_string()
}

/// punctuation.no-repetition：不重复使用标点符号。
pub fn no_repeat_punct(text: &str) -> String {
    static MIXED: OnceLock<Regex> = OnceLock::new();
    // 三步：1) [！？!?~～] 同字符折叠；2) [。，；：、] 同字符折叠；3) 混合叹问号 -> ？！
    let collapsed = collapse_repeated_runs(
        &collapse_repeated_runs(text, &['！', '？', '!', '?', '~', '～']),
        &['。', '，', '；', '：', '、'],
    );
    MIXED
        .get_or_init(|| Regex::new(r"[！？!?][！？!?]+").unwrap())
        .replace_all(&collapsed, "？！")
        .to_string()
}

/// punctuation.fullwidth-cjk：使用全角中文标点（仅对含中文的行生效）。
pub fn fullwidth_chinese_punct(text: &str) -> String {
    if !contains_cjk(text) {
        return text.to_string();
    }
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        let prev = idx.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(idx + 1).copied();
        let converted = match *ch {
            ',' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '，',
            ';' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '；',
            ':' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '：',
            '!' if !is_ascii_alnum(next) => '！',
            '?' if !is_ascii_alnum(next) => '？',
            '(' if !is_ascii_alnum(prev) => '（',
            ')' if !is_ascii_alnum(next) => '）',
            '.' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '。',
            _ => *ch,
        };
        out.push(converted);
    }
    straight_corner_quotes(&out)
}

/// text.halfwidth-digits：数字使用半角字符。
pub fn fullwidth_digits(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '０'..='９' => char::from_u32(ch as u32 - 0xfee0).unwrap_or(ch),
            _ => ch,
        })
        .collect()
}

/// text.ascii-punct-in-latin：英文整句/特殊名词中使用半角标点。
pub fn halfwidth_in_english(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        let prev = idx.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(idx + 1).copied();
        let replacement = match *ch {
            '，' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some(", "),
            '：' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some(": "),
            '；' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some("; "),
            '＆' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some(" & "),
            '。' if is_ascii_alnum(prev) && (is_ascii_alnum(next) || is_closing_quote(next)) => {
                Some(".")
            }
            '！' if is_ascii_alnum(prev) && (is_ascii_alnum(next) || is_closing_quote(next)) => {
                Some("!")
            }
            '？' if is_ascii_alnum(prev) && (is_ascii_alnum(next) || is_closing_quote(next)) => {
                Some("?")
            }
            '（' if is_ascii_alnum(prev) && is_ascii_alnum(next) => Some("("),
            '）' if is_ascii_alnum(prev) && is_ascii_alnum(next) => Some(")"),
            _ => None,
        };
        if let Some(s) = replacement {
            out.push_str(s);
        } else {
            out.push(*ch);
        }
    }
    normalize_spaces(&out)
}

/// punctuation.corner-quotes：简体中文使用直角引号（争议规则）。
pub fn corner_quotes(text: &str) -> String {
    static SMART_DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SMART_SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = straight_corner_quotes(text);
    let text = SMART_DOUBLE
        .get_or_init(|| Regex::new(&format!(r"“([^”\n]*?{cjk}[^”\n]*?)”")).unwrap())
        .replace_all(&text, "「$1」");
    SMART_SINGLE
        .get_or_init(|| Regex::new(&format!(r"‘([^’\n]*?{cjk}[^’\n]*?)’")).unwrap())
        .replace_all(&text, "『$1』")
        .to_string()
}

// ---------------------------------------------------------------------------
// 词典型规则数据
// ---------------------------------------------------------------------------

const PROPER_NOUNS: &[(&str, &str)] = &[
    ("github", "GitHub"),
    ("foursquare", "Foursquare"),
    ("microsoft", "Microsoft"),
    ("google", "Google"),
    ("facebook", "Facebook"),
    ("twitter", "Twitter"),
    ("youtube", "YouTube"),
    ("linkedin", "LinkedIn"),
    ("instagram", "Instagram"),
    ("wikipedia", "Wikipedia"),
    ("wechat", "WeChat"),
    ("javascript", "JavaScript"),
    ("typescript", "TypeScript"),
    ("html5", "HTML5"),
    ("css3", "CSS3"),
    ("json", "JSON"),
    ("http", "HTTP"),
    ("https", "HTTPS"),
    ("api", "API"),
    ("sql", "SQL"),
    ("php", "PHP"),
    ("ios", "iOS"),
    ("ipados", "iPadOS"),
    ("android", "Android"),
    ("iphone", "iPhone"),
    ("ipad", "iPad"),
    ("imac", "iMac"),
    ("mac", "Mac"),
    ("macos", "macOS"),
    ("windows", "Windows"),
    ("linux", "Linux"),
    ("bluetooth", "Bluetooth"),
    ("wifi", "Wi-Fi"),
    ("wi-fi", "Wi-Fi"),
    ("nextjs", "Next.js"),
    ("npm", "npm"),
    ("react", "React"),
    ("vue", "Vue"),
    ("mongodb", "MongoDB"),
    ("corporation", "Corporation"),
    ("inc", "Inc"),
];

const ABBR_MAP: &[(&str, &str)] = &[
    ("ts", "TypeScript"),
    ("h5", "HTML5"),
    ("rjs", "React"),
    ("nextjs", "Next.js"),
    ("fed", "前端开发者"),
];

/// naming.proper-nouns：专有名词使用正确的大小写。
pub fn proper_nouns(text: &str) -> String {
    static RULE: OnceLock<Regex> = OnceLock::new();
    let rule = RULE.get_or_init(|| {
        let mut words: Vec<&str> = PROPER_NOUNS.iter().map(|(wrong, _)| *wrong).collect();
        // 前缀词（如 `mac` / `macos`、`http` / `https`）必须让较长候选优先，
        // 否则合并正则可能先尝试较短候选，再在边界校验失败后产生额外回溯。
        words.sort_by_key(|word| std::cmp::Reverse(word.len()));
        let alternatives = words
            .into_iter()
            .map(regex::escape)
            .collect::<Vec<_>>()
            .join("|");
        Regex::new(&format!(r"(?i)(?:{alternatives})")).expect("invalid proper-noun regex")
    });

    rule.replace_all(text, |caps: &Captures| {
        let capture = caps.get(0).expect("proper-noun match must have a word");
        let matched = capture.as_str();
        let has_left_boundary = capture.start() == 0
            || !text[..capture.start()]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphanumeric());
        let has_right_boundary = capture.end() == text.len()
            || !text[capture.end()..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric());
        if !has_left_boundary || !has_right_boundary {
            return matched.to_string();
        }
        let normalized = matched.to_ascii_lowercase();
        let right = PROPER_NOUNS
            .iter()
            .find_map(|(wrong, right)| (*wrong == normalized).then_some(*right))
            .expect("proper-noun regex match must exist in dictionary");
        right.to_string()
    })
    .to_string()
}

/// naming.expand-abbreviations：不要使用不地道的缩写。
pub fn no_abbr(text: &str) -> String {
    static RULES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    static CJK_COLLAPSE: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    let rules = RULES.get_or_init(|| {
        ABBR_MAP
            .iter()
            .map(|(wrong, right)| (word_pattern(wrong), *right))
            .collect()
    });
    let collapses = CJK_COLLAPSE.get_or_init(|| {
        ABBR_MAP
            .iter()
            .filter(|(_, right)| contains_cjk(right))
            .map(|(_, right)| {
                (
                    Regex::new(&format!(r"\s+{}", regex::escape(right))).unwrap(),
                    *right,
                )
            })
            .collect()
    });
    let mut out = text.to_string();
    for (pattern, right) in rules {
        out = replace_word_case_insensitive(&out, pattern, right);
    }
    for (pattern, right) in collapses {
        out = pattern.replace_all(&out, *right).to_string();
    }
    out
}

/// spacing.around-links：链接之间增加空格（争议规则）。
/// 实现位于 protection::space_around_links（需要保护层配合），此处转发。
pub fn around_links(text: &str) -> String {
    super::protection::space_around_links(text)
}
