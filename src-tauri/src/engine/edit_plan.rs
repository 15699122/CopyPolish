//! TextEdit 计划与冲突仲裁。
//!
//! 本模块是 Span → TextEdit 迁移基础设施。标点/名词规范化阶段通过
//! `apply_editable_rules`、全角标点清理阶段通过 `apply_protected_text_rules`
//! 接入生产 pipeline；编辑使用原文 UTF-8 字节区间，先校验边界，再按优先级
//! 仲裁，最后从后向前应用。

use super::model::RuleSelection;
use super::registry::{execution_rules, RulePhase};
use super::semantic_tokens::scan_math_expressions;
use super::semantic_tokens::scan_semantic_tokens;
use super::spans::{
    scan_all_spans, scan_editable_protection_spans, SpanKind, SpanPriority, TextSpan,
};
use super::tokenizer::{classify, CharKind};
use super::unicode_boundaries::{units, BoundaryStrategy, ScriptClass};

const EDITABLE_PHASES: &[RulePhase] = &[
    RulePhase::Cleanup,
    RulePhase::PunctuationNormalization,
    RulePhase::NamingNormalization,
];

/// 结构边界、文本边界与 FinalCleanup 阶段在受保护文本（占位符已就位）上执行：
/// 这些规则可能互相生成新的处理边界（如直角引号转换产生新的全角标点），
/// 因此保持 `execution_rules()` 的全局顺序逐条应用；占位符行与空行跳过，
/// 其余整行作为可编辑区间生成 TextEdit。
const PROTECTED_TEXT_PHASES: &[RulePhase] = &[
    RulePhase::StructureBoundary,
    RulePhase::TextBoundary,
    RulePhase::FinalCleanup,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EditPriority {
    Editable = 0,
    SemanticAtomic = 1,
    OpaqueStructure = 2,
}

impl From<SpanPriority> for EditPriority {
    fn from(priority: SpanPriority) -> Self {
        match priority {
            SpanPriority::Editable => Self::Editable,
            SpanPriority::SemanticAtomic => Self::SemanticAtomic,
            SpanPriority::OpaqueStructure => Self::OpaqueStructure,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub priority: EditPriority,
}

impl TextEdit {
    pub(crate) fn new(
        text: &str,
        start: usize,
        end: usize,
        replacement: impl Into<String>,
        priority: EditPriority,
    ) -> Result<Self, String> {
        if start > end {
            return Err(format!("invalid edit range: {start}..{end}"));
        }
        if end > text.len() {
            return Err(format!("edit range exceeds text: {start}..{end}"));
        }
        if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return Err(format!("edit range splits UTF-8 text: {start}..{end}"));
        }
        Ok(Self {
            start,
            end,
            replacement: replacement.into(),
            priority,
        })
    }

    fn len(&self) -> usize {
        self.end - self.start
    }

    fn conflicts(&self, other: &Self) -> bool {
        match (self.start == self.end, other.start == other.end) {
            (true, true) => self.start == other.start,
            (true, false) => self.start > other.start && self.start < other.end,
            (false, true) => other.start > self.start && other.start < self.end,
            (false, false) => self.start < other.end && other.start < self.end,
        }
    }
}

/// 选择不重叠编辑：优先级更高、同优先级区间更长、再按起点稳定排序。
pub(crate) fn arbitrate_edits(mut candidates: Vec<TextEdit>) -> Vec<TextEdit> {
    candidates.sort_by_key(|edit| {
        (
            std::cmp::Reverse(edit.priority),
            edit.start,
            std::cmp::Reverse(edit.len()),
        )
    });

    let mut accepted: Vec<TextEdit> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if accepted.iter().all(|edit| !edit.conflicts(&candidate)) {
            accepted.push(candidate);
        }
    }
    accepted.sort_by_key(|edit| (edit.start, edit.end));
    accepted
}

/// 在原文上应用已仲裁的非重叠编辑。
pub(crate) fn apply_edits(text: &str, edits: &[TextEdit]) -> Result<String, String> {
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| (edit.start, edit.end));

    for edit in &ordered {
        if edit.end > text.len()
            || edit.start > edit.end
            || !text.is_char_boundary(edit.start)
            || !text.is_char_boundary(edit.end)
        {
            return Err(format!("invalid edit range: {}..{}", edit.start, edit.end));
        }
    }
    if ordered.windows(2).any(|pair| pair[0].conflicts(&pair[1])) {
        return Err("overlapping edits must be arbitrated before application".to_string());
    }

    let mut output = text.to_string();
    for edit in ordered.into_iter().rev() {
        output.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Ok(output)
}

fn selection_enabled(selection: &RuleSelection, key: &str) -> bool {
    match selection {
        RuleSelection::All => true,
        RuleSelection::Defaults => super::registry::enabled_defaults()
            .iter()
            .any(|item| item == key),
        RuleSelection::Only { keys } => keys.iter().any(|item| item == key),
        RuleSelection::None => false,
    }
}

fn editable_line_ranges(text: &str, spans: &[TextSpan]) -> Vec<(usize, usize)> {
    let opaque: Vec<(usize, usize)> = spans
        .iter()
        .filter(|span| {
            span.priority == SpanPriority::OpaqueStructure || span.kind == SpanKind::ChemicalFormula
        })
        .map(|span| (span.start, span.end))
        .collect();
    let mut ranges = Vec::new();
    let mut line_start = 0usize;
    for line in text.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let content_end = line_end.saturating_sub(usize::from(line.ends_with('\n')));
        let mut cursor = line_start;
        for &(start, end) in opaque.iter().filter(|&&(start, _)| start < content_end) {
            if end <= line_start || start >= content_end {
                continue;
            }
            let start = start.max(line_start);
            if cursor < start {
                ranges.push((cursor, start));
            }
            cursor = cursor.max(end.min(content_end));
        }
        if cursor < content_end {
            ranges.push((cursor, content_end));
        }
        line_start = line_end;
    }
    ranges
}

/// 在可编辑文本区间内按规则生成并应用 TextEdit。
///
/// 清洗、标点规范化和名词规范化阶段均在这里处理。这些规则不应改写结构/语义
/// span，因此每个编辑都限定在单行可编辑区间内；结构边界规则仍由 pipeline
/// 的内部保护层负责，避免改变既有边界空格语义。跨行的连续空行规则由
/// `apply_blank_line_cleanup` 单独处理。
/// 在可编辑文本区间内按规则生成并应用 TextEdit。
///
/// 覆盖标点规范化和名词规范化阶段。这些规则不应改写结构/语义 span，因此
/// 编辑限定在单行可编辑区间内；span 只扫描一次，同区间内的规则按
/// `execution_rules()` 顺序在片段内串行执行（等价于迁移前“每条规则全文
/// 重扫并应用”的语义，因为规则均为行内纯函数且编辑不跨区间），
/// 避免规则数量增长带来的 O(rules × 全文扫描) 开销。
pub(crate) fn apply_editable_rules(
    text: &str,
    selection: &RuleSelection,
) -> Result<String, String> {
    let rules: Vec<&super::registry::RuleDef> = execution_rules()
        .into_iter()
        .filter(|rule| {
            EDITABLE_PHASES.contains(&rule.phase)
                && rule.key() != super::registry::keys::CLEANUP_LIMIT_BLANK_LINES
                && selection_enabled(selection, rule.key())
        })
        .collect();
    if rules.is_empty() {
        return Ok(text.to_string());
    }

    let spans = scan_editable_protection_spans(text);
    let edits: Vec<TextEdit> = editable_line_ranges(text, &spans)
        .into_iter()
        .filter_map(|(start, end)| {
            let original = &text[start..end];
            let mut fragment = original.to_string();
            for rule in &rules {
                fragment = (rule.apply)(&fragment);
            }
            (fragment != original).then(|| {
                TextEdit::new(text, start, end, fragment, EditPriority::Editable)
                    .expect("editable line range must be valid")
            })
        })
        .collect();
    apply_edits(text, &arbitrate_edits(edits))
}

/// 在结构 span 之外限制连续空行；每个普通文本空行 run 保留一个空行。
///
/// 该规则必须跨行处理，不能复用普通的逐行规则循环。与 Markdown、代码、
/// LaTeX 或 HTML 结构重叠的行完全跳过，优先保证结构不被破坏。
pub(crate) fn apply_blank_line_cleanup(
    text: &str,
    selection: &RuleSelection,
) -> Result<String, String> {
    let key = super::registry::keys::CLEANUP_LIMIT_BLANK_LINES;
    if !selection_enabled(selection, key) {
        return Ok(text.to_string());
    }

    let spans = scan_all_spans(text);
    let opaque: Vec<(usize, usize)> = spans
        .iter()
        .filter(|span| {
            span.priority == SpanPriority::OpaqueStructure || span.kind == SpanKind::ChemicalFormula
        })
        .map(|span| (span.start, span.end))
        .collect();

    let mut edits = Vec::new();
    let mut line_start = 0usize;
    let mut blank_run: Vec<(usize, usize)> = Vec::new();
    for line in text.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let content_end = line_end.saturating_sub(usize::from(line.ends_with('\n')));
        let is_blank = text[line_start..content_end].trim().is_empty();
        let protected = opaque
            .iter()
            .any(|&(start, end)| start < line_end && line_start < end);
        if is_blank && !protected {
            blank_run.push((line_start, line_end));
        } else {
            for &(start, end) in blank_run.iter().skip(1) {
                edits.push(TextEdit::new(text, start, end, "", EditPriority::Editable)?);
            }
            blank_run.clear();
        }
        line_start = line_end;
    }
    for &(start, end) in blank_run.iter().skip(1) {
        edits.push(TextEdit::new(text, start, end, "", EditPriority::Editable)?);
    }
    apply_edits(text, &arbitrate_edits(edits))
}

/// 在受保护文本上按行应用结构边界、文本边界与 FinalCleanup 阶段规则（TextEdit 形式）。
///
/// 调用时机：保护占位符就位之后、行内占位符补空格之前。规则按
/// `execution_rules()` 的全局顺序逐条应用（StructureBoundary → TextBoundary →
/// FinalCleanup，同阶段保持注册表顺序），与迁移前保护层行循环的执行顺序一致。
/// 占位符文本不含会被这些规则改写的边界，整行占位符与空行直接跳过，
/// 其余行作为单一可编辑区间生成编辑。
pub(crate) fn apply_protected_text_rules(
    text: &str,
    selection: &RuleSelection,
) -> Result<String, String> {
    let rules: Vec<&super::registry::RuleDef> = execution_rules()
        .into_iter()
        .filter(|rule| {
            PROTECTED_TEXT_PHASES.contains(&rule.phase) && selection_enabled(selection, rule.key())
        })
        .collect();
    if rules.is_empty() {
        return Ok(text.to_string());
    }

    let mut current = text.to_string();
    for rule in rules {
        let mut cursor = 0usize;
        let mut edits = Vec::new();
        for line in current.split('\n') {
            let start = cursor;
            let end = cursor + line.len();
            cursor = end + 1;
            if line.trim().is_empty() || super::protection::is_placeholder_line(line) {
                continue;
            }
            let replacement = (rule.apply)(line);
            if replacement != line {
                edits.push(
                    TextEdit::new(&current, start, end, replacement, EditPriority::Editable)
                        .expect("whole-line range must be a valid edit"),
                );
            }
        }
        current = apply_edits(&current, &arbitrate_edits(edits))?;
    }
    Ok(current)
}

/// 为已经仲裁的语义 span 生成边界编辑计划。
///
/// 该函数是生产 pipeline 迁移前的对照路径：只处理最终保留下来的
/// `Measurement` / `Temperature` / `ScientificUnit` / `MathExpression` /
/// `LatexMath` span，结构 span 内部不会生成编辑。温标的数字与符号间距
/// 继续 `25°C` / `4℃` 的既有语义；文本边界受规则选择门控。
pub(crate) fn plan_semantic_boundary_edits(text: &str) -> Vec<TextEdit> {
    plan_semantic_boundary_edits_with_selection(text, &RuleSelection::All)
}

/// 文本边界（Han↔Latin / Han↔Digit）的规则启用门控，
/// 对齐生产 pipeline 中 `spacing.cjk-latin` / `spacing.cjk-number` 的选择语义。
#[derive(Clone, Copy, Debug)]
struct TextBoundaryGate {
    cjk_latin: bool,
    cjk_number: bool,
}

impl TextBoundaryGate {
    fn from_selection(selection: &RuleSelection) -> Self {
        fn enabled(selection: &RuleSelection, key: &str) -> bool {
            match selection {
                RuleSelection::All => true,
                RuleSelection::None => false,
                RuleSelection::Defaults => {
                    super::registry::enabled_defaults().iter().any(|k| k == key)
                }
                RuleSelection::Only { keys } => keys.iter().any(|k| k == key),
            }
        }
        Self {
            cjk_latin: enabled(selection, super::registry::keys::SPACING_CJK_LATIN),
            cjk_number: enabled(selection, super::registry::keys::SPACING_CJK_NUMBER),
        }
    }

    fn allows(&self, pair: (ScriptClass, ScriptClass)) -> bool {
        use ScriptClass::{Digit, Han, Latin};
        match pair {
            (Han, Latin) | (Latin, Han) => self.cjk_latin,
            (Han, Digit) | (Digit, Han) => self.cjk_number,
            _ => false,
        }
    }
}

pub(crate) fn plan_semantic_boundary_edits_with_selection(
    text: &str,
    selection: &RuleSelection,
) -> Vec<TextEdit> {
    let gate = TextBoundaryGate::from_selection(selection);
    let spans = scan_all_spans(text);
    // 参与边界处理的 span：语义原子 + 美元定界的 LaTeX 数学（结构保护）。
    let relevant: Vec<(usize, usize, SpanKind)> = spans
        .iter()
        .filter_map(|span| match span.kind {
            SpanKind::Measurement
            | SpanKind::Temperature
            | SpanKind::ScientificUnit
            | SpanKind::MathExpression
            | SpanKind::LatexMath => Some((span.start, span.end, span.kind)),
            _ => None,
        })
        .collect();

    let mut edits = Vec::new();
    for (start, end, kind) in relevant {
        // 数字|单位 拆分仅适用于非温度、非百分号类计量单位；
        // 温度与百分号类保持「数字紧贴符号」，边界空格交给边缘处理。
        let mut percent_like = false;
        if matches!(kind, SpanKind::Measurement | SpanKind::ScientificUnit) {
            for token in scan_semantic_tokens(text) {
                if token.start != start || token.end != end {
                    continue;
                }
                let unit = &text[token.unit_start..token.end];
                percent_like = matches!(unit, "%" | "％" | "‰");
                if !percent_like
                    && token.kind != super::semantic_tokens::SemanticTokenKind::Temperature
                    && token.number_end == token.unit_start
                    && next_char_range(text, token.unit_start).is_some()
                    && !scan_math_expressions(text)
                        .iter()
                        .any(|&(math_start, math_end)| {
                            math_start <= token.start && token.number_end <= math_end
                        })
                {
                    edits.push(
                        TextEdit::new(
                            text,
                            token.unit_start,
                            token.unit_start,
                            " ",
                            EditPriority::SemanticAtomic,
                        )
                        .expect("semantic unit edit must use valid UTF-8 boundaries"),
                    );
                }
                break;
            }
        }

        // 边缘处理：span 与直接相邻汉字之间插入空格。对齐生产管线中
        // 数学表达式占位符补空格、温标符号规则与百分号-中文规则的行为；
        // 覆盖检查会吸收文本边界在同一位置的零宽插入，避免双空格。
        let edge_handled = matches!(
            kind,
            SpanKind::Temperature | SpanKind::MathExpression | SpanKind::LatexMath
        ) || percent_like;
        if edge_handled {
            if let Some((before_start, before_end, before)) = previous_char_range(text, start) {
                if classify(before) == CharKind::Cjk {
                    edits.push(
                        TextEdit::new(
                            text,
                            before_start,
                            before_end,
                            format!("{before} "),
                            EditPriority::SemanticAtomic,
                        )
                        .expect("semantic edge edit must use valid UTF-8 boundaries"),
                    );
                }
            }
            if let Some((after_start, after_end, after)) = next_char_range(text, end) {
                if classify(after) == CharKind::Cjk {
                    edits.push(
                        TextEdit::new(
                            text,
                            after_start,
                            after_end,
                            format!(" {after}"),
                            EditPriority::SemanticAtomic,
                        )
                        .expect("semantic edge edit must use valid UTF-8 boundaries"),
                    );
                }
            }
        }
    }

    // 文本边界（对齐 spacing.cjk-latin / spacing.cjk-number 的核心行为）：
    // Han↔Latin、Han↔Digit 直接相邻时插入空格；数学运算符 ↔ Han 边界归入
    // cjk-number（对应 break_han_math_boundaries）。以 grapheme cluster 为
    // 判定单位，且不插入任何结构/语义 span 内部；受规则选择门控。
    let text_boundary_edits = plan_text_boundary_edits(text, &spans, &edits, &gate);
    edits.extend(text_boundary_edits);

    // 扩展边界（对齐 cn_en_space 的 break_emphasis_boundaries /
    // break_superscript_unit_boundaries）：Markdown 单星强调片段与
    // Unicode 上标结尾科学单位片段，受 spacing.cjk-latin 门控。
    if gate.cjk_latin {
        let extended_edits = plan_extended_boundary_edits(text, &spans, &edits);
        edits.extend(extended_edits);
    }

    // Inline placeholder 边缘补空格（对齐 space_around_inline_placeholders，
    // 无条件执行）：行内代码 / 链接 / 行内 HTML / 化学式等非可编辑片段
    // 与任意非空白前字符、非空白非全角标点后字符之间插入空格。
    let inline_edge_edits = plan_inline_placeholder_edge_edits(text, &spans, &edits);
    edits.extend(inline_edge_edits);

    // 未闭合反引号 delimiter 仅保护反引号串本身（对齐 protect_inline_code），
    // 同样获得边缘补空格。
    let unclosed_edits = plan_unclosed_backtick_edits(text, &spans, &edits);
    edits.extend(unclosed_edits);

    arbitrate_edits(edits)
}

/// Inline placeholder 边缘补空格：
/// - before：span 前任意非空白字符 → 插入空格（含全角标点，如 `、Fe²⁺`）；
/// - after：span 后非空白且不属于全角标点排除集的字符 → 插入空格。
fn plan_inline_placeholder_edge_edits(
    text: &str,
    spans: &[TextSpan],
    existing: &[TextEdit],
) -> Vec<TextEdit> {
    const AFTER_EXCLUDED: &[char] = &[
        '，', '。', '；', '：', '！', '？', '、', '）', '】', '》', '」', '』',
    ];
    let mut edits: Vec<TextEdit> = Vec::new();
    for span in spans {
        if span.priority == SpanPriority::Editable {
            continue;
        }
        // 仅处理「行内」保护片段：块级结构（fenced/indented code、HTML block、
        // front matter 等）整行保持原样，不补边界空格。
        if !matches!(
            span.kind,
            SpanKind::InlineCode
                | SpanKind::MarkdownLink
                | SpanKind::InlineHtml
                | SpanKind::ChemicalFormula
        ) {
            continue;
        }

        fn covered(existing: &[TextEdit], edits: &[TextEdit], boundary: usize) -> bool {
            existing
                .iter()
                .chain(edits.iter())
                .any(|edit: &TextEdit| boundary >= edit.start && boundary <= edit.end)
        }

        if let Some((before_start, before_end, before)) = previous_char_range(text, span.start) {
            if before != ' ' && !covered(existing, &edits, span.start) {
                if let Ok(edit) = TextEdit::new(
                    text,
                    before_start,
                    before_end,
                    format!("{before} "),
                    EditPriority::Editable,
                ) {
                    edits.push(edit);
                }
            }
        }
        if let Some((after_start, after_end, after)) = next_char_range(text, span.end) {
            if after != ' '
                && !AFTER_EXCLUDED.contains(&after)
                && !covered(existing, &edits, span.end)
            {
                if let Ok(edit) = TextEdit::new(
                    text,
                    after_start,
                    after_end,
                    format!(" {after}"),
                    EditPriority::Editable,
                ) {
                    edits.push(edit);
                }
            }
        }
    }
    edits
}

/// 未闭合反引号 delimiter 的边缘补空格。
///
/// 生产 `protect_inline_code` 对未闭合 delimiter「仅保护反引号串本身」，
/// 因此该串同样获得 `\S` / 非-非 排除集的边缘空格。此处找出不属于任何
/// 已闭合行内代码 span 的反引号 run，套用同一规则。
fn plan_unclosed_backtick_edits(
    text: &str,
    spans: &[TextSpan],
    existing: &[TextEdit],
) -> Vec<TextEdit> {
    const AFTER_EXCLUDED: &[char] = &[
        '，', '。', '；', '：', '！', '？', '、', '）', '】', '》', '」', '』',
    ];
    let code_spans: Vec<&TextSpan> = spans
        .iter()
        .filter(|span| span.kind == SpanKind::InlineCode)
        .collect();
    let bytes = text.as_bytes();
    let mut edits: Vec<TextEdit> = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }
        let run_start = index;
        while index < bytes.len() && bytes[index] == b'`' {
            index += 1;
        }
        let run_end = index;
        // 属于已闭合行内代码或任何结构 span（fenced 围栏等）的反引号不处理。
        let inside_structure = spans.iter().any(|span| {
            span.priority == SpanPriority::OpaqueStructure
                && span.start < run_end
                && run_start < span.end
        });
        if inside_structure
            || code_spans
                .iter()
                .any(|span| span.start < run_end && run_start < span.end)
        {
            continue;
        }

        fn covered(existing: &[TextEdit], edits: &[TextEdit], boundary: usize) -> bool {
            existing
                .iter()
                .chain(edits.iter())
                .any(|edit: &TextEdit| boundary >= edit.start && boundary <= edit.end)
        }

        if let Some((before_start, before_end, before)) = previous_char_range(text, run_start) {
            if before != ' '
                && !covered(existing, &edits, run_start)
                && !covered(existing, &edits, before_start)
            {
                if let Ok(edit) = TextEdit::new(
                    text,
                    before_start,
                    before_end,
                    format!("{before} "),
                    EditPriority::Editable,
                ) {
                    edits.push(edit);
                }
            }
        }
        if let Some((after_start, after_end, after)) = next_char_range(text, index) {
            if after != ' ' && !AFTER_EXCLUDED.contains(&after) && !covered(existing, &edits, index)
            {
                if let Ok(edit) = TextEdit::new(
                    text,
                    after_start,
                    after_end,
                    format!(" {after}"),
                    EditPriority::Editable,
                ) {
                    edits.push(edit);
                }
            }
        }
    }
    edits
}

/// 扩展边界：Markdown 单星强调片段（`*word*`）与以 Unicode 上标结尾的
/// 科学单位片段（如 `mg·mL⁻¹`）同 CJK 直接相邻时插入空格。
///
/// 对齐生产 `break_emphasis_boundaries` / `break_superscript_unit_boundaries`：
/// - 强调片段边界为 CJK 或比较运算符 `<`/`>`/`=`，不拆 `a*b*c` 与 `**粗体**`；
/// - 上标单位片段为字母开头、可含 `·` 连接段、以上标字符结尾；
/// - 匹配落入结构/语义 span 内部时跳过；已被既有编辑覆盖的位置跳过。
fn plan_extended_boundary_edits(
    text: &str,
    spans: &[TextSpan],
    existing: &[TextEdit],
) -> Vec<TextEdit> {
    use std::sync::OnceLock;

    fn regexes() -> &'static (regex::Regex, regex::Regex, regex::Regex, regex::Regex) {
        static RE: OnceLock<(regex::Regex, regex::Regex, regex::Regex, regex::Regex)> =
            OnceLock::new();
        RE.get_or_init(|| {
            let cjk = r"\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}";
            let superscript_unit = format!(
                r"[A-Za-z][A-Za-z0-9]*(?:[·⋅][A-Za-z0-9]+)*[{}]+",
                "⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾"
            );
            (
                regex::Regex::new(&format!(r"([{cjk}])(\*[A-Za-z]+\*)"))
                    .expect("invalid emphasis-before regex"),
                regex::Regex::new(&format!(r"(\*[A-Za-z]+\*)([{cjk}<>=])"))
                    .expect("invalid emphasis-after regex"),
                regex::Regex::new(&format!(r"([{cjk}])({superscript_unit})"))
                    .expect("invalid superscript-before regex"),
                regex::Regex::new(&format!(r"({superscript_unit})([{cjk}])"))
                    .expect("invalid superscript-after regex"),
            )
        })
    }

    let (emphasis_before, emphasis_after, unit_before, unit_after) = regexes();
    let mut edits: Vec<TextEdit> = Vec::new();

    fn inside_span(spans: &[TextSpan], start: usize, end: usize) -> bool {
        spans
            .iter()
            .any(|span| span.start < end && start < span.end)
    }

    fn push_boundary(
        text: &str,
        boundary: usize,
        spans: &[TextSpan],
        existing: &[TextEdit],
        edits: &mut Vec<TextEdit>,
    ) {
        // 边界两侧字符任一落入非可编辑 span 即跳过；已被既有编辑覆盖的位置
        // 跳过（避免与语义/数学边缘编辑叠加为双空格）。
        if inside_span(spans, boundary.saturating_sub(1), boundary + 1) {
            return;
        }
        let already_covered = existing
            .iter()
            .chain(edits.iter())
            .any(|edit: &TextEdit| boundary >= edit.start && boundary <= edit.end);
        if !already_covered {
            if let Ok(edit) = TextEdit::new(text, boundary, boundary, " ", EditPriority::Editable) {
                edits.push(edit);
            }
        }
    }

    for captures in emphasis_before.captures_iter(text) {
        let group2 = captures.get(2).expect("capture 2 exists");
        push_boundary(text, group2.start(), spans, existing, &mut edits);
    }
    for captures in emphasis_after.captures_iter(text) {
        let group2 = captures.get(2).expect("capture 2 exists");
        push_boundary(text, group2.start(), spans, existing, &mut edits);
    }
    for captures in unit_before.captures_iter(text) {
        let group2 = captures.get(2).expect("capture 2 exists");
        push_boundary(text, group2.start(), spans, existing, &mut edits);
    }
    for captures in unit_after.captures_iter(text) {
        let group1 = captures.get(1).expect("capture 1 exists");
        push_boundary(text, group1.end(), spans, existing, &mut edits);
    }

    edits
}

/// 为 Han↔Latin / Han↔Digit 直接相邻边界生成零宽插入编辑。
///
/// `existing` 是已生成的语义/结构编辑；数学边界编辑以「字符替换」方式插空，
/// 可能与零宽插入落在同一位置，覆盖检查依赖它避免产生双空格。
fn plan_text_boundary_edits(
    text: &str,
    spans: &[TextSpan],
    existing: &[TextEdit],
    gate: &TextBoundaryGate,
) -> Vec<TextEdit> {
    const MATH_OPERATORS: &[char] = &['∂', '±', '×', '≈', '≤', '≥'];
    let mut edits = Vec::new();
    let units = units(text, BoundaryStrategy::Graphemes);
    for pair in units.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        let boundary = right.byte_start;
        let inside_span = spans
            .iter()
            .any(|span| span.start < boundary && boundary < span.end);
        if inside_span {
            continue;
        }
        // 数学运算符 ↔ Han 边界归入 cjk-number（对齐 break_han_math_boundaries）。
        let needs_space = gate.allows((left.script, right.script))
            || (gate.cjk_number
                && matches!(
                    (left.script, right.script),
                    (ScriptClass::Han, ScriptClass::Other) | (ScriptClass::Other, ScriptClass::Han)
                )
                && (MATH_OPERATORS.contains(&right.text.chars().next().unwrap_or_default())
                    || MATH_OPERATORS.contains(&left.text.chars().next().unwrap_or_default())));
        if needs_space {
            let already_covered = existing
                .iter()
                .any(|edit: &TextEdit| boundary >= edit.start && boundary <= edit.end);
            if !already_covered {
                if let Ok(edit) =
                    TextEdit::new(text, boundary, boundary, " ", EditPriority::Editable)
                {
                    edits.push(edit);
                }
            }
        }
    }
    edits
}

fn previous_char_range(text: &str, index: usize) -> Option<(usize, usize, char)> {
    text[..index]
        .char_indices()
        .next_back()
        .map(|(start, ch)| (start, index, ch))
}

/// 测试入口：对原文仅应用单位/数学语义边界编辑。
///
/// 该函数目前只覆盖单位/数学语义边界；标点/名词规则由
/// `apply_editable_rules` 通过同一 TextEdit 基础设施接入生产路径。随迁移推进，
/// 温标、全角标点清理和其余边界规则将逐步纳入编辑计划。
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn format_units_and_math_via_edits(text: &str, selection: &RuleSelection) -> String {
    apply_edits(
        text,
        &plan_semantic_boundary_edits_with_selection(text, selection),
    )
    .expect("semantic boundary edits must apply cleanly")
}

fn next_char_range(text: &str, index: usize) -> Option<(usize, usize, char)> {
    let ch = text[index..].chars().next()?;
    Some((index, index + ch.len_utf8(), ch))
}

#[cfg(test)]
mod tests {
    use super::super::model::RuleSelection;
    use super::{
        apply_edits, apply_protected_text_rules, arbitrate_edits, plan_semantic_boundary_edits,
        EditPriority, TextEdit,
    };

    fn only(key: &str) -> RuleSelection {
        RuleSelection::Only {
            keys: vec![key.to_string()],
        }
    }

    fn edit(
        text: &str,
        start: usize,
        end: usize,
        replacement: &str,
        priority: EditPriority,
    ) -> TextEdit {
        TextEdit::new(text, start, end, replacement, priority).unwrap()
    }

    #[test]
    fn rejects_invalid_and_non_char_boundary_ranges() {
        assert!(TextEdit::new("中文", 0, 1, "x", EditPriority::Editable).is_err());
        assert!(TextEdit::new("abc", 3, 2, "x", EditPriority::Editable).is_err());
        assert!(TextEdit::new("abc", 0, 4, "x", EditPriority::Editable).is_err());
    }

    #[test]
    fn opaque_edit_wins_over_inner_semantic_edit() {
        let text = "代码10μm继续";
        let edits = arbitrate_edits(vec![
            edit(text, 6, 10, "10 μm", EditPriority::SemanticAtomic),
            edit(
                text,
                0,
                text.len(),
                "代码10μm继续",
                EditPriority::OpaqueStructure,
            ),
        ]);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].priority, EditPriority::OpaqueStructure);
    }

    #[test]
    fn longer_same_priority_edit_wins() {
        let text = "abcdef";
        let edits = arbitrate_edits(vec![
            edit(text, 0, 3, "left", EditPriority::SemanticAtomic),
            edit(text, 0, 5, "long", EditPriority::SemanticAtomic),
        ]);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].replacement, "long");
    }

    #[test]
    fn non_overlapping_edits_apply_from_right_to_left() {
        let text = "中文abc数字";
        let edits = vec![
            edit(text, 0, 6, "文本", EditPriority::Editable),
            edit(text, 9, 15, "数字", EditPriority::Editable),
        ];
        assert_eq!(apply_edits(text, &edits).unwrap(), "文本abc数字");
    }

    #[test]
    fn overlapping_edits_are_rejected_before_application() {
        let text = "abcdef";
        let edits = vec![
            edit(text, 0, 3, "x", EditPriority::Editable),
            edit(text, 2, 5, "y", EditPriority::Editable),
        ];
        assert!(apply_edits(text, &edits).is_err());
    }

    #[test]
    fn semantic_plan_formats_units_and_math_boundaries_without_touching_code() {
        let text = "样品10μm且计算∂f/∂x很重要，代码`10μm $x$`继续";
        let edits = plan_semantic_boundary_edits(text);
        let output = apply_edits(text, &edits).unwrap();
        // 单位/数学/文本边界 + inline placeholder 边缘补空格；行内代码内部不动。
        assert_eq!(
            output,
            "样品 10 μm 且计算 ∂f/∂x 很重要，代码 `10μm $x$` 继续"
        );
    }

    #[test]
    fn semantic_plan_formats_temperature_and_text_boundaries() {
        let text = "样品25°C保存，4℃冷藏";
        let output = apply_edits(text, &plan_semantic_boundary_edits(text)).unwrap();
        // ASCII 温标：Han↔Digit 与 Latin↔Han 边界插空；Unicode 温标符号
        // （℃）经边缘处理与后续汉字之间插空——与生产管线行为一致。
        assert_eq!(output, "样品 25°C 保存，4℃ 冷藏");
    }

    /// FinalCleanup 迁移回归：全角标点两侧空格按行移除，多行各自独立生效。
    #[test]
    fn final_cleanup_removes_spaces_around_fullwidth_punct_per_line() {
        let selection = only(super::super::registry::keys::SPACING_NO_SPACE_AROUND_FW_PUNCT);
        let text = "你好， 世界！ 继续\n第二行 ：测试\n英文 line stays";
        let output = apply_protected_text_rules(text, &selection).unwrap();
        assert_eq!(output, "你好，世界！继续\n第二行：测试\n英文 line stays");
    }

    /// 幂等性：清理结果再跑一次不再变化。
    #[test]
    fn final_cleanup_is_idempotent() {
        let selection = only(super::super::registry::keys::SPACING_NO_SPACE_AROUND_FW_PUNCT);
        let text = "你好， 世界 ！ 继续";
        let once = apply_protected_text_rules(text, &selection).unwrap();
        let twice = apply_protected_text_rules(&once, &selection).unwrap();
        assert_eq!(once, twice);
    }

    /// 规则选择为 None 时清理必须完全不生效。
    #[test]
    fn final_cleanup_respects_selection_none() {
        let text = "你好， 世界";
        let output = apply_protected_text_rules(text, &RuleSelection::None).unwrap();
        assert_eq!(output, text);
    }

    /// 文本边界规则同样经 `apply_protected_text_rules` 生效（受保护文本路径）。
    #[test]
    fn text_boundary_rules_apply_via_protected_text_path() {
        let selection = only(super::super::registry::keys::SPACING_CJK_LATIN);
        let output = apply_protected_text_rules("在GitHub上发布", &selection).unwrap();
        assert_eq!(output, "在 GitHub 上发布");
    }

    /// 结构边界规则（直角引号）同样经受保护文本路径生效。
    #[test]
    fn structure_boundary_rules_apply_via_protected_text_path() {
        let selection = only(super::super::registry::keys::PUNCT_CORNER_QUOTES);
        let output = apply_protected_text_rules("说“你好”", &selection).unwrap();
        assert_eq!(output, "说「你好」");
    }
}
