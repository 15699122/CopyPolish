// Tauri 2 应用装配：纯 Rust 实现（engine + user_settings），
// 不依赖任何 Python / PyO3，也不受历史 12 条规则清单的架构约束。

mod commands;
pub mod engine;
mod user_settings;

#[cfg(feature = "tui")]
pub mod tui;

pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(feature = "e2e")]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());

    builder
        .invoke_handler(tauri::generate_handler![
            commands::format_text,
            commands::get_rules,
            commands::get_enabled_defaults,
            commands::get_user_settings,
            commands::get_settings_path,
            commands::save_user_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
