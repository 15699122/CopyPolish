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
/// 读取当前工作目录设置文件；不存在时返回 None（前端使用默认规则集）。
#[tauri::command]
pub fn get_user_settings() -> Result<Option<crate::user_settings::UserSettings>, String> {
    Ok(crate::user_settings::load())
}

/// save_user_settings(enabled, last_input)：写入当前工作目录设置文件。
#[tauri::command]
pub fn save_user_settings(enabled: Vec<String>, last_input: String) -> Result<(), String> {
    crate::user_settings::save(&crate::user_settings::UserSettings {
        enabled,
        last_input,
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
