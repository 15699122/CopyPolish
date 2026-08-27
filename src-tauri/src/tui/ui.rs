use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use super::app::{App, FocusedPane, Overlay, Status};

pub fn render(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " 文案净排 ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/ CopyPolish TUI"),
        ])),
        root[0],
    );

    let body = if root[1].width >= 100 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(root[1])
            .to_vec()
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(root[1])
            .to_vec()
    };

    let input_style = if app.focused == FocusedPane::Input {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let input_area = body[0];
    let input_inner_height = input_area.height.saturating_sub(2).max(1) as usize;
    let (cursor_line, _) = app.input.line_column();
    let input_scroll = cursor_line.saturating_sub(input_inner_height.saturating_sub(1)) as u16;
    let output_style = if app.focused == FocusedPane::Output {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    frame.render_widget(
        Paragraph::new(Text::from(app.input.text().to_string()))
            .block(
                Block::default()
                    .title(" 输入 ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(input_style)),
            )
            .wrap(Wrap { trim: false })
            .scroll((input_scroll, 0)),
        input_area,
    );
    frame.render_widget(
        Paragraph::new(Text::from(app.output.clone()))
            .block(
                Block::default()
                    .title(" 输出 ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(output_style)),
            )
            .wrap(Wrap { trim: false })
            .scroll((app.output_scroll, 0)),
        body[1],
    );

    if app.focused == FocusedPane::Input {
        let visible_line = cursor_line.saturating_sub(input_scroll as usize);
        let line_start = app.input.text()[..app.input.cursor()]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let column_text = &app.input.text()[line_start..app.input.cursor()];
        let cursor_x = input_area
            .x
            .saturating_add(1)
            .saturating_add(column_text.width() as u16);
        let cursor_y = input_area
            .y
            .saturating_add(1)
            .saturating_add(visible_line as u16);
        if cursor_x < input_area.right().saturating_sub(1)
            && cursor_y < input_area.bottom().saturating_sub(1)
        {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    let status = match &app.status {
        Status::Ready => "就绪".to_string(),
        Status::Formatted { elapsed } => {
            format!("已格式化 · {:.2} ms", elapsed.as_secs_f64() * 1000.0)
        }
        Status::Info(message) => message.clone(),
        Status::Error(error) => format!("错误：{error}"),
    };
    let selected = app.selected_keys().len();
    let footer = format!(
        " {status} · 规则 {selected}/{} · Tab 切换 · r 规则 · c 复制 · Ctrl+S 保存 · ? 帮助 · q 退出",
        app.rules.len()
    );
    frame.render_widget(Paragraph::new(footer), root[2]);

    if app.overlay == Some(Overlay::Rules) {
        render_rules(frame, app);
    } else if app.overlay == Some(Overlay::Help) {
        render_help(frame);
    }
}

fn render_rules(frame: &mut Frame, app: &App) {
    let area = centered_rect(78, 82, frame.area());
    frame.render_widget(Clear, area);
    let selected = app.selected_keys();
    let items = app.rules.iter().map(|rule| {
        let marker = if selected.contains(&rule.key) {
            "[x]"
        } else {
            "[ ]"
        };
        let disputed = if rule.disputed { " · 争议" } else { "" };
        ListItem::new(format!(
            "{marker} {} · {}{disputed}",
            rule.section, rule.name
        ))
    });
    let list = List::new(items)
        .block(
            Block::default()
                .title(" 规则 · ↑↓ 选择 · Space 切换 · a 全选 · d 默认 · n 全不选 · Esc 关闭 ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = list_state(app.selected_rule);
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);
    let help = [
        "快捷键",
        "",
        "Tab / Shift+Tab  切换输入、输出和规则区域",
        "Ctrl+Enter       立即排版",
        "r                打开规则面板",
        "Space / Enter     切换当前规则",
        "a / d / n          全选 / 恢复默认 / 全不选",
        "c                复制输出（OSC 52）",
        "Ctrl+S           保存规则与最近输入到 rules.yaml",
        "x                清空输入",
        "? / Esc           帮助 / 关闭覆盖层",
        "q                退出（退出时自动保存设置）",
        "",
        "Esc 关闭此帮助",
    ];
    frame.render_widget(
        Paragraph::new(help.join("\n"))
            .block(Block::default().title(" 帮助 ").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn list_state(selected: usize) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected));
    state
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
