//! Ratatui 终端界面。
//!
//! TUI 只负责交互状态和展示，格式化行为统一复用公开的 Rust engine。

mod app;
pub mod cli;
mod clipboard;
mod editor;
pub(crate) mod events;
pub mod settings;
mod terminal;
pub(crate) mod ui;
mod wrap;

pub use app::App;
pub use terminal::run;
