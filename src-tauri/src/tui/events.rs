use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::app::{App, FocusedPane, Overlay};
use crate::engine::RuleSelection;

pub fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
        _ => {}
    }
}

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if app.overlay == Some(Overlay::Help) {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
            app.overlay = None;
        }
        return;
    }

    if app.overlay == Some(Overlay::Rules) {
        handle_rules_overlay_key(app, key);
        return;
    }

    if key.code == KeyCode::Char('?') {
        app.overlay = Some(Overlay::Help);
        return;
    }
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.save_settings_now();
        return;
    }
    if key.code == KeyCode::Char('r') && key.modifiers.is_empty() {
        app.overlay = Some(Overlay::Rules);
        return;
    }
    if key.code == KeyCode::Esc {
        app.overlay = None;
        app.focused = FocusedPane::Input;
        return;
    }
    if app.overlay == Some(Overlay::Rules) && key.code == KeyCode::Char('q') {
        app.overlay = None;
        return;
    }
    if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
        app.should_quit = true;
        return;
    }

    match app.focused {
        FocusedPane::Input => handle_input_key(app, key),
        FocusedPane::Output => handle_navigation(app, key),
        FocusedPane::Rules => handle_rules_key(app, key),
    }
}

fn handle_input_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => app.focused = FocusedPane::Output,
        KeyCode::BackTab => app.focused = FocusedPane::Rules,
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => app.format(),
        KeyCode::Enter => app.insert_text("\n"),
        KeyCode::Backspace => {
            app.input.backspace();
            app.format();
        }
        KeyCode::Delete => {
            app.input.delete();
            app.format();
        }
        KeyCode::Left => app.input.move_left(),
        KeyCode::Right => app.input.move_right(),
        KeyCode::Up => app.input.move_up(),
        KeyCode::Down => app.input.move_down(),
        KeyCode::Home => app.input.move_home(),
        KeyCode::End => app.input.move_end(),
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => app.clear_input(),
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.move_home()
        }
        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            app.insert_text(&c.to_string())
        }
        _ => {}
    }
}

fn handle_navigation(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => app.focused = FocusedPane::Rules,
        KeyCode::BackTab => app.focused = FocusedPane::Input,
        KeyCode::Left | KeyCode::Up => app.focused = FocusedPane::Input,
        KeyCode::Down => app.scroll_output(1),
        KeyCode::PageUp => app.scroll_output(-5),
        KeyCode::PageDown => app.scroll_output(5),
        KeyCode::Home => app.scroll_output_to_start(),
        KeyCode::End => app.scroll_output_to_end(),
        KeyCode::Char('c') if key.modifiers.is_empty() => app.copy_output(),
        _ => {}
    }
}

fn handle_rules_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => app.focused = FocusedPane::Input,
        KeyCode::BackTab => app.focused = FocusedPane::Output,
        KeyCode::Up => app.move_rule(-1),
        KeyCode::Down => app.move_rule(1),
        KeyCode::Char(' ') | KeyCode::Enter => app.toggle_selected_rule(),
        KeyCode::Char('a') => app.set_selection(RuleSelection::All),
        KeyCode::Char('d') => app.set_selection(RuleSelection::Defaults),
        KeyCode::Char('n') => app.set_selection(RuleSelection::None),
        KeyCode::Char('c') if key.modifiers.is_empty() => app.copy_output(),
        KeyCode::Char('r') => app.overlay = Some(Overlay::Rules),
        _ => {}
    }
}

fn handle_rules_overlay_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => app.overlay = None,
        KeyCode::Up => app.move_rule(-1),
        KeyCode::Down => app.move_rule(1),
        KeyCode::PageUp => app.move_rule(-5),
        KeyCode::PageDown => app.move_rule(5),
        KeyCode::Home => app.selected_rule = 0,
        KeyCode::End if !app.rules.is_empty() => app.selected_rule = app.rules.len() - 1,
        KeyCode::Char(' ') | KeyCode::Enter => app.toggle_selected_rule(),
        KeyCode::Char('a') => app.set_selection(RuleSelection::All),
        KeyCode::Char('d') => app.set_selection(RuleSelection::Defaults),
        KeyCode::Char('n') => app.set_selection(RuleSelection::None),
        KeyCode::Char('c') if key.modifiers.is_empty() => app.copy_output(),
        KeyCode::Char('?') => app.overlay = Some(Overlay::Help),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    use super::handle_event;
    use crate::tui::app::{App, FocusedPane, Overlay};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn rules_overlay_can_open_from_input_and_close_without_quitting() {
        let mut app = App::new();
        handle_event(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.overlay, Some(Overlay::Rules));
        handle_event(&mut app, key(KeyCode::Char('q')));
        assert_eq!(app.overlay, None);
        assert!(!app.should_quit);
    }

    #[test]
    fn rules_overlay_handles_rule_actions_independently_of_focus() {
        let mut app = App::new();
        app.focused = FocusedPane::Output;
        app.overlay = Some(Overlay::Rules);
        handle_event(&mut app, key(KeyCode::Char('n')));
        assert!(matches!(app.selection, crate::engine::RuleSelection::None));
        handle_event(&mut app, key(KeyCode::Char('d')));
        assert!(matches!(
            app.selection,
            crate::engine::RuleSelection::Defaults
        ));
    }

    #[test]
    fn output_navigation_changes_scroll_without_editing_text() {
        let mut app = App::new();
        app.focused = FocusedPane::Output;
        let input = app.input.text().to_string();
        handle_event(&mut app, key(KeyCode::Down));
        assert_eq!(app.output_scroll, 1);
        assert_eq!(app.input.text(), input);
    }

    #[test]
    fn ctrl_s_triggers_settings_save_status() {
        let mut app = App::new();
        let event = Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        handle_event(&mut app, event);
        // 默认 --no-config 场景下只提示，不写入文件。
        assert!(matches!(app.status, crate::tui::app::Status::Info(_)));
    }

    #[test]
    fn copy_key_in_output_pane_reports_info_or_error_without_quitting() {
        let mut app = App::new();
        app.insert_text("hi");
        app.focused = FocusedPane::Output;
        handle_event(&mut app, key(KeyCode::Char('c')));
        match app.status {
            crate::tui::app::Status::Info(_) | crate::tui::app::Status::Error(_) => {}
            other => panic!("expected clipboard status, got {other:?}"),
        }
        assert!(!app.should_quit);
    }

    #[test]
    fn help_overlay_opens_with_question_mark_and_closes_with_escape() {
        let mut app = App::new();
        handle_event(&mut app, key(KeyCode::Char('?')));
        assert_eq!(app.overlay, Some(Overlay::Help));
        handle_event(&mut app, key(KeyCode::Esc));
        assert_eq!(app.overlay, None);
    }
}
