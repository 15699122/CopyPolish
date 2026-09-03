//! 内置用户工作流预设。
//!
//! 预设只组合稳定的规则选择、替换和字符转换字段，最终仍通过
//! `Preset::to_request` 进入同一个 Rust 格式化管线；这里不复制规则实现。

use super::model::{CharacterConversion, Preset, RuleSelection};
use super::registry::keys;

pub const COPYWRITING: &str = "copywriting";
pub const PDF_CLEANING: &str = "pdf-cleaning";
pub const TECHNICAL_DOCS: &str = "technical-docs";

/// 返回内置工作流预设，顺序也是前端/TUI 的稳定展示顺序。
pub fn presets() -> Vec<Preset> {
    vec![
        Preset {
            key: COPYWRITING.to_string(),
            name: "中文文案".to_string(),
            description: "使用默认排版规则，适合网页和产品文案。".to_string(),
            selection: RuleSelection::Defaults,
            replacements: Vec::new(),
            conversion: CharacterConversion::None,
        },
        Preset {
            key: PDF_CLEANING.to_string(),
            name: "PDF 清洗".to_string(),
            description: "适合从 PDF/CAJ 复制的文本：启用来源空白和引用角标清洗。".to_string(),
            selection: RuleSelection::Only {
                keys: vec![
                    keys::CLEANUP_REFERENCE_SQUARE.to_string(),
                    keys::CLEANUP_COLLAPSE_HORIZONTAL_SPACES.to_string(),
                    keys::CLEANUP_LIMIT_BLANK_LINES.to_string(),
                    keys::PUNCT_NO_REPETITION.to_string(),
                    keys::PUNCT_FULLWIDTH_CJK.to_string(),
                    keys::TEXT_HALFWIDTH_DIGITS.to_string(),
                    keys::TEXT_ASCII_PUNCT_IN_LATIN.to_string(),
                    keys::SPACING_CJK_LATIN.to_string(),
                    keys::SPACING_CJK_NUMBER.to_string(),
                    keys::SPACING_NUMBER_UNIT.to_string(),
                    keys::SPACING_TEMPERATURE_CJK.to_string(),
                    keys::SPACING_NO_SPACE_AROUND_FW_PUNCT.to_string(),
                ],
            },
            replacements: Vec::new(),
            conversion: CharacterConversion::None,
        },
        Preset {
            key: TECHNICAL_DOCS.to_string(),
            name: "技术文档".to_string(),
            description: "在默认排版基础上启用专有名词和缩写规范化。".to_string(),
            selection: RuleSelection::Only {
                keys: vec![
                    keys::PUNCT_NO_REPETITION.to_string(),
                    keys::PUNCT_FULLWIDTH_CJK.to_string(),
                    keys::TEXT_HALFWIDTH_DIGITS.to_string(),
                    keys::TEXT_ASCII_PUNCT_IN_LATIN.to_string(),
                    keys::NAMING_PROPER_NOUNS.to_string(),
                    keys::NAMING_EXPAND_ABBREVIATIONS.to_string(),
                    keys::SPACING_CJK_LATIN.to_string(),
                    keys::SPACING_CJK_NUMBER.to_string(),
                    keys::SPACING_NUMBER_UNIT.to_string(),
                    keys::SPACING_TEMPERATURE_CJK.to_string(),
                    keys::SPACING_NO_SPACE_AROUND_FW_PUNCT.to_string(),
                ],
            },
            replacements: Vec::new(),
            conversion: CharacterConversion::None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_presets_have_stable_keys_and_expand_to_requests() {
        let presets = presets();
        assert_eq!(
            presets
                .iter()
                .map(|preset| preset.key.as_str())
                .collect::<Vec<_>>(),
            vec![COPYWRITING, PDF_CLEANING, TECHNICAL_DOCS]
        );
        for preset in presets {
            let request = preset.to_request("示例".to_string());
            assert_eq!(request.text, "示例");
            assert_eq!(request.replacements, preset.replacements);
            assert_eq!(request.conversion, preset.conversion);
        }
    }

    #[test]
    fn pdf_and_technical_presets_include_their_workflow_rules() {
        let all = presets();
        let pdf = all
            .iter()
            .find(|preset| preset.key == PDF_CLEANING)
            .unwrap();
        let technical = all
            .iter()
            .find(|preset| preset.key == TECHNICAL_DOCS)
            .unwrap();
        match &pdf.selection {
            RuleSelection::Only { keys } => {
                assert!(keys.iter().any(|key| key == keys::CLEANUP_REFERENCE_SQUARE))
            }
            other => panic!("expected explicit PDF cleaning rules, got {other:?}"),
        }
        match &technical.selection {
            RuleSelection::Only { keys } => {
                assert!(keys.iter().any(|key| key == keys::NAMING_PROPER_NOUNS))
            }
            other => panic!("expected explicit technical document rules, got {other:?}"),
        }
    }
}
