// 引擎属性回归测试（不引入外部属性测试依赖）。
//
// 使用确定性 xorshift 伪随机生成器组合真实语料片段（CJK、拉丁、数字、
// 单位、emoji ZWJ、组合附加符、Markdown/LaTeX/URL、未闭合结构），
// 覆盖 roadmap P1 的以下不变量：
// 1. 幂等性：format(format(x)) == format(x)，任意规则选择下成立；
// 2. 任意规则选择不 panic，且输出始终是合法 UTF-8 文本（String 保证），
//    不会出现 char boundary panic；
// 3. CRLF/CR 输入按原换行风格还原；
// 4. 受保护结构（fenced code、行内代码、URL、LaTeX）内容不被改写；
// 5. legacy key 归一化：旧中文 key 迁移、未知 key 丢弃，且归一化幂等。

use chinese_copywriting_formatter_lib::engine::{self, FormatRequest, RuleSelection};

/// 确定性伪随机数生成器（xorshift64*）；不依赖外部 crate。
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// 语料片段池：覆盖规则引擎需要区分的全部边界类别。
const SEGMENTS: &[&str] = &[
    // CJK / 标点
    "中文文案",
    "在LeanCloud上，花了5000元",
    "你好。",
    "！？",
    "（括号）",
    // 拉丁 / 数字 / 单位
    "hello world",
    "5000",
    "3cm²",
    "10 mg/mL",
    "-20 ℃",
    "Ω",
    // emoji / 组合字符（grapheme 边界）
    "👩‍👩‍👧‍👦",
    "é",
    "🏳️‍🌈",
    "𠀀",
    // Markdown / 保护结构
    "# 标题\n\n正文段落。\n",
    "`inline code`",
    "```rust\nfn main() {}\n```",
    "[链接](https://example.com/a?b=1&c=2)",
    "<https://example.com>",
    "user@example.com",
    "---
title: 标题
---
",
    "| a | b |\n| --- | --- |\n| 1 | 2 |",
    // LaTeX
    "$E=mc^2$",
    "\\(x^2 + y^2 = z^2\\)",
    "\\[\\int_0^1 x dx\\]",
    // 化学式
    "Fe²⁺",
    "CuSO₄·5H₂O",
    // 未闭合结构（宁漏格式化，不破坏结构）
    "未闭合 [链接",
    "未闭合 `反引号",
    "未闭合 \\(公式",
    "**粗体**",
];

fn build_corpus(seed: u64, segments: usize) -> String {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut text = String::new();
    for _ in 0..segments {
        text.push_str(SEGMENTS[rng.below(SEGMENTS.len())]);
        match rng.below(4) {
            0 => text.push('\n'),
            1 => text.push_str("\r\n"),
            2 => text.push(' '),
            _ => text.push_str("\n\n"),
        }
    }
    text
}

fn selections() -> Vec<RuleSelection> {
    vec![
        RuleSelection::All,
        RuleSelection::Defaults,
        RuleSelection::None,
        RuleSelection::Only {
            keys: vec![
                "spacing.cjk-latin".into(),
                "spacing.cjk-number".into(),
                "punctuation.fullwidth-cjk".into(),
            ],
        },
    ]
}

fn format(text: &str, selection: &RuleSelection) -> String {
    engine::format_text(&FormatRequest {
        text: text.to_string(),
        selection: selection.clone(),
    })
    .expect("format_text 不应返回错误")
}

#[test]
fn formatting_is_idempotent_for_all_selections() {
    for seed in 1..=8u64 {
        let corpus = build_corpus(seed, 40);
        for selection in selections() {
            let once = format(&corpus, &selection);
            let twice = format(&once, &selection);
            assert_eq!(
                once, twice,
                "seed={seed} 时幂等性失败（selection: {selection:?}）"
            );
        }
    }
}

#[test]
fn arbitrary_selections_never_panic_and_preserve_emoji_clusters() {
    // ZWJ 家庭 emoji 与组合附加符不应被拆开（按 grapheme 边界处理）。
    let family = "中👩‍👩‍👧‍👦文";
    let output = format(family, &RuleSelection::All);
    assert!(output.contains("👩‍👩‍👧‍👦"), "ZWJ 家庭 emoji 被拆开：{output}");
    // 逐条单规则也不 panic。
    for rule in engine::rules() {
        let selection = RuleSelection::Only {
            keys: vec![rule.meta.key.clone()],
        };
        let corpus = build_corpus(42, 30);
        let _ = format(&corpus, &selection);
    }
}

#[test]
fn crlf_and_cr_are_preserved() {
    let lf = format("中文English 中文2", &RuleSelection::All);
    for newline in ["\r\n", "\r"] {
        let joined = ["中文English", "中文2"].join(newline);
        let output = format(&joined, &RuleSelection::All);
        assert!(
            output.contains(newline),
            "换行风格 {newline:?} 未被保留：{output:?}"
        );
        // LF 版本换行后内容应与多行版本逐行等价。
        let expected_lines: Vec<String> = joined
            .split(newline)
            .map(|line| format(line, &RuleSelection::All))
            .collect();
        assert_eq!(output.split(newline).collect::<Vec<_>>(), expected_lines);
    }
    let _ = lf;
}

#[test]
fn protected_structures_survive_formatting() {
    let cases: &[(&str, &str)] = &[
        ("```rust\nfn main() {}\n```", "fn main() {}"),
        ("`sp acing.test`", "sp acing.test"),
        (
            "https://example.com/a?b=1&c=2",
            "https://example.com/a?b=1&c=2",
        ),
        ("user@example.com", "user@example.com"),
        ("$E=mc^2$", "E=mc^2"),
        ("FeCl₂·4H₂O", "FeCl₂·4H₂O"),
    ];
    for (input, must_contain) in cases {
        let output = format(input, &RuleSelection::All);
        assert!(
            output.contains(must_contain),
            "受保护内容 `{must_contain}` 被改写：输入 {input:?} → 输出 {output:?}"
        );
    }
}

#[test]
fn legacy_key_normalization_is_stable() {
    // 旧中文 key 迁移为 stable key；重复归一化结果一致；未知 key 丢弃。
    let legacy = vec![
        "中英文之间需要增加空格".to_string(),
        "不重复使用标点符号".to_string(),
        "未知规则key".to_string(),
    ];
    let once = engine::normalize_rule_keys(&legacy);
    assert!(
        once.contains(&"spacing.cjk-latin".to_string()),
        "legacy key 未迁移：{once:?}"
    );
    assert!(
        !once.contains(&"未知规则key".to_string()),
        "未知 key 未被丢弃：{once:?}"
    );
    let twice = engine::normalize_rule_keys(&once);
    assert_eq!(once, twice, "归一化不幂等");
    // stable key 归一化后保持不变。
    let stable = vec!["spacing.cjk-latin".to_string()];
    assert_eq!(engine::normalize_rule_keys(&stable), stable);
}
