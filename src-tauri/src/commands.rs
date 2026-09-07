// commands.rs
// =============================================================================
// Tauri command 层：前端唯一合法入口。
//
// 全部 command 由 engine / user_settings 提供。规则列表来自注册表，
// 保存/读取设置时通过 normalize_rule_keys 把旧版中文 key 迁移为稳定 key，
// 并安全丢弃未知规则。
// =============================================================================

use crate::engine::{self, CharacterConversion, ReplacementPair, RuleMeta};

/// Tauri IPC 对外错误分类。message 只包含用户可行动的安全提示，
/// 不直接透传用户正文、绝对路径或底层文件系统错误。
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandErrorCode {
    InputTooLarge,
    TooManyRules,
    TooManyReplacements,
    ReplacementTooLarge,
    SettingsTooLarge,
    SettingsInvalid,
    SettingsPermissionDenied,
    SettingsPathUnsafe,
    Internal,
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub code: CommandErrorCode,
    pub message: String,
}

impl CommandError {
    fn new(code: CommandErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn map_engine_error(error: String) -> CommandError {
    if error.contains("input text exceeds") {
        return CommandError::new(
            CommandErrorCode::InputTooLarge,
            "输入文本超过允许大小（10 MiB）。",
        );
    }
    if error.contains("rule selection exceeds") {
        return CommandError::new(
            CommandErrorCode::TooManyRules,
            "启用的规则数量超过允许上限。",
        );
    }
    if error.contains("replacements exceed") {
        return CommandError::new(
            CommandErrorCode::TooManyReplacements,
            "自定义替换项数量超过允许上限。",
        );
    }
    if error.contains("replacement fields exceed") {
        return CommandError::new(
            CommandErrorCode::ReplacementTooLarge,
            "自定义替换项字段超过允许大小。",
        );
    }
    CommandError::new(CommandErrorCode::Internal, "格式化失败，请检查输入后重试。")
}

fn map_settings_error(error: String) -> CommandError {
    if error.contains("exceeds the 2 MiB limit") {
        return CommandError::new(CommandErrorCode::SettingsTooLarge, "设置文件超过允许大小。");
    }
    if error.contains("symbolic link") || error.contains("reparse") {
        return CommandError::new(
            CommandErrorCode::SettingsPathUnsafe,
            "设置路径不安全，已拒绝写入。",
        );
    }
    if error.contains("permission")
        || error.contains("Permission")
        || error.contains("permission denied")
        || error.contains("Access is denied")
    {
        return CommandError::new(
            CommandErrorCode::SettingsPermissionDenied,
            "没有权限保存设置文件。",
        );
    }
    if error.contains("parse") || error.contains("invalid") || error.contains("corrupt") {
        return CommandError::new(
            CommandErrorCode::SettingsInvalid,
            "设置文件无效，已拒绝写入。",
        );
    }
    CommandError::new(CommandErrorCode::Internal, "设置保存失败，请稍后重试。")
}

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
    use super::{get_build_capabilities, map_engine_error, map_settings_error, CommandErrorCode};

    #[test]
    fn build_capabilities_match_the_compiled_feature() {
        assert_eq!(
            get_build_capabilities().simplified_trad_conversion,
            cfg!(feature = "simplified-trad-conversion"),
        );
    }

    #[test]
    fn command_errors_use_stable_codes_and_safe_messages() {
        let input =
            map_engine_error("input text exceeds the 10 MiB limit: secret body".to_string());
        assert_eq!(input.code, CommandErrorCode::InputTooLarge);
        assert!(!input.message.contains("secret body"));

        let settings = map_settings_error(
            "settings path must not be a symbolic link: /home/user/private/rules.yaml".to_string(),
        );
        assert_eq!(settings.code, CommandErrorCode::SettingsPathUnsafe);
        assert!(!settings.message.contains("/home/user"));

        let internal = map_engine_error("unexpected internal path /tmp/private/input".to_string());
        assert_eq!(internal.code, CommandErrorCode::Internal);
        assert!(!internal.message.contains("/tmp/private"));
    }
}

/// format_text(text, selection, replacements, conversion) -> String / CommandError
///
/// `replacements` 与 `conversion` 为可选的请求层阶段；默认空 / None 时
/// 输出与扩展前完全一致，旧调用方可只传 `{ text, selection }`。
#[tauri::command]
pub fn format_text(
    text: String,
    selection: engine::RuleSelection,
    replacements: Option<Vec<ReplacementPair>>,
    conversion: Option<CharacterConversion>,
) -> Result<String, CommandError> {
    let req = engine::FormatRequest {
        text,
        selection,
        replacements: replacements.unwrap_or_default(),
        conversion: conversion.unwrap_or_default(),
    };
    engine::format_text(&req).map_err(map_engine_error)
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

/// get_settings_path() -> String / CommandError
/// 返回设置文件的完整路径（rules.yaml），供前端在设置弹窗中展示。
#[tauri::command]
pub fn get_settings_path() -> Result<String, CommandError> {
    Ok(crate::user_settings::settings_path()
        .to_string_lossy()
        .into_owned())
}

/// save_user_settings(settings)：写入 exe 同目录设置文件；失败时返回 CommandError。
/// 归一化规则 key：旧 key 迁移、未知 key 丢弃，避免无效数据被回写。
#[tauri::command]
pub fn save_user_settings(
    settings: crate::user_settings::UserSettings,
) -> Result<(), CommandError> {
    let mut settings = settings;
    settings.enabled = engine::normalize_rule_keys(&settings.enabled);
    crate::user_settings::enforce_input_privacy(&mut settings);
    crate::user_settings::validate_user_settings(&settings).map_err(map_settings_error)?;
    crate::user_settings::save(&settings).map_err(map_settings_error)
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
