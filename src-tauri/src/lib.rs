// Tauri 2 应用装配：纯 Rust 实现（rust_engine + user_settings），
// 不含任何 Python / PyO3 依赖。

mod commands;
pub mod rust_engine;
mod user_settings;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::format_text,
            commands::get_rules,
            commands::get_enabled_defaults,
            commands::get_user_settings,
            commands::save_user_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
