// engine/pipeline.rs
// =============================================================================
// 格式化主流程（全部规则执行已收敛到 TextEdit 应用层）：
//   1. 归一化换行符（处理后还原）；
//   2. 跨行来源清洗（当前为普通文本连续空行限制）；
//   3. 请求层：自定义字面量替换（有序、span 保护前）；
//   4. 请求层：字符转换（简繁，互斥，当前仅 None 生效）；
//   5. 在可编辑区间通过 TextEdit 应用清洗、标点/名词规范化规则；
//   6. 保护层：不透明结构 span（含化学式）转为内部占位符；
//   7. 在受保护文本上通过 TextEdit 应用结构边界/文本边界/最终清理规则；
//   8. 行内占位符补边界空格 -> 还原全部占位符。
//
// 规则选择由 `RuleSelection` 显式表达；未知 key 安全忽略。
// =============================================================================

use super::edit_plan::{
    apply_blank_line_cleanup, apply_editable_rules, apply_protected_text_rules,
};
use super::model::{CharacterConversion, FormatRequest, ReplacementPair};
use super::protection::{
    placeholder, restore, space_around_inline_placeholders, space_around_math_placeholders,
};
use super::spans::{scan_all_spans, TextSpan};

#[cfg(feature = "simplified-trad-conversion")]
use opencc_fmmseg::{OpenCC, OpenccConfig};

/// 格式化文本的正式入口。
pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    format_text_impl(req)
}

/// 返回当前文本中应受结构保护、禁止清洗/转换改写的不透明区间。
/// 与 `apply_replacements` / `apply_character_conversion` 共用，保证
/// 自定义替换与字符转换都不会改写 Markdown 链接、URL、代码、公式或化学式内部。
fn opaque_ranges(text: &str) -> Vec<(usize, usize)> {
    scan_all_spans(text)
        .into_iter()
        .filter(|span| {
            span.priority == super::spans::SpanPriority::OpaqueStructure
                || span.kind == super::spans::SpanKind::ChemicalFormula
        })
        .map(|span| (span.start, span.end))
        .collect()
}

/// 应用自定义字面量替换（有序、仅 active 项）。
///
/// 替换在 span 保护前执行，但先扫描当前文本中的不透明结构，
/// 只改写可编辑区间；因此不会改写 Markdown 链接、URL、代码、公式或化学式内部。
/// `from` 为字面量字符串（非正则），按向量顺序依次应用。空 `from` 项被安全跳过。
fn apply_replacements(text: &str, replacements: &[ReplacementPair]) -> String {
    let mut out = text.to_string();
    for pair in replacements {
        if !pair.active || pair.from.is_empty() {
            continue;
        }
        let protected_ranges = opaque_ranges(&out);
        let mut next = String::with_capacity(out.len());
        let mut cursor = 0usize;
        while let Some(relative_start) = out[cursor..].find(&pair.from) {
            let start = cursor + relative_start;
            let end = start + pair.from.len();
            let overlaps_protected =
                protected_ranges
                    .iter()
                    .any(|&(protected_start, protected_end)| {
                        start < protected_end && protected_start < end
                    });
            if overlaps_protected {
                next.push_str(&out[cursor..end]);
            } else {
                next.push_str(&out[cursor..start]);
                next.push_str(&pair.to);
            }
            cursor = end;
        }
        next.push_str(&out[cursor..]);
        out = next;
    }
    out
}

/// 应用字符转换（简繁）。
///
/// 未启用 `simplified-trad-conversion` feature 时为互斥占位：仅 `None`
/// 实际生效，T2S/S2T 返回原文。
#[cfg(not(feature = "simplified-trad-conversion"))]
fn apply_character_conversion(text: &str, conversion: CharacterConversion) -> String {
    match conversion {
        CharacterConversion::None => text.to_string(),
        // 简繁转换依赖与词汇级语义由独立 Spike 决策；默认构建占位保持原文。
        CharacterConversion::TraditionalToSimplified
        | CharacterConversion::SimplifiedToTraditional => text.to_string(),
    }
}

/// 启用 `simplified-trad-conversion` 时的字符转换实现。
///
/// 通过 `opencc-fmmseg`（MIT，OpenCC 风格词典 + FMM 分词）实现互斥的
/// T2S/S2T；转换只作用于可编辑区间，不改写结构 span。转换器以
/// `OnceLock` 缓存（内置压缩词典解压成本高），`convert_with_config`
/// 以 `&self` 调用，可安全跨请求共享。
#[cfg(feature = "simplified-trad-conversion")]
fn apply_character_conversion(text: &str, conversion: CharacterConversion) -> String {
    match conversion {
        CharacterConversion::None => text.to_string(),
        CharacterConversion::TraditionalToSimplified => {
            static T2S: std::sync::OnceLock<OpenCC> = std::sync::OnceLock::new();
            let conv = T2S.get_or_init(OpenCC::new);
            convert_editable(text, |segment| convert_traditional(conv, segment))
        }
        CharacterConversion::SimplifiedToTraditional => {
            static S2T: std::sync::OnceLock<OpenCC> = std::sync::OnceLock::new();
            let conv = S2T.get_or_init(OpenCC::new);
            convert_editable(text, |segment| convert_simplified(conv, segment))
        }
    }
}

#[cfg(feature = "simplified-trad-conversion")]
fn convert_traditional(conv: &OpenCC, segment: &str) -> String {
    conv.convert_with_config(segment, OpenccConfig::T2s, false)
}

#[cfg(feature = "simplified-trad-conversion")]
fn convert_simplified(conv: &OpenCC, segment: &str) -> String {
    conv.convert_with_config(segment, OpenccConfig::S2t, false)
}

/// 对文本的可编辑区间逐个应用转换函数，跳过不透明结构，再重组。
#[cfg(feature = "simplified-trad-conversion")]
fn convert_editable(text: &str, mut convert: impl FnMut(&str) -> String) -> String {
    let ranges = opaque_ranges(text);
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for &(start, end) in &ranges {
        out.push_str(&convert(&text[cursor..start]));
        out.push_str(&text[start..end]);
        cursor = end;
    }
    out.push_str(&convert(&text[cursor..]));
    out
}

fn normalize_newlines(text: &str) -> (String, &'static str) {
    if text.contains("\r\n") {
        (text.replace("\r\n", "\n"), "\r\n")
    } else if text.contains('\r') {
        (text.replace('\r', "\n"), "\r")
    } else {
        (text.to_string(), "\n")
    }
}

fn restore_newlines(text: &str, newline: &str) -> String {
    if newline == "\n" {
        text.to_string()
    } else {
        text.replace('\n', newline)
    }
}

// ---------------------------------------------------------------------------
// span 感知格式化管线。
//
// 用 scan_all_spans 划定不可编辑区间：可编辑规则在原文上以 TextEdit 应用，
// 其余规则在受保护文本上以 TextEdit 应用。占位符仅用于承载不可编辑 span；
// 全部规则执行已收敛到 edit_plan.rs 的 TextEdit 模型。
// ---------------------------------------------------------------------------

/// 把“不透明结构”span 替代入内部保护占位符，返回受控文本与占位符表。
/// 语义原子（测量/温度/科学单位/数学）不占位——它们应作为普通文本参与
/// 逐行规则（如 `spacing.number-unit`、`temperature-cjk`），与生产一致。
type Placeholder = (String, String);

struct ProtectedSpans {
    text: String,
    all: Vec<Placeholder>,
    inline: Vec<Placeholder>,
    math: Vec<Placeholder>,
}

fn protect_spans(text: &str, spans: &[TextSpan]) -> ProtectedSpans {
    let mut opaque: Vec<TextSpan> = spans
        .iter()
        .filter(|span| {
            span.priority == super::spans::SpanPriority::OpaqueStructure
                || span.kind == super::spans::SpanKind::ChemicalFormula
        })
        .copied()
        .collect();

    // `2×3cm²` 会被数学扫描识别为 `2×3`、单位扫描识别为 `3cm²`。
    // 生产 pipeline 的占位符边界不会在二者之间插空，因此混合管线需要将
    // 数学表达式后紧邻的单位后缀扩展进同一保护区间。
    for span in spans
        .iter()
        .filter(|span| span.kind == super::spans::SpanKind::MathExpression)
    {
        let mut end = span.end;
        while let Some(ch) = text[end..].chars().next() {
            let next_end = end + ch.len_utf8();
            if ch.is_ascii_alphanumeric()
                || matches!(ch, '²' | '³' | '⁰'..='⁹' | '₀'..='₉' | '⁻' | '⁺')
            {
                end = next_end;
            } else {
                break;
            }
        }
        if end > span.end {
            opaque.push(TextSpan {
                start: span.start,
                end,
                kind: span.kind,
                priority: super::spans::SpanPriority::OpaqueStructure,
            });
        }
    }
    opaque.sort_by_key(|span| (span.start, std::cmp::Reverse(span.end)));
    opaque = super::spans::arbitrate_spans(opaque);
    let mut output = String::with_capacity(text.len());
    let mut placeholders = Vec::with_capacity(opaque.len());
    let mut inline_placeholders = Vec::with_capacity(opaque.len());
    let mut math_placeholders = Vec::with_capacity(opaque.len());
    let mut cursor = 0usize;
    for (index, span) in opaque.iter().enumerate() {
        output.push_str(&text[cursor..span.start]);
        let ph = placeholder(index);
        let value = text[span.start..span.end].to_string();
        placeholders.push((ph.clone(), value.clone()));
        if span.kind == super::spans::SpanKind::LatexMath {
            math_placeholders.push((ph.clone(), value));
        } else {
            inline_placeholders.push((ph.clone(), value));
        }
        output.push_str(&ph);
        cursor = span.end;
    }
    output.push_str(&text[cursor..]);
    ProtectedSpans {
        text: output,
        all: placeholders,
        inline: inline_placeholders,
        math: math_placeholders,
    }
}

/// 用 span 划分不可编辑区间，再格式化可编辑内容。
fn format_text_impl(req: &FormatRequest) -> Result<String, String> {
    let (text, newline) = normalize_newlines(&req.text);

    // 1. 跨行来源清洗先处理（连续空行）。
    let text = apply_blank_line_cleanup(&text, &req.selection)?;

    // 2. 请求层：自定义字面量替换与字符转换（均在 span 保护前执行，
    //    避免破坏 Markdown / URL / 代码结构）。
    let text = apply_replacements(&text, &req.replacements);
    let text = apply_character_conversion(&text, req.conversion);

    // 3. 在可编辑区间通过 TextEdit 应用清洗、标点/名词规范化规则。
    let text = apply_editable_rules(&text, &req.selection)?;

    let spans = scan_all_spans(&text);
    let protected_spans = protect_spans(&text, &spans);

    // 结构边界/文本边界/清理规则：受保护文本上的 TextEdit，
    // 顺序与迁移前保护层行循环一致，随后补占位符边缘空格并还原。
    let formatted = apply_protected_text_rules(&protected_spans.text, &req.selection)?;
    let formatted = space_around_inline_placeholders(&formatted, &protected_spans.inline);
    let formatted = space_around_math_placeholders(&formatted, &protected_spans.math);
    let restored = restore(&formatted, &protected_spans.all);
    Ok(restore_newlines(&restored, newline))
}

/// 分阶段计时剖析入口（`--features profile-stages`，仅本地性能分析用）。
///
/// 按 `format_text_impl` 的实际执行顺序返回各阶段耗时（纳秒）：
/// 1. `normalize`：换行归一化；
/// 2. `blank_line_cleanup`：跨行连续空行清洗；
/// 3. `replacements`：自定义字面量替换；
/// 4. `conversion`：字符转换（简繁）；
/// 5. `editable_rules`：清洗、标点/名词规范化（原文 TextEdit，内含一次全文 span 扫描）；
/// 6. `scan_spans`：Markdown/URL/LaTeX/化学式等 span 扫描；
/// 7. `protect`：不透明 span 转占位符；
/// 8. `protected_rules`：结构边界/文本边界/最终清理规则（受保护文本 TextEdit）；
/// 9. `placeholder_spacing`：占位符边缘补空格；
/// 10. `restore`：还原占位符与换行符。
///
/// 输出与 `format_text` 完全一致；本函数不参与任何测试门禁与打包。
#[cfg(feature = "profile-stages")]
pub fn format_text_stage_timings(
    req: &FormatRequest,
) -> Result<Vec<(&'static str, std::time::Duration)>, String> {
    use std::time::Instant;

    let mut timings: Vec<(&'static str, std::time::Duration)> = Vec::with_capacity(10);

    let t = Instant::now();
    let (text, newline) = normalize_newlines(&req.text);
    timings.push(("normalize", t.elapsed()));

    let t = Instant::now();
    let text = apply_blank_line_cleanup(&text, &req.selection)?;
    timings.push(("blank_line_cleanup", t.elapsed()));

    let t = Instant::now();
    let text = apply_replacements(&text, &req.replacements);
    timings.push(("replacements", t.elapsed()));

    let t = Instant::now();
    let text = apply_character_conversion(&text, req.conversion);
    timings.push(("conversion", t.elapsed()));

    let t = Instant::now();
    let text = apply_editable_rules(&text, &req.selection)?;
    timings.push(("editable_rules", t.elapsed()));

    let t = Instant::now();
    let spans = scan_all_spans(&text);
    timings.push(("scan_spans", t.elapsed()));

    let t = Instant::now();
    let protected_spans = protect_spans(&text, &spans);
    timings.push(("protect", t.elapsed()));

    let t = Instant::now();
    let formatted = apply_protected_text_rules(&protected_spans.text, &req.selection)?;
    timings.push(("protected_rules", t.elapsed()));

    let t = Instant::now();
    let formatted = space_around_inline_placeholders(&formatted, &protected_spans.inline);
    let formatted = space_around_math_placeholders(&formatted, &protected_spans.math);
    timings.push(("placeholder_spacing", t.elapsed()));

    let t = Instant::now();
    let restored = restore(&formatted, &protected_spans.all);
    let _ = restore_newlines(&restored, newline);
    timings.push(("restore", t.elapsed()));

    Ok(timings)
}

/// 逐规则计时（`--features profile-stages`，仅本地性能分析用）。
///
/// 对每条已注册规则在整篇文本上直接调用其 `apply`，返回规则 key 与耗时。
/// 注意：生产管线是“按可编辑行区间逐行应用”，本函数按整篇应用，
/// 绝对值偏大，但用于**规则间相对热点比较**与生产归因方向一致。
#[cfg(feature = "profile-stages")]
pub fn per_rule_timings(req: &FormatRequest) -> Vec<(&'static str, std::time::Duration)> {
    use std::time::Instant;

    let (text, _) = normalize_newlines(&req.text);
    let mut out = Vec::new();
    for rule in super::registry::rules() {
        let t = Instant::now();
        let _ = (rule.apply)(&text);
        out.push((rule.key(), t.elapsed()));
    }
    out
}

/// 语义 span 与结构 span 的分段扫描计时（`--features profile-stages`）。
#[cfg(feature = "profile-stages")]
pub fn scan_split_timings(text: &str) -> Vec<(&'static str, std::time::Duration)> {
    use std::time::Instant;

    let mut out = Vec::with_capacity(2);
    let t = Instant::now();
    let _ = super::spans::scan_semantic_spans(text);
    out.push(("scan_semantic", t.elapsed()));
    let t = Instant::now();
    let _ = super::spans::scan_structure_spans(text);
    out.push(("scan_structure", t.elapsed()));
    out
}

/// 结构扫描器逐个计时（`--features profile-stages`，仅本地性能分析用）。
#[cfg(feature = "profile-stages")]
pub fn scan_structure_timings(text: &str) -> Vec<(&'static str, std::time::Duration)> {
    super::spans::scan_structure_spans_timings(text).1
}
