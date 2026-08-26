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

fn parse_fixture(file: &str, yaml: &str) -> Vec<(String, GoldenCase)> {
    serde_yaml::from_str::<Vec<GoldenCase>>(yaml)
        .unwrap_or_else(|error| panic!("failed to parse fixture {file}: {error}"))
        .into_iter()
        .map(|case| (file.to_string(), case))
        .collect()
}

fn load_passing_golden_cases() -> Vec<(String, GoldenCase)> {
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
        (
            "measurements.yaml",
            include_str!("../../tests/fixtures/measurements.yaml"),
        ),
        (
            "mathematical-symbols.yaml",
            include_str!("../../tests/fixtures/mathematical-symbols.yaml"),
        ),
        (
            "markdown-protection.yaml",
            include_str!("../../tests/fixtures/markdown-protection.yaml"),
        ),
        (
            "complex-compositions.yaml",
            include_str!("../../tests/fixtures/complex-compositions.yaml"),
        ),
        (
            "rule-interactions.yaml",
            include_str!("../../tests/fixtures/rule-interactions.yaml"),
        ),
        (
            "newline-and-idempotence.yaml",
            include_str!("../../tests/fixtures/newline-and-idempotence.yaml"),
        ),
    ]
    .into_iter()
    .flat_map(|(file, yaml)| parse_fixture(file, yaml))
    .collect()
}

fn load_pending_baseline_cases() -> Vec<(String, GoldenCase)> {
    [
        (
            "markdown-inline.yaml",
            include_str!("../../tests/fixtures/markdown-inline.yaml"),
        ),
        (
            "markdown-blocks.yaml",
            include_str!("../../tests/fixtures/markdown-blocks.yaml"),
        ),
        (
            "punctuation-contexts.yaml",
            include_str!("../../tests/fixtures/punctuation-contexts.yaml"),
        ),
        (
            "structure-precedence.yaml",
            include_str!("../../tests/fixtures/structure-precedence.yaml"),
        ),
    ]
    .into_iter()
    .flat_map(|(file, yaml)| parse_fixture(file, yaml))
    .collect()
}

#[test]
fn golden_fixtures_match_expected_output() {
    let cases = load_passing_golden_cases();
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
    let cases = load_passing_golden_cases();
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
fn pending_baselines_are_loadable_and_expose_unimplemented_behavior() {
    let cases = load_pending_baseline_cases();
    assert!(
        !cases.is_empty(),
        "pending baseline suite must not be empty"
    );

    let mismatches: Vec<String> = cases
        .iter()
        .filter_map(|(file, case)| {
            let request = FormatRequest {
                text: case.input.clone(),
                selection: case.selection.clone(),
            };
            let actual = format_text(&request).unwrap_or_else(|error| {
                panic!("fixture {file} / {} failed to format: {error}", case.name)
            });
            (actual != case.expected).then(|| {
                format!(
                    "{file} / {}\n  expected: {:?}\n  actual:   {:?}",
                    case.name, case.expected, actual
                )
            })
        })
        .collect();

    assert!(
        !mismatches.is_empty(),
        "pending baseline unexpectedly has no current gaps; review and promote it to passing fixtures"
    );
}

#[test]
fn passing_golden_fixtures_are_idempotent() {
    for (file, case) in load_passing_golden_cases() {
        let request = FormatRequest {
            text: case.input,
            selection: case.selection,
        };
        let once = format_text(&request)
            .unwrap_or_else(|error| panic!("fixture {file} / {} failed: {error}", case.name));
        let twice = format_text(&FormatRequest {
            text: once.clone(),
            selection: request.selection,
        })
        .unwrap_or_else(|error| panic!("fixture {file} / {} failed twice: {error}", case.name));
        assert_eq!(
            once, twice,
            "fixture {file} / {} is not idempotent",
            case.name
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
fn registry_execution_order_is_phase_explicit_and_stable() {
    let ordered = execution_rules();
    let keys: Vec<&str> = ordered.iter().map(|rule| rule.key()).collect();
    assert_eq!(
        keys,
        vec![
            keys::PUNCT_NO_REPETITION,
            keys::PUNCT_FULLWIDTH_CJK,
            keys::TEXT_HALFWIDTH_DIGITS,
            keys::TEXT_ASCII_PUNCT_IN_LATIN,
            keys::NAMING_PROPER_NOUNS,
            keys::NAMING_EXPAND_ABBREVIATIONS,
            keys::SPACING_AROUND_LINKS,
            keys::PUNCT_CORNER_QUOTES,
            keys::SPACING_CJK_LATIN,
            keys::SPACING_CJK_NUMBER,
            keys::SPACING_NUMBER_UNIT,
            keys::SPACING_TEMPERATURE_CJK,
            keys::SPACING_NO_SPACE_AROUND_FW_PUNCT,
        ]
    );
    assert!(ordered
        .windows(2)
        .all(|pair| pair[0].phase <= pair[1].phase));
}

#[test]
fn registry_dependency_graph_is_valid() {
    let ordered = execution_rules();
    for (index, rule) in ordered.iter().enumerate() {
        for dependency in rule.before {
            let target = ordered
                .iter()
                .position(|candidate| candidate.key() == *dependency);
            assert!(target.is_some_and(|target| index < target));
        }
        for dependency in rule.after {
            let target = ordered
                .iter()
                .position(|candidate| candidate.key() == *dependency);
            assert!(target.is_some_and(|target| target < index));
        }
    }
}

#[test]
fn registry_dependency_graph_rejects_unknown_and_cyclic_edges() {
    use super::registry::{resolve_execution_order, RuleDef, RulePhase};

    fn test_rule(
        key: &'static str,
        before: &'static [&'static str],
        after: &'static [&'static str],
    ) -> RuleDef {
        RuleDef {
            meta: RuleMeta {
                key: key.to_string(),
                section: "test".to_string(),
                name: key.to_string(),
                disputed: false,
                default: false,
            },
            phase: RulePhase::TextBoundary,
            before,
            after,
            legacy: &[],
            apply: super::rule_impls::cn_en_space,
        }
    }

    let unknown = [test_rule("a", &["missing"], &[])];
    let unknown_error = resolve_execution_order(&unknown)
        .err()
        .expect("unknown dependency must fail");
    assert!(unknown_error.contains("unknown rule dependency"));

    let cyclic = [test_rule("a", &["b"], &[]), test_rule("b", &["a"], &[])];
    let cycle_error = resolve_execution_order(&cyclic)
        .err()
        .expect("cycle must fail");
    assert!(cycle_error.contains("cyclic rule dependency"));

    let duplicate = [test_rule("a", &[], &[]), test_rule("a", &[], &[])];
    let duplicate_error = resolve_execution_order(&duplicate)
        .err()
        .expect("duplicate key must fail");
    assert!(duplicate_error.contains("duplicate rule key"));
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
fn formats_unicode_and_compound_measurements_without_word_false_positives() {
    assert_eq!(
        format_text(&req("薄膜厚度10μm，孔径5μm")).unwrap(),
        "薄膜厚度 10 μm，孔径 5 μm"
    );
    assert_eq!(
        format_text(&req("薄膜厚度10µm，晶格常数3Å，间距2Å")).unwrap(),
        "薄膜厚度 10 µm，晶格常数 3 Å，间距 2 Å"
    );
    assert_eq!(
        format_text(&req("电阻10kΩ，阻抗50Ω")).unwrap(),
        "电阻 10 kΩ，阻抗 50 Ω"
    );
    assert_eq!(
        format_text(&req("浓度30mg·mL⁻¹，密度2kg·m⁻³")).unwrap(),
        "浓度 30 mg·mL⁻¹，密度 2 kg·m⁻³"
    );
    assert_eq!(
        format_text(&req("浓度30mg/mL，流速2m/s，密度2kg/m³，浓度4mol/L")).unwrap(),
        "浓度 30 mg/mL，流速 2 m/s，密度 2 kg/m³，浓度 4 mol/L"
    );
    assert_eq!(
        format_text(&req("25°C保存，68°F运输")).unwrap(),
        "25°C 保存，68°F 运输"
    );
    assert_eq!(
        format_text(&req("第10chapter开始，参数为2alpha与3beta")).unwrap(),
        "第 10chapter 开始，参数为 2alpha 与 3beta"
    );
}

#[test]
fn number_unit_rule_keeps_chemical_formulas_protected() {
    assert_eq!(
        super::rule_impls::digit_unit_space("Fe²⁺、SO₄²⁻与10μm样品"),
        "Fe²⁺、SO₄²⁻与10 μm样品"
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
fn rule_selection_key_order_does_not_change_pipeline_order() {
    let text = "使用github登录，样品10μm在25°C保存";
    let forward = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::Only {
            keys: vec![
                keys::NAMING_PROPER_NOUNS.to_string(),
                keys::SPACING_CJK_LATIN.to_string(),
                keys::SPACING_CJK_NUMBER.to_string(),
                keys::SPACING_NUMBER_UNIT.to_string(),
                keys::SPACING_TEMPERATURE_CJK.to_string(),
            ],
        },
    };
    let reverse = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::Only {
            keys: vec![
                keys::SPACING_TEMPERATURE_CJK.to_string(),
                keys::SPACING_NUMBER_UNIT.to_string(),
                keys::SPACING_CJK_NUMBER.to_string(),
                keys::SPACING_CJK_LATIN.to_string(),
                keys::NAMING_PROPER_NOUNS.to_string(),
            ],
        },
    };
    assert_eq!(
        format_text(&forward).unwrap(),
        format_text(&reverse).unwrap()
    );
}

#[test]
fn representative_rule_compositions_are_idempotent() {
    let cases = [
        (
            "使用github登录，样品10μm在25°C保存",
            vec![
                keys::NAMING_PROPER_NOUNS,
                keys::SPACING_CJK_LATIN,
                keys::SPACING_CJK_NUMBER,
                keys::SPACING_NUMBER_UNIT,
                keys::SPACING_TEMPERATURE_CJK,
            ],
        ),
        (
            "请看[GitHub](https://example.com)然后继续5000元",
            vec![
                keys::SPACING_AROUND_LINKS,
                keys::SPACING_CJK_LATIN,
                keys::SPACING_CJK_NUMBER,
            ],
        ),
        (
            "使用github！！，价格为１０μm",
            vec![
                keys::NAMING_PROPER_NOUNS,
                keys::TEXT_HALFWIDTH_DIGITS,
                keys::PUNCT_NO_REPETITION,
                keys::SPACING_NUMBER_UNIT,
                keys::SPACING_CJK_NUMBER,
            ],
        ),
    ];

    for (text, selected) in cases {
        let request = FormatRequest {
            text: text.to_string(),
            selection: RuleSelection::Only {
                keys: selected.iter().map(|key| (*key).to_string()).collect(),
            },
        };
        let once = format_text(&request).unwrap();
        let twice = format_text(&FormatRequest {
            text: once.clone(),
            selection: request.selection.clone(),
        })
        .unwrap();
        assert_eq!(once, twice, "composition is not idempotent: {text}");
    }
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
    let long_inline_math = format!("长公式${}$后续GitHub", "x".repeat(301));
    assert_eq!(
        format_text(&req(&long_inline_math)).unwrap(),
        format!("长公式 ${}$ 后续 GitHub", "x".repeat(301))
    );
    assert_eq!(
        format_text(&req(r"公式$a\$b+c$很重要")).unwrap(),
        r"公式 $a\$b+c$ 很重要"
    );
    assert_eq!(
        format_text(&req(r"偶数\\$x$继续GitHub")).unwrap(),
        r"偶数\\$x$ 继续 GitHub"
    );
    assert_eq!(
        format_text(&req(r"奇数\$x$继续GitHub")).unwrap(),
        r"奇数\$x$继续 GitHub"
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
        format_text(&req("中文``a ` b``后续GitHub")).unwrap(),
        "中文 ``a ` b`` 后续 GitHub"
    );
    assert_eq!(
        format_text(&req("中文```a `` b```后续GitHub")).unwrap(),
        "中文 ```a `` b``` 后续 GitHub"
    );
    assert_eq!(
        format_text(&req("中文``未闭合GitHub")).unwrap(),
        "中文 `` 未闭合 GitHub"
    );
    assert_eq!(
        format_text(&req(
            "请看[GitHub链接](https://example.com/a;b?x=$1|y)然后继续"
        ))
        .unwrap(),
        "请看 [GitHub链接](https://example.com/a;b?x=$1|y) 然后继续"
    );
    assert_eq!(
        format_text(&req(
            "请看[文档](https://example.com/foo_(bar_(baz)))然后继续"
        ))
        .unwrap(),
        "请看 [文档](https://example.com/foo_(bar_(baz))) 然后继续"
    );
    assert_eq!(
        format_text(&req("图片![示例](img/foo_(small).png)结束GitHub")).unwrap(),
        "图片 ![示例](img/foo_(small).png) 结束 GitHub"
    );
    assert_eq!(
        format_text(&req("未闭合[文档](https://example.com/foo_(bar)继续GitHub")).unwrap(),
        "未闭合[文档]（ https://example.com/foo_(bar)继续GitHub"
    );
    assert_eq!(
        format_text(&req(r#"图片![alt text](image/path.png "title")很好"#)).unwrap(),
        r#"图片 ![alt text](image/path.png "title") 很好"#
    );

    let html_block = "<div class=\"notice\">\n在GitHub上发布5000元\n</div>\n正文在GitHub上发布";
    assert_eq!(
        format_text(&req(html_block)).unwrap(),
        "<div class=\"notice\">\n在GitHub上发布5000元\n</div>\n正文在 GitHub 上发布"
    );

    assert_eq!(
        format_text(&req("行内<span>GitHub</span>继续")).unwrap(),
        "行内 <span>GitHub</span> 继续"
    );
    assert_eq!(
        format_text(&req(
            r#"点击<a href="https://example.com/a>b">GitHub</a>继续"#
        ))
        .unwrap(),
        r#"点击 <a href="https://example.com/a>b">GitHub</a> 继续"#
    );
    assert_eq!(
        format_text(&req("换行<br/>继续GitHub")).unwrap(),
        "换行 <br/> 继续 GitHub"
    );
    assert_eq!(
        format_text(&req("比较a < b且x > y继续GitHub")).unwrap(),
        "比较 a < b 且 x > y 继续 GitHub"
    );
    assert_eq!(
        format_text(&req("未闭合<span>GitHub继续")).unwrap(),
        "未闭合 <span> GitHub 继续"
    );
    assert_eq!(
        format_text(&req("在\\*此处\\*粗体GitHub发布")).unwrap(),
        "在\\*此处\\*粗体 GitHub 发布"
    );
    assert_eq!(
        format_text(&req("转义\\*标记\\*和\\_强调\\_继续GitHub")).unwrap(),
        "转义\\*标记\\*和\\_强调\\_继续 GitHub"
    );
    assert_eq!(
        format_text(&req(r"路径是 <windows-user-home>，价格是\$100")).unwrap(),
        r"路径是 <windows-user-home>，价格是\$100"
    );

    let hard_break_spaces = "第一行GitHub  \n第二行5000元继续";
    assert_eq!(
        format_text(&req(hard_break_spaces)).unwrap(),
        "第一行 GitHub  \n第二行 5000 元继续"
    );
    let hard_break_backslash = "第一行GitHub\\\n第二行5000元继续";
    assert_eq!(
        format_text(&req(hard_break_backslash)).unwrap(),
        "第一行 GitHub\\\n第二行 5000 元继续"
    );
    assert_eq!(
        format_text(&req("普通行尾空格 GitHub ")).unwrap(),
        "普通行尾空格 GitHub "
    );

    let html_comment = "文本\n<!-- 注释GitHub 5000元\n第二行10cm -->\n结束";
    assert_eq!(format_text(&req(html_comment)).unwrap(), html_comment);

    let front_matter = "---\ntitle: 在GitHub上发布\ncount: 5000元\n---\n正文在GitHub上发布";
    assert_eq!(
        format_text(&req(front_matter)).unwrap(),
        "---\ntitle: 在GitHub上发布\ncount: 5000元\n---\n正文在 GitHub 上发布"
    );

    let bom_front_matter = "\u{FEFF}---\ntitle: 在GitHub上发布\n---\n正文在GitHub上发布";
    assert_eq!(
        format_text(&req(bom_front_matter)).unwrap(),
        "\u{FEFF}---\ntitle: 在GitHub上发布\n---\n正文在 GitHub 上发布"
    );

    // 文档中部的水平分隔线不是 front matter，后续文本仍可格式化。
    let non_front_matter = "正文\n---\ntitle: 在GitHub上发布";
    assert_eq!(
        format_text(&req(non_front_matter)).unwrap(),
        "正文\n---\ntitle: 在 GitHub 上发布"
    );

    let table = "| 标题GitHub | 数值5000元 |\n| :---: | ---: |\n| 内容GitHub | 价格5000元 |";
    assert_eq!(
        format_text(&req(table)).unwrap(),
        "| 标题 GitHub | 数值 5000 元 |\n| :---: | ---: |\n| 内容 GitHub | 价格 5000 元 |"
    );

    let non_table_separator = "正文---|---正文";
    assert_eq!(
        format_text(&req(non_table_separator)).unwrap(),
        "正文---|---正文"
    );

    let reference_definition =
        "查看[官网][home]然后继续\n\n  [home]: <https://example.com/a;b> \"官网\"";
    assert_eq!(
        format_text(&req(reference_definition)).unwrap(),
        "查看 [官网][home] 然后继续\n\n  [home]: <https://example.com/a;b> \"官网\""
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
        "采用双尾非配对 Student’s *t*检验，以*p*<0.05为显著差异。",
        "结果a*b*c保持不变",
        "AC磁场，且30 mg·mL⁻¹比10 mg·mL⁻¹作用更强，旋转DC磁场下。",
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

#[test]
fn formats_superscript_unit_and_acronym_boundaries() {
    let src = "AC磁场，且30 mg\u{b7}mL\u{207b}\u{b9}比10 mg\u{b7}mL\u{207b}\u{b9}作用更强，旋转DC磁场下。";
    println!("ACTUAL=[{}]", format_text(&req(src)).unwrap());
    assert_eq!(
        format_text(&req(src)).unwrap(),
        "AC 磁场，且 30 mg\u{b7}mL\u{207b}\u{b9} 比 10 mg\u{b7}mL\u{207b}\u{b9} 作用更强，旋转 DC 磁场下。"
    );
}
/// roadmap §5：新旧边界策略在既有黄金样例输入上输出必须一致；
/// grapheme 策略的新行为只体现在 unicode-boundaries.yaml 中。
#[test]
fn spacing_rules_grapheme_strategy_matches_legacy_on_golden_inputs() {
    use super::rule_impls::{cn_digit_space_with, cn_en_space_with};
    use super::unicode_boundaries::BoundaryStrategy;

    for (file, case) in load_passing_golden_cases() {
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

/// 编辑计划路径与生产 placeholder 路径的逐例对照（roadmap §5.7）。
///
/// 覆盖全部稳定黄金 fixture；新路径尚未复刻的能力差异被显式冻结在
/// `PENDING_DIFFS` 中；随 R3 迁移把更多规则纳入编辑计划，差异应单调减少，
/// 清单中的案例消除后必须同步移除。
#[test]
fn edit_plan_path_matches_placeholder_pipeline_on_stable_fixtures() {
    use super::edit_plan::format_units_and_math_via_edits;

    // 全量稳定 fixture 对照基线（roadmap §5.7）：语义边界类能力已完全一致；
    // 剩余 35 例差异对应编辑计划尚未复刻的四类生产能力，随 R3 迁移逐项消除：
    // 1) 非边界规则（标点规范化、名词规范化、直角引号、链接空格等逐行规则）；
    // 2) 全角标点清理（spacing.no-space-around-fw-punct，FinalCleanup 阶段）；
    // 3) cn_en_space 扩展边界：Markdown 单星强调片段、Unicode 上标结尾单位片段；
    // 4) 结构保护的占位符补空格/还原行为（行内代码、链接、HTML block 等周边空格）。
    const PENDING_DIFFS: &[&str] = &[
        // —— spacing.yaml ——
        "spacing.yaml / 全角标点与其他字符之间不加空格",
        "spacing.yaml / 上标单位片段与中文及缩写之间增加空格",
        "spacing.yaml / 默认规则处理含上标单位的实验列表",
        // —— punctuation.yaml（标点规范化规则未迁移）——
        "punctuation.yaml / 不重复使用标点符号",
        "punctuation.yaml / 使用全角中文标点",
        "punctuation.yaml / 数字使用半角字符",
        "punctuation.yaml / 英文整句使用半角标点",
        "punctuation.yaml / 简体中文使用直角引号",
        // —— naming-and-links.yaml（名词规则与链接空格未迁移）——
        "naming-and-links.yaml / 专有名词使用正确的大小写",
        "naming-and-links.yaml / 不要使用不地道的缩写",
        // —— protection.yaml / markdown-protection.yaml（结构保护还原未迁移）——
        // 差异细分为三类（已通过对照输出确认，2026-08-26）：
        // a) inline placeholder 周边 CJK 空格：✅ 已实现
        //    （edit_plan.rs::plan_inline_placeholder_edge_edits + 新增 InlineHtml span）；
        // b) HTML block 的 span 覆盖缺口：✅ 已修复
        //    （scan_html_block_spans 终点计算漏中间行；回归测试 spans.rs::
        //    html_block_span_covers_interior_lines）；
        // c) 未闭合结构的特殊还原（如 `[文档]（` 全角括号替换）。
        "markdown-protection.yaml / 未闭合链接不吞掉后续正文",
        "markdown-protection.yaml / 未闭合反引号不吞掉后续正文",
        "markdown-protection.yaml / 引用式链接定义保持完整而正文继续格式化",
        // —— unicode-boundaries.yaml（组合场景）——
        // —— 复合场景 ——
        "selection-and-regressions.yaml / 全选模式执行全部规则",
        "rule-interactions.yaml / 专有名词与中英文边界共同处理",
        "rule-interactions.yaml / 全角数字与数字单位共同处理",
        "rule-interactions.yaml / 中英文数字单位和温标共同处理",
    ];

    let all_cases = load_passing_golden_cases();

    let mut observed_diffs: Vec<String> = Vec::new();
    let mut diff_details: Vec<String> = Vec::new();
    for (file, case) in &all_cases {
        {
            let request = FormatRequest {
                text: case.input.clone(),
                selection: case.selection.clone(),
            };
            let pipeline_output = format_text(&request)
                .unwrap_or_else(|error| panic!("fixture {file} / {} failed: {error}", case.name));
            let edit_plan_output = format_units_and_math_via_edits(&case.input, &case.selection);

            // 单向不变量（无误报）：生产管线未修改的输入，编辑计划也不得修改。
            if pipeline_output == case.input {
                assert_eq!(
                    edit_plan_output, case.input,
                    "edit plan changed {} / {} but the placeholder pipeline would not",
                    file, case.name
                );
            }

            if pipeline_output != edit_plan_output {
                observed_diffs.push(format!("{file} / {}", case.name));
                diff_details.push(format!(
                    "{} / {}\n  input:    {:?}\n  pipeline: {:?}\n  editplan: {:?}",
                    file, case.name, case.input, pipeline_output, edit_plan_output
                ));
            }
        }
    }

    let unexpected: Vec<&String> = observed_diffs
        .iter()
        .filter(|name| !PENDING_DIFFS.contains(&name.as_str()))
        .collect();
    let stale: Vec<&&str> = PENDING_DIFFS
        .iter()
        .filter(|pending| !observed_diffs.iter().any(|name| name == *pending))
        .collect();
    assert!(
        unexpected.is_empty() && stale.is_empty(),
        "PENDING_DIFFS is out of date;\nunexpected diffs: {unexpected:?}\nstale entries: {stale:?}\n\ndiff details:\n{}",
        diff_details
            .iter()
            .filter(|detail| unexpected.iter().any(|name| detail.contains(name.as_str())))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
