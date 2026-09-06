// commands.rs
// =============================================================================
// Tauri command 层：前端唯一合法入口。
//
// 全部 command 由 engine / user_settings 提供。规则列表来自注册表，
// 保存/读取设置时通过 normalize_rule_keys 把旧版中文 key 迁移为稳定 key，
// 并安全丢弃未知规则。
// =============================================================================

use crate::engine::{self, CharacterConversion, ReplacementPair, RuleMeta};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCapabilities {
    pub simplified_trad_conversion: bool,
}

/// 返回当前 binary 编译时包含的可选能力。
#[tauri::command]
pub fn get_build_capabilities() -> BuildCapabilities {
    BuildCapabilities {
        simplified_trad_conversion: cfg!(feature = "simplified-trad-conversion"),
    }
}

#[cfg(test)]
mod tests {
    use super::get_build_capabilities;

    #[test]
    fn build_capabilities_match_the_compiled_feature() {
        assert_eq!(
            get_build_capabilities().simplified_trad_conversion,
            cfg!(feature = "simplified-trad-conversion"),
        );
    }
}

/// format_text(text, selection, replacements, conversion) -> String
///
/// `replacements` 与 `conversion` 为可选的请求层阶段；默认空 / None 时
/// 输出与扩展前完全一致，旧调用方可只传 `{ text, selection }`。
#[tauri::command]
pub fn format_text(
    text: String,
    selection: engine::RuleSelection,
    replacements: Option<Vec<ReplacementPair>>,
    conversion: Option<CharacterConversion>,
) -> Result<String, String> {
    let req = engine::FormatRequest {
        text,
        selection,
        replacements: replacements.unwrap_or_default(),
        conversion: conversion.unwrap_or_default(),
    };
    engine::format_text(&req)
}

/// get_user_settings() -> Option<LoadedUserSettings>
/// 读取 exe 同目录设置文件；不存在时返回 None（前端使用默认规则集）。
/// 历史中文规则 key 在读取时迁移为稳定 key。
#[tauri::command]
pub fn get_user_settings() -> Result<Option<crate::user_settings::LoadedUserSettings>, String> {
    Ok(crate::user_settings::load_with_status().map(|mut loaded| {
        loaded.settings.enabled = engine::normalize_rule_keys(&loaded.settings.enabled);
        loaded
    }))
}

/// get_settings_path() -> String
/// 返回设置文件的完整路径（rules.yaml），供前端在设置弹窗中展示。
#[tauri::command]
pub fn get_settings_path() -> Result<String, String> {
    Ok(crate::user_settings::settings_path()
        .to_string_lossy()
        .into_owned())
}

/// save_user_settings(settings)：写入 exe 同目录设置文件。
/// 归一化规则 key：旧 key 迁移、未知 key 丢弃，避免无效数据被回写。
#[tauri::command]
pub fn save_user_settings(settings: crate::user_settings::UserSettings) -> Result<(), String> {
    let mut settings = settings;
    settings.enabled = engine::normalize_rule_keys(&settings.enabled);
    crate::user_settings::enforce_input_privacy(&mut settings);
    crate::user_settings::save(&settings)
}

/// get_rules() -> Vec<RuleMeta>
/// 注册表内置规则元数据。
#[tauri::command]
pub fn get_rules() -> Result<Vec<RuleMeta>, String> {
    let rules = engine::default_rules();
    if rules.is_empty() {
        return Err("engine rule registry is empty".to_string());
    }
    Ok(rules)
}

/// get_enabled_defaults() -> Vec<String>
#[tauri::command]
pub fn get_enabled_defaults() -> Result<Vec<String>, String> {
    let defaults = engine::enabled_defaults();
    if defaults.is_empty() {
        return Err("engine default rules are empty".to_string());
    }
    Ok(defaults)
}

/// 返回内置工作流预设；预设只组合统一请求模型字段。
#[tauri::command]
pub fn get_presets() -> Result<Vec<engine::Preset>, String> {
    Ok(engine::presets())
}
