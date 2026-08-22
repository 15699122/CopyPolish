// commands.rs
// =============================================================================
// Tauri command 层：前端唯一合法入口。
//
// 纯 Rust 实现：全部 command 由 rust_engine / user_settings 提供，
// 不依赖 Python。与 frontend/src/lib/tauri.ts 的调用一一对应（扁平参数）：
//   invoke("format_text", { text, enabled })
//   invoke("get_rules")
//   invoke("get_enabled_defaults")
// =============================================================================

use crate::rust_engine::{self, RuleMeta};

/// format_text(text, enabled) -> String
#[tauri::command]
pub fn format_text(text: String, enabled: Vec<String>) -> Result<String, String> {
    let rust_req = rust_engine::FormatRequest { text, enabled };
    rust_engine::format_text(&rust_req)
}

/// get_user_settings() -> Option<UserSettings>
/// 读取 exe 同目录设置文件；不存在时返回 None（前端使用默认规则集）。
#[tauri::command]
pub fn get_user_settings() -> Result<Option<crate::user_settings::UserSettings>, String> {
    Ok(crate::user_settings::load())
}

/// get_settings_path() -> String
/// 返回设置文件的完整路径（rules.yaml），供前端在设置弹窗中展示。
#[tauri::command]
pub fn get_settings_path() -> Result<String, String> {
    Ok(crate::user_settings::settings_path()
        .to_string_lossy()
        .into_owned())
}

/// save_user_settings(enabled, lastInput, theme)：写入 exe 同目录设置文件。
/// 过滤掉引擎中不存在的规则 key，避免旧设置中的已删除规则被回写。
#[tauri::command]
pub fn save_user_settings(
    enabled: Vec<String>,
    last_input: String,
    theme: crate::user_settings::ThemeMode,
) -> Result<(), String> {
    let known: std::collections::HashSet<String> = rust_engine::default_rules()
        .into_iter()
        .map(|r| r.key)
        .collect();
    let filtered: Vec<String> = enabled.into_iter().filter(|k| known.contains(k)).collect();
    crate::user_settings::save(&crate::user_settings::UserSettings {
        enabled: filtered,
        last_input,
        theme,
    })
}

/// get_rules() -> Vec<RuleMeta>
/// Rust 端内置规则元数据。
#[tauri::command]
pub fn get_rules() -> Result<Vec<RuleMeta>, String> {
    let rules = rust_engine::default_rules();
    if rules.is_empty() {
        return Err("rust engine rule metadata is empty".to_string());
    }
    Ok(rules)
}

/// get_enabled_defaults() -> Vec<String>
#[tauri::command]
pub fn get_enabled_defaults() -> Result<Vec<String>, String> {
    let defaults = rust_engine::enabled_defaults();
    if defaults.is_empty() {
        return Err("rust engine default rules are empty".to_string());
    }
    Ok(defaults)
}
