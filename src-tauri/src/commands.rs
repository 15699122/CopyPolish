// commands.rs
// =============================================================================
// Tauri command 层：前端唯一合法入口。每个 command 通过
// python_runtime 调用嵌入式 CPython 规则引擎，返回 serde 可序列化的结果。
//
// 与 frontend/src/lib/tauri.ts 的调用一一对应（扁平参数，按名映射）：
//   invoke("format_text", { text, enabled })
//   invoke("get_rules")
//   invoke("get_enabled_defaults")
// =============================================================================

use crate::python_runtime;
use crate::python_runtime::RuleMeta;
use crate::rust_engine;

/// format_text(text, enabled) -> String
#[tauri::command]
pub fn format_text(text: String, enabled: Vec<String>) -> Result<String, String> {
    let rust_req = rust_engine::FormatRequest {
        text: text.clone(),
        enabled: enabled.clone(),
    };

    match rust_engine::format_text(&rust_req) {
        Ok(output) => Ok(output),
        Err(err) => {
            eprintln!("[rust-engine] fallback to Python: {err}");
            python_runtime::format_text(&python_runtime::FormatRequest { text, enabled })
        }
    }
}

/// get_rules() -> Vec<RuleMeta>
#[tauri::command]
pub fn get_rules() -> Result<Vec<RuleMeta>, String> {
    python_runtime::get_rules()
}

/// get_enabled_defaults() -> Vec<String>
#[tauri::command]
pub fn get_enabled_defaults() -> Result<Vec<String>, String> {
    let defaults = rust_engine::enabled_defaults();
    if defaults.is_empty() {
        python_runtime::get_enabled_defaults()
    } else {
        Ok(defaults)
    }
}
