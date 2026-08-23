// commands.rs
// =============================================================================
// Tauri command 层：前端唯一合法入口。
//
// 全部 command 由 engine / user_settings 提供。规则列表来自注册表，
// 保存/读取设置时通过 normalize_rule_keys 把旧版中文 key 迁移为稳定 key，
// 并安全丢弃未知规则。
// =============================================================================

use crate::engine::{self, RuleMeta};

/// format_text(text, selection) -> String
#[tauri::command]
pub fn format_text(text: String, selection: engine::RuleSelection) -> Result<String, String> {
    let req = engine::FormatRequest { text, selection };
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

/// save_user_settings(enabled, lastInput, theme, font)：写入 exe 同目录设置文件。
/// 归一化规则 key：旧 key 迁移、未知 key 丢弃，避免无效数据被回写。
#[tauri::command]
pub fn save_user_settings(
    enabled: Vec<String>,
    last_input: String,
    theme: crate::user_settings::ThemeMode,
    font: crate::user_settings::FontFamily,
) -> Result<(), String> {
    let filtered = engine::normalize_rule_keys(&enabled);
    crate::user_settings::save(&crate::user_settings::UserSettings {
        enabled: filtered,
        last_input,
        theme,
        font,
    })
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
