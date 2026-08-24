// engine/tests.rs —— 引擎单元测试（迁移自旧 rust_engine.rs，并新增
// 化学式识别、注册表扩展性与旧 key 迁移的回归测试）。

use super::*;

#[derive(serde::Deserialize)]
struct GoldenCase {
    name: String,
    selection: RuleSelection,
    input: String,
    expected: String,
}

fn req(text: &str) -> FormatRequest {
    FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::Defaults,
    }
}

use crate::engine::registry::keys;

fn load_golden_cases() -> Vec<(String, GoldenCase)> {
    [
        (
            "spacing.yaml",
            include_str!("../../tests/fixtures/spacing.yaml"),
        ),
        (
            "punctuation.yaml",
            include_str!("../../tests/fixtures/punctuation.yaml"),
        ),
        (
            "naming-and-links.yaml",
            include_str!("../../tests/fixtures/naming-and-links.yaml"),
        ),
        (
            "protection.yaml",
            include_str!("../../tests/fixtures/protection.yaml"),
        ),
        (
            "selection-and-regressions.yaml",
            include_str!("../../tests/fixtures/selection-and-regressions.yaml"),
        ),
        (
            "unicode-boundaries.yaml",
            include_str!("../../tests/fixtures/unicode-boundaries.yaml"),
        ),
    ]
    .into_iter()
    .flat_map(|(file, yaml)| {
        serde_yaml::from_str::<Vec<GoldenCase>>(yaml)
            .unwrap_or_else(|error| panic!("failed to parse fixture {file}: {error}"))
            .into_iter()
            .map(move |case| (file.to_string(), case))
    })
    .collect()
}

#[test]
fn golden_fixtures_match_expected_output() {
    let cases = load_golden_cases();
    assert!(!cases.is_empty(), "golden fixture suite must not be empty");

    for (file, case) in &cases {
        let request = FormatRequest {
            text: case.input.clone(),
            selection: case.selection.clone(),
        };
        let actual = format_text(&request).unwrap_or_else(|error| {
            panic!("fixture {file} / {} failed to format: {error}", case.name)
        });
        assert_eq!(
            actual, case.expected,
            "fixture {file} / {} produced unexpected output",
            case.name
        );
    }
}

#[test]
fn golden_fixtures_cover_every_registered_rule() {
    let cases = load_golden_cases();
    let covered: std::collections::HashSet<&str> = cases
        .iter()
        .flat_map(|(_, case)| match &case.selection {
            RuleSelection::Only { keys } => keys.iter().map(String::as_str).collect(),
            _ => Vec::new(),
        })
        .collect();

    for rule in rules() {
        assert!(
            covered.contains(rule.key()),
            "no golden fixture covers registered rule {}",
            rule.key()
        );
    }
}

#[test]
fn registry_contains_migrated_rules_with_defaults() {
    let all = rules();
    // 历史 12 条规则及温标空格规则均已注册。
    assert_eq!(all.len(), 13);
    assert_eq!(enabled_defaults().len(), 9);
    let disabled: Vec<_> = all
        .iter()
        .filter(|r| !r.meta.default)
        .map(|r| r.key().to_string())
        .collect();
    assert_eq!(
        disabled,
        vec![
            keys::NAMING_PROPER_NOUNS,
            keys::NAMING_EXPAND_ABBREVIATIONS,
            keys::SPACING_AROUND_LINKS,
            keys::PUNCT_CORNER_QUOTES,
        ]
    );
    for rule in all {
        if rule.meta.disputed {
            assert!(!rule.meta.default);
        }
    }
    // 稳定机器 key：不再使用中文展示名作为技术标识。
    assert!(all.iter().all(|r| r
        .key()
        .chars()
        .all(|c| c.is_ascii() && (c.is_ascii_alphanumeric() || c == '.' || c == '-'))));
}

#[test]
fn formats_basic_copywriting_sample() {
    assert_eq!(
        format_text(&req("在LeanCloud上，花了5000元")).unwrap(),
        "在 LeanCloud 上，花了 5000 元"
    );
}

#[test]
fn formats_digit_units_and_percentages() {
    assert_eq!(
        format_text(&req("宽带有 10Gbps")).unwrap(),
        "宽带有 10 Gbps"
    );
    assert_eq!(
        format_text(&req("SSD 一共有 20TB")).unwrap(),
        "SSD 一共有 20 TB"
    );
    assert_eq!(
        format_text(&req("角度为 90 ° 的角")).unwrap(),
        "角度为 90° 的角"
    );
    assert_eq!(
        format_text(&req("有 15 % 的 CPU")).unwrap(),
        "有 15% 的 CPU"
    );
    assert_eq!(
        format_text(&req("-20 ℃保存，32℉解冻")).unwrap(),
        "-20 ℃ 保存，32℉ 解冻"
    );
}

#[test]
fn formats_punctuation_and_proper_nouns() {
    assert_eq!(
        format_text(&req("德国队竟然战胜了巴西队！！")).unwrap(),
        "德国队竟然战胜了巴西队！"
    );
    assert_eq!(
        format_text(&req("只卖 １０００ 元")).unwrap(),
        "只卖 1000 元"
    );
    // 专有名词规则默认关闭：默认集不替换 github；显式启用后生效。
    assert_eq!(
        format_text(&req("使用 github 登录")).unwrap(),
        "使用 github 登录"
    );
    let mut enabled = enabled_defaults();
    enabled.push(keys::NAMING_PROPER_NOUNS.to_string());
    let with_nouns = FormatRequest {
        text: "使用 github 登录".to_string(),
        selection: RuleSelection::Only { keys: enabled },
    };
    assert_eq!(format_text(&with_nouns).unwrap(), "使用 GitHub 登录");
}

#[test]
fn disabled_rules_are_not_applied() {
    // 新架构：未启用的规则不执行（旧实现中基础空格规则始终强制执行）。
    let none = FormatRequest {
        text: "在LeanCloud上".to_string(),
        selection: RuleSelection::Only {
            keys: vec![keys::PUNCT_NO_REPETITION.to_string()],
        },
    };
    assert_eq!(format_text(&none).unwrap(), "在LeanCloud上");
}

#[test]
fn unknown_keys_are_safely_ignored() {
    let mixed = FormatRequest {
        text: "花了5000元".to_string(),
        selection: RuleSelection::Only {
            keys: vec![
                "不存在的规则".to_string(),
                keys::SPACING_CJK_NUMBER.to_string(),
            ],
        },
    };
    assert_eq!(format_text(&mixed).unwrap(), "花了 5000 元");
}

#[test]
fn explicit_rule_selection_modes_are_unambiguous() {
    let text = "在LeanCloud上，花了5000元！！";

    let all = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::All,
    };
    assert_eq!(
        format_text(&all).unwrap(),
        "在 LeanCloud 上，花了 5000 元！"
    );

    let defaults = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::Defaults,
    };
    assert_eq!(
        format_text(&defaults).unwrap(),
        "在 LeanCloud 上，花了 5000 元！"
    );

    let only = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::Only {
            keys: vec![keys::PUNCT_NO_REPETITION.to_string()],
        },
    };
    assert_eq!(format_text(&only).unwrap(), "在LeanCloud上，花了5000元！");

    let none = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::None,
    };
    assert_eq!(format_text(&none).unwrap(), text);
}

#[test]
fn legacy_keys_are_normalized_to_stable_keys() {
    let legacy = vec![
        "中英文之间需要增加空格".to_string(),
        "遇到完整的英文整句_特殊名词_其内容使用半角标点".to_string(),
        "已被删除的旧规则".to_string(),
    ];
    assert_eq!(
        normalize_rule_keys(&legacy),
        vec![
            keys::SPACING_CJK_LATIN.to_string(),
            keys::TEXT_ASCII_PUNCT_IN_LATIN.to_string(),
        ]
    );
    // 新 key 原样保留并去重。
    let dup = vec![
        keys::SPACING_CJK_LATIN.to_string(),
        keys::SPACING_CJK_LATIN.to_string(),
    ];
    assert_eq!(
        normalize_rule_keys(&dup),
        vec![keys::SPACING_CJK_LATIN.to_string()]
    );
}

#[test]
fn formats_protected_content() {
    // LaTeX / Markdown 保护。
    assert_eq!(
        format_text(&req("公式$E=mc^2$很重要")).unwrap(),
        "公式 $E=mc^2$ 很重要"
    );
    assert_eq!(
        format_text(&req(r"公式\( E=mc^2 \)很重要")).unwrap(),
        r"公式 \( E=mc^2 \) 很重要"
    );
    assert_eq!(
        format_text(&req(r"使用\frac{a}{b}计算")).unwrap(),
        r"使用 \frac{a}{b} 计算"
    );
    assert_eq!(format_text(&req(r"价格是\$100")).unwrap(), r"价格是\$100");

    let display_math = "如下：\n$$\nE=mc^2; github\n$$\n结束";
    assert_eq!(format_text(&req(display_math)).unwrap(), display_math);

    let latex_env = "如下：\n\\begin{align}\na&=b+c; github\n\\end{align}\n结束";
    assert_eq!(format_text(&req(latex_env)).unwrap(), latex_env);

    let fenced = "示例：\n```python\nprint('github; $x | y')\n```\n结束";
    assert_eq!(format_text(&req(fenced)).unwrap(), fenced);

    let indented = "命令：\n    npm install foo/bar; echo '$x|y'\n完成";
    assert_eq!(format_text(&req(indented)).unwrap(), indented);

    assert_eq!(
        format_text(&req("使用`a;b|c/$x`安装")).unwrap(),
        "使用 `a;b|c/$x` 安装"
    );
    assert_eq!(
        format_text(&req(
            "请看[GitHub链接](https://example.com/a;b?x=$1|y)然后继续"
        ))
        .unwrap(),
        "请看 [GitHub链接](https://example.com/a;b?x=$1|y) 然后继续"
    );
    assert_eq!(
        format_text(&req(r#"图片![alt text](image/path.png "title")很好"#)).unwrap(),
        r#"图片 ![alt text](image/path.png "title") 很好"#
    );
}

#[test]
fn chemical_formulas_are_recognized_as_whole_units() {
    use super::tokenizer::detect_chemical_formulas as detect;

    let spans = detect("铁离子Fe²⁺用于反应");
    assert_eq!(spans.len(), 1);
    assert_eq!(&"铁离子Fe²⁺用于反应"[spans[0].0..spans[0].1], "Fe²⁺");

    for sample in ["FeCl₂·4H₂O", "SO₄²⁻", "CuSO₄·5H₂O", "Fe³⁺", "Na⁺"] {
        let spans = detect(sample);
        assert_eq!(spans.len(), 1, "{sample}");
        assert_eq!(&sample[spans[0].0..spans[0].1], sample);
    }

    // 普通单词与不含特征的简单式子不被吞并（保守策略）。
    assert!(detect("GitHub TypeScript").is_empty());
    assert!(detect("H2O and CO2").is_empty());

    // 同文存在真实化学式（Fe²⁺）时，普通大写缩写不得被误判为化学式；
    // 否则 DA-PEG-DA 末尾的 DA 会被保护并在补空格阶段产生 `DA-PEG- DA`。
    let spans = detect("MIONPs、Fe²⁺或DA-PEG-DA/PEG");
    assert_eq!(spans.len(), 1);
    assert_eq!(
        &"MIONPs、Fe²⁺或DA-PEG-DA/PEG"[spans[0].0..spans[0].1],
        "Fe²⁺"
    );
}

#[test]
fn chemical_formulas_survive_formatting() {
    assert_eq!(
        format_text(&req("铁离子Fe²⁺用于反应")).unwrap(),
        "铁离子 Fe²⁺ 用于反应"
    );
    assert_eq!(
        format_text(&req("样品为FeCl₂·4H₂O，纯度99%")).unwrap(),
        "样品为 FeCl₂·4H₂O，纯度 99%"
    );
    // 全角数字转换、标点规则都不得改写化学式内部。
    assert_eq!(
        format_text(&req("电解质如SO₄²⁻溶液！！")).unwrap(),
        "电解质如 SO₄²⁻ 溶液！"
    );
}

#[test]
fn protected_cases_are_idempotent() {
    let cases = [
        "第一段\n\n第二段",
        "公式$E=mc^2$很重要",
        "示例：\n```\ngithub; $x | y\n```\n结束",
        r"路径是 <windows-user-home>，价格是\$100",
        "样品为FeCl₂·4H₂O，纯度99%",
        "铁离子Fe²⁺用于反应",
        "不与MIONPs、Fe²⁺或DA-PEG-DA/PEG接触",
        "𠀀Fe²⁺用于反应",
        "中文👨‍👩‍👧‍👦GitHub",
    ];
    for src in cases {
        let once = format_text(&req(src)).unwrap();
        assert_eq!(format_text(&req(&once)).unwrap(), once, "{src}");
    }
}

#[test]
fn preserves_newline_style() {
    assert_eq!(
        format_text(&req("在LeanCloud上\r\n\r\n花了5000元")).unwrap(),
        "在 LeanCloud 上\r\n\r\n花了 5000 元"
    );
}

/// UTF-8 回归：emoji、CJK 扩展区、全角字符、多行混合文本。
#[test]
fn handles_utf8_multibyte_and_emoji() {
    assert_eq!(
        format_text(&req("好👍！用GitHub提交")).unwrap(),
        "好👍！用 GitHub 提交"
    );
    assert_eq!(
        format_text(&req("古字𠀀在LeanCloud文档中")).unwrap(),
        "古字𠀀在 LeanCloud 文档中"
    );
    assert_eq!(
        format_text(&req("版本Ｖ２已发布！！")).unwrap(),
        "版本Ｖ2 已发布！"
    );
    assert_eq!(
        format_text(&req("第一行LeanCloud\n第二行5000元\n第三行👍")).unwrap(),
        "第一行 LeanCloud\n第二行 5000 元\n第三行👍"
    );
}

/// roadmap §5：新旧边界策略在既有黄金样例输入上输出必须一致；
/// grapheme 策略的新行为只体现在 unicode-boundaries.yaml 中。
#[test]
fn spacing_rules_grapheme_strategy_matches_legacy_on_golden_inputs() {
    use super::rule_impls::{cn_digit_space_with, cn_en_space_with};
    use super::unicode_boundaries::BoundaryStrategy;

    for (file, case) in load_golden_cases() {
        assert_eq!(
            cn_en_space_with(&case.input, BoundaryStrategy::LegacyChars),
            cn_en_space_with(&case.input, BoundaryStrategy::Graphemes),
            "cn_en_space diverged on {file} / {}",
            case.name
        );
        assert_eq!(
            cn_digit_space_with(&case.input, BoundaryStrategy::LegacyChars),
            cn_digit_space_with(&case.input, BoundaryStrategy::Graphemes),
            "cn_digit_space diverged on {file} / {}",
            case.name
        );
    }
}

/// 化学式检测不经过 Unicode 边界层：扩展区 B 文本中的化学式 span 保持不变。
#[test]
fn chemical_detection_unaffected_by_boundary_layer() {
    use super::tokenizer::detect_chemical_formulas as detect;

    let sample = "𠀀Fe²⁺用于反应";
    let spans = detect(sample);
    assert_eq!(spans.len(), 1);
    assert_eq!(&sample[spans[0].0..spans[0].1], "Fe²⁺");
}
