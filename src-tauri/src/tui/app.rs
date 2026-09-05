use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use crate::engine::{
    self, CharacterConversion, FormatRequest, Preset, ReplacementPair, RuleMeta, RuleSelection,
};

use super::clipboard;
use super::editor::TextEditor;
use super::settings::{self, SharedConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusedPane {
    Input,
    Output,
    Rules,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    Help,
    Rules,
    Request,
    Presets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestField {
    From,
    To,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Ready,
    Formatted {
        elapsed: Duration,
    },
    /// 中性提示信息（已复制、已保存等），状态栏原样展示。
    Info(String),
    Error(String),
}

pub struct App {
    pub input: TextEditor,
    pub output: String,
    pub rules: Vec<RuleMeta>,
    pub selection: RuleSelection,
    pub focused: FocusedPane,
    pub overlay: Option<Overlay>,
    pub status: Status,
    pub should_quit: bool,
    pub selected_rule: usize,
    pub output_scroll: u16,
    pub replacements: Vec<ReplacementPair>,
    pub conversion: CharacterConversion,
    pub selected_replacement: usize,
    pub request_field: RequestField,
    pub presets: Vec<Preset>,
    pub selected_preset: usize,
    /// `--no-config`：跳过共享 rules.yaml 的读取与写入。
    pub no_config: bool,
}

impl App {
    pub fn new() -> Self {
        Self::with_config(None, true)
    }

    /// 按共享设置构造应用；`shared` 为 None 时回落到默认规则与空输入。
    pub fn with_config(shared: Option<SharedConfig>, no_config: bool) -> Self {
        let mut rules = engine::default_rules();
        // TUI 展示顺序独立于 engine pipeline：默认启用规则优先，组内保持
        // 注册表顺序，避免改变实际执行顺序或设置 key 语义。
        rules.sort_by_key(|rule| !rule.default);
        let selection = shared
            .as_ref()
            .map(|config| config.selection.clone())
            .unwrap_or(RuleSelection::Defaults);
        let last_input = shared
            .as_ref()
            .filter(|config| !config.last_input.is_empty())
            .map(|config| config.last_input.clone())
            .unwrap_or_default();
        let replacements = shared
            .as_ref()
            .map(|config| config.replacements.clone())
            .unwrap_or_default();
        let conversion = shared
            .as_ref()
            .map(|config| settings::normalize_conversion(config.conversion))
            .unwrap_or_default();
        let mut app = Self {
            input: TextEditor::new(last_input),
            output: String::new(),
            rules,
            selection,
            focused: FocusedPane::Input,
            overlay: None,
            status: Status::Ready,
            should_quit: false,
            selected_rule: 0,
            output_scroll: 0,
            replacements,
            conversion,
            selected_replacement: 0,
            request_field: super::app::RequestField::From,
            presets: engine::presets(),
            selected_preset: 0,
            no_config,
        };
        app.format();
        app
    }

    pub fn format(&mut self) {
        let started = Instant::now();
        let request = FormatRequest {
            text: self.input.text().to_string(),
            selection: self.selection.clone(),
            replacements: self.replacements.clone(),
            conversion: settings::normalize_conversion(self.conversion),
        };
        match engine::format_text(&request) {
            Ok(output) => {
                self.output = output;
                self.status = Status::Formatted {
                    elapsed: started.elapsed(),
                };
            }
            Err(error) => self.status = Status::Error(error),
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        self.input.insert(text);
        self.format();
    }

    pub fn set_selection(&mut self, selection: RuleSelection) {
        self.selection = selection;
        self.format();
    }

    pub fn toggle_selected_rule(&mut self) {
        let Some(rule) = self.rules.get(self.selected_rule) else {
            return;
        };
        let mut keys = self.selected_keys();
        if !keys.remove(&rule.key) {
            keys.insert(rule.key.clone());
        }
        self.selection = RuleSelection::Only {
            keys: keys.into_iter().collect(),
        };
        self.format();
    }

    pub fn selected_keys(&self) -> BTreeSet<String> {
        match &self.selection {
            RuleSelection::All => self.rules.iter().map(|rule| rule.key.clone()).collect(),
            RuleSelection::Defaults => self
                .rules
                .iter()
                .filter(|rule| rule.default)
                .map(|rule| rule.key.clone())
                .collect(),
            RuleSelection::Only { keys } => keys.iter().cloned().collect(),
            RuleSelection::None => BTreeSet::new(),
        }
    }

    pub fn move_rule(&mut self, delta: isize) {
        if self.rules.is_empty() {
            return;
        }
        let last = self.rules.len() - 1;
        self.selected_rule = self.selected_rule.saturating_add_signed(delta).min(last);
    }

    pub fn clear_input(&mut self) {
        self.input = TextEditor::default();
        self.output_scroll = 0;
        self.format();
    }

    pub fn add_replacement(&mut self) {
        self.replacements.push(ReplacementPair::default());
        self.selected_replacement = self.replacements.len().saturating_sub(1);
        self.request_field = RequestField::From;
    }

    pub fn remove_selected_replacement(&mut self) {
        if self.selected_replacement < self.replacements.len() {
            self.replacements.remove(self.selected_replacement);
            self.selected_replacement = self
                .selected_replacement
                .min(self.replacements.len().saturating_sub(1));
            self.format();
        }
    }

    pub fn move_replacement(&mut self, delta: isize) {
        if self.replacements.is_empty() {
            return;
        }
        self.selected_replacement = self
            .selected_replacement
            .saturating_add_signed(delta)
            .min(self.replacements.len() - 1);
    }

    pub fn toggle_selected_replacement(&mut self) {
        if let Some(replacement) = self.replacements.get_mut(self.selected_replacement) {
            replacement.active = !replacement.active;
            self.format();
        }
    }

    pub fn insert_request_text(&mut self, text: &str) {
        let Some(replacement) = self.replacements.get_mut(self.selected_replacement) else {
            return;
        };
        match self.request_field {
            RequestField::From => replacement.from.push_str(text),
            RequestField::To => replacement.to.push_str(text),
        }
    }

    pub fn backspace_request_text(&mut self) {
        let Some(replacement) = self.replacements.get_mut(self.selected_replacement) else {
            return;
        };
        let value = match self.request_field {
            RequestField::From => &mut replacement.from,
            RequestField::To => &mut replacement.to,
        };
        if let Some((index, _)) = value.char_indices().next_back() {
            value.truncate(index);
        }
    }

    pub fn cycle_conversion(&mut self) {
        self.conversion = if !cfg!(feature = "simplified-trad-conversion") {
            CharacterConversion::None
        } else {
            match self.conversion {
                CharacterConversion::None => CharacterConversion::TraditionalToSimplified,
                CharacterConversion::TraditionalToSimplified => {
                    CharacterConversion::SimplifiedToTraditional
                }
                CharacterConversion::SimplifiedToTraditional => CharacterConversion::None,
            }
        };
        self.format();
    }

    pub fn move_preset(&mut self, delta: isize) {
        if self.presets.is_empty() {
            return;
        }
        self.selected_preset = self
            .selected_preset
            .saturating_add_signed(delta)
            .min(self.presets.len() - 1);
    }

    pub fn apply_selected_preset(&mut self) {
        let Some(preset) = self.presets.get(self.selected_preset).cloned() else {
            return;
        };
        self.selection = preset.selection;
        self.replacements = preset.replacements;
        self.conversion = settings::normalize_conversion(preset.conversion);
        self.selected_replacement = 0;
        self.request_field = RequestField::From;
        self.format();
        self.status = Status::Info(format!("已应用预设：{}", preset.name));
    }

    pub fn scroll_output(&mut self, delta: i16) {
        if delta.is_negative() {
            self.output_scroll = self.output_scroll.saturating_sub(delta.unsigned_abs());
        } else {
            self.output_scroll = self.output_scroll.saturating_add(delta as u16);
        }
    }

    pub fn scroll_output_to_start(&mut self) {
        self.output_scroll = 0;
    }

    pub fn scroll_output_to_end(&mut self) {
        self.output_scroll = u16::MAX;
    }

    /// 通过 OSC 52 复制当前输出到系统剪贴板。
    pub fn copy_output(&mut self) {
        match clipboard::copy_to_clipboard(&self.output) {
            Ok(()) => self.status = Status::Info(
                "已复制输出（OSC 52）；若粘贴为空，说明终端不支持或禁用了 OSC 52，请改用 --stdin/--output".to_string(),
            ),
            Err(message) => self.status = Status::Error(message),
        }
    }

    /// 将规则选择与最近输入写入共享 `rules.yaml`（读改写，保留 GUI 字段）。
    pub fn save_settings_now(&mut self) {
        if self.no_config {
            self.status = Status::Info("--no-config 模式不保存设置".to_string());
            return;
        }
        match settings::persist(
            &self.selection,
            self.input.text(),
            &self.replacements,
            self.conversion,
        ) {
            Ok(()) => self.status = Status::Info("已保存规则、替换与转换设置".to_string()),
            Err(error) => self.status = Status::Error(format!("保存设置失败：{error}")),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{App, FocusedPane, Overlay, Status};
    use crate::engine::{CharacterConversion, ReplacementPair, RuleSelection};

    #[test]
    fn default_app_uses_default_rules() {
        let app = App::new();
        assert_eq!(app.selection, RuleSelection::Defaults);
        assert!(matches!(app.status, Status::Formatted { .. }));
    }

    #[test]
    fn tui_rules_keep_defaults_before_non_defaults_stably() {
        let app = App::new();
        let first_disabled = app.rules.iter().position(|rule| !rule.default);
        assert!(
            first_disabled.is_some(),
            "expected at least one disabled rule"
        );
        let first_disabled = first_disabled.unwrap();
        assert!(app.rules[..first_disabled].iter().all(|rule| rule.default));
        assert!(app.rules[first_disabled..].iter().all(|rule| !rule.default));
    }

    #[test]
    fn none_selection_keeps_input_unchanged() {
        let mut app = App::new();
        app.insert_text("在LeanCloud上");
        app.set_selection(RuleSelection::None);
        assert_eq!(app.output, "在LeanCloud上");
    }

    #[test]
    fn toggling_rule_creates_explicit_only_selection() {
        let mut app = App::new();
        app.focused = FocusedPane::Rules;
        let key = app.rules[0].key.clone();
        app.toggle_selected_rule();
        assert!(matches!(app.selection, RuleSelection::Only { .. }));
        assert!(!app.selected_keys().contains(&key));
    }

    #[test]
    fn output_scroll_is_saturating() {
        let mut app = App::new();
        app.scroll_output(-1);
        assert_eq!(app.output_scroll, 0);
        app.scroll_output(3);
        assert_eq!(app.output_scroll, 3);
        app.scroll_output_to_start();
        assert_eq!(app.output_scroll, 0);
        app.scroll_output_to_end();
        assert_eq!(app.output_scroll, u16::MAX);
    }

    #[test]
    fn shared_config_restores_last_input_and_selection() {
        let config = super::SharedConfig {
            selection: RuleSelection::None,
            last_input: "在LeanCloud上".to_string(),
            replacements: vec![],
            conversion: CharacterConversion::None,
        };
        let app = App::with_config(Some(config), false);
        assert_eq!(app.input.text(), "在LeanCloud上");
        assert_eq!(app.selection, RuleSelection::None);
    }

    #[test]
    fn empty_shared_last_input_falls_back_to_empty_editor() {
        let config = super::SharedConfig {
            selection: RuleSelection::Defaults,
            last_input: String::new(),
            replacements: vec![],
            conversion: CharacterConversion::None,
        };
        let app = App::with_config(Some(config), true);
        assert_eq!(app.input.text(), "");
        // 尽管恢复了共享选择，仍不回写设置。
        assert!(app.no_config);
    }

    #[test]
    fn shared_config_restores_request_settings_and_formats_with_replacement() {
        let config = super::SharedConfig {
            selection: RuleSelection::None,
            last_input: "TODO".to_string(),
            replacements: vec![ReplacementPair {
                from: "TODO".to_string(),
                to: "待办".to_string(),
                active: true,
            }],
            conversion: CharacterConversion::None,
        };
        let app = App::with_config(Some(config), true);
        assert_eq!(app.replacements[0].from, "TODO");
        assert_eq!(app.replacements[0].to, "待办");
        assert_eq!(app.output, "待办");
    }

    #[test]
    fn copy_output_reports_success_with_osc52_fallback_hint() {
        let mut app = App::new();
        app.insert_text("hi");
        app.copy_output();
        match &app.status {
            Status::Info(message) => {
                assert!(message.contains("OSC 52"), "got: {message}");
                // 降级提示：粘贴为空时用户能知道原因与替代方案。
                assert!(message.contains("粘贴为空"), "got: {message}");
                assert!(message.contains("--stdin"), "got: {message}");
            }
            other => panic!("expected info status, got {other:?}"),
        }
    }

    #[test]
    fn oversized_output_reports_clipboard_error_without_panic() {
        let mut app = App::new();
        app.output = "a".repeat(super::super::clipboard::MAX_CLIPBOARD_BYTES + 1);
        app.copy_output();
        match &app.status {
            Status::Error(message) => assert!(message.contains("输出过大"), "got: {message}"),
            other => panic!("expected clipboard error, got {other:?}"),
        }
    }

    #[test]
    fn save_settings_is_noop_under_no_config() {
        let mut app = App::new();
        assert!(app.no_config);
        app.save_settings_now();
        assert_eq!(
            app.status,
            Status::Info("--no-config 模式不保存设置".to_string())
        );
        assert!(!app.should_quit);
    }

    #[test]
    fn overlay_variants_cover_help_and_rules() {
        let app = App::new();
        assert_eq!(app.overlay, None);
        let _ = (
            Overlay::Help,
            Overlay::Rules,
            Overlay::Request,
            Overlay::Presets,
        );
    }
}
