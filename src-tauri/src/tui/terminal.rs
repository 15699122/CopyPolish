use std::io;
use std::time::Duration;

use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::{app::App, events, ui};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            DisableBracketedPaste,
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

pub fn run(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        if app.should_quit {
            break;
        }
        if let Some(event) = events::poll_event(Duration::from_millis(50))? {
            events::handle_event(&mut app, event);
        }
    }

    // 正常退出时把规则选择、替换、转换和最近输入写回共享设置；失败仅提示，不影响退出码。
    if !app.no_config {
        if let Err(error) = super::settings::persist(
            &app.selection,
            app.input.text(),
            &app.replacements,
            app.conversion,
        ) {
            eprintln!("文案净排：保存设置失败：{error}");
        }
    }
    Ok(())
}
