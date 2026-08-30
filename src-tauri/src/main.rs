// Tauri 2 桌面应用入口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chinese_copywriting_formatter_lib::run()
}
