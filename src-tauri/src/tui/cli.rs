//! 非交互命令行模式：stdin / 文件 → 引擎 → stdout / 文件。
//!
//! 与 TUI 共享同一套 `RuleSelection` 构建逻辑（见 `super::settings`），
//! 优先级为：显式 `--rules` > 共享 `rules.yaml` > 默认规则。
//! 仅使用标准库解析参数，不引入 clap。

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use super::settings;
use crate::engine::{FormatRequest, RuleMeta};

/// `--rules` 的基础模式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RulesMode {
    All,
    Defaults,
    None,
}

/// 解析后的命令行选项。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Cli {
    pub stdin: bool,
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub rules_mode: Option<RulesMode>,
    pub enable: Vec<String>,
    pub disable: Vec<String>,
    pub no_config: bool,
    pub help: bool,
}

impl Cli {
    /// 是否进入非交互模式：只要指定了任一输入/输出来源即生效。
    pub fn wants_non_interactive(&self) -> bool {
        self.stdin || self.input.is_some() || self.output.is_some()
    }
}

/// 返回 `--help` 文本；交互参数之外的用法都在这里说明。
pub fn usage() -> &'static str {
    "用法：copypolish-tui [选项]

默认（无以下输入输出参数时）启动交互式终端界面。

非交互模式：
  --stdin               从标准输入读取文本并输出结果
  --input <路径>        从文件读取文本
  --output <路径>       将结果写入文件（缺省写到标准输出）

规则选择（仅影响本次运行）：
  --rules <all|defaults|none>
                        指定基础规则集，覆盖共享设置
  --enable <key>        在基础集之上追加启用规则（可多次使用）
  --disable <key>       从基础集移除规则（可多次使用）

其它：
  --no-config           不读取也不写入共享的 rules.yaml
  --help, -h            显示本帮助

示例：
  printf '在LeanCloud上，花了5000元' | copypolish-tui --stdin --no-config
"
}

fn take_value<'a>(flag: &str, value: Option<&'a String>) -> Result<&'a str, String> {
    value
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} 缺少取值"))
}

fn parse_rules_mode(value: &str) -> Result<RulesMode, String> {
    match value {
        "all" => Ok(RulesMode::All),
        "defaults" | "default" => Ok(RulesMode::Defaults),
        "none" => Ok(RulesMode::None),
        other => Err(format!(
            "--rules 仅接受 all、defaults 或 none，收到：{other}"
        )),
    }
}

/// 解析命令行参数；失败时返回人类可读错误。
pub fn parse(args: &[String]) -> Result<Cli, String> {
    let mut cli = Cli::default();
    let mut values = args.iter();
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--stdin" => cli.stdin = true,
            "--input" => cli.input = Some(PathBuf::from(take_value("--input", values.next())?)),
            "--output" => {
                cli.output = Some(PathBuf::from(take_value("--output", values.next())?));
            }
            "--rules" => {
                cli.rules_mode = Some(parse_rules_mode(take_value("--rules", values.next())?)?);
            }
            "--enable" => cli
                .enable
                .push(take_value("--enable", values.next())?.to_string()),
            "--disable" => cli
                .disable
                .push(take_value("--disable", values.next())?.to_string()),
            "--no-config" => cli.no_config = true,
            "--help" | "-h" => cli.help = true,
            other => return Err(format!("未知参数：{other}（使用 --help 查看用法）")),
        }
    }
    Ok(cli)
}

/// 收集 enable/disable 中不存在的规则 key，供调用方向 stderr 警告。
pub fn unknown_filter_keys(cli: &Cli, rules: &[RuleMeta]) -> Vec<String> {
    let mut unknown = Vec::new();
    for key in cli.enable.iter().chain(cli.disable.iter()) {
        if !rules.iter().any(|rule| &rule.key == key) && !unknown.contains(key) {
            unknown.push(key.clone());
        }
    }
    unknown
}

/// 构建本次运行使用的 `RuleSelection`。
///
/// 基础集优先级：显式 `--rules` > 共享设置的规则选择 > 默认规则；
/// 然后依序应用 `--disable` 与 `--enable`，最终归一化为规范形式。
pub fn build_selection(
    cli: &Cli,
    rules: &[RuleMeta],
    shared: Option<crate::engine::RuleSelection>,
) -> crate::engine::RuleSelection {
    let mut keys: BTreeSet<String> = match cli.rules_mode {
        Some(RulesMode::All) => rules.iter().map(|rule| rule.key.clone()).collect(),
        Some(RulesMode::Defaults) => rules
            .iter()
            .filter(|rule| rule.default)
            .map(|rule| rule.key.clone())
            .collect(),
        Some(RulesMode::None) => BTreeSet::new(),
        None => shared
            .map(|selection| settings::expand_selection(&selection, rules))
            .unwrap_or_else(|| {
                rules
                    .iter()
                    .filter(|rule| rule.default)
                    .map(|rule| rule.key.clone())
                    .collect()
            }),
    };
    for key in &cli.disable {
        keys.remove(key);
    }
    for key in &cli.enable {
        keys.insert(key.clone());
    }
    settings::canonical_selection(&keys, rules)
}

/// 执行非交互流程，返回进程退出码。
pub fn run_non_interactive(cli: &Cli) -> i32 {
    match run_once(cli) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("文案净排：{message}");
            1
        }
    }
}

fn run_once(cli: &Cli) -> Result<(), String> {
    let rules = crate::engine::default_rules();
    for key in unknown_filter_keys(cli, &rules) {
        eprintln!("警告：未知的规则 key：{key}");
    }

    let shared = if cli.no_config {
        None
    } else {
        settings::load_shared(false).map(|config| config.selection)
    };
    let selection = build_selection(cli, &rules, shared);

    let text = if cli.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("读取标准输入失败：{error}"))?;
        buffer
    } else if let Some(path) = &cli.input {
        fs::read_to_string(path)
            .map_err(|error| format!("读取输入文件 {} 失败：{error}", path.display()))?
    } else {
        String::new()
    };

    let request = FormatRequest { text, selection };
    let output = crate::engine::format_text(&request)?;

    if let Some(path) = &cli.output {
        fs::write(path, output.as_bytes())
            .map_err(|error| format!("写入输出文件 {} 失败：{error}", path.display()))?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(output.as_bytes())
            .and_then(|()| {
                if output.ends_with('\n') {
                    Ok(())
                } else {
                    writeln!(stdout)
                }
            })
            .and_then(|()| stdout.flush())
            .map_err(|error| format!("写入标准输出失败：{error}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::engine::RuleSelection;

    fn flags(args: &[&str]) -> Cli {
        let owned: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        parse(&owned).expect("args should parse")
    }

    #[test]
    fn parses_all_supported_flags() {
        let cli = flags(&[
            "--stdin",
            "--input",
            "in.txt",
            "--output",
            "out.txt",
            "--rules",
            "all",
            "--enable",
            "k1",
            "--disable",
            "k2",
            "--no-config",
        ]);
        assert!(cli.stdin);
        assert_eq!(cli.input.as_deref(), Some(std::path::Path::new("in.txt")));
        assert_eq!(cli.output.as_deref(), Some(std::path::Path::new("out.txt")));
        assert_eq!(cli.rules_mode, Some(RulesMode::All));
        assert_eq!(cli.enable, vec!["k1".to_string()]);
        assert_eq!(cli.disable, vec!["k2".to_string()]);
        assert!(cli.no_config);
        assert!(!cli.help);
    }

    #[test]
    fn missing_flags_require_values() {
        let err = parse(&["--input".to_string()]).expect_err("missing value must fail");
        assert!(err.contains("--input"));
        let err = parse(&["--rules".to_string(), "sometimes".to_string()])
            .expect_err("bad mode must fail");
        assert!(err.contains("all、defaults 或 none"));
        let err = parse(&["--wat".to_string()]).expect_err("unknown flag must fail");
        assert!(err.contains("未知参数"));
    }

    #[test]
    fn non_interactive_only_when_source_or_sink_given() {
        assert!(!flags(&[]).wants_non_interactive());
        assert!(flags(&["--stdin"]).wants_non_interactive());
        assert!(flags(&["--output", "o.txt"]).wants_non_interactive());
    }

    #[test]
    fn rules_mode_overrides_shared_selection() {
        let rules = crate::engine::default_rules();
        let shared = RuleSelection::None;
        match build_selection(&flags(&["--rules", "all"]), &rules, Some(shared)) {
            RuleSelection::All => {}
            other => panic!("expected All, got {other:?}"),
        }
    }

    #[test]
    fn no_mode_no_shared_falls_back_to_defaults() {
        let rules = crate::engine::default_rules();
        match build_selection(&flags(&[]), &rules, None) {
            RuleSelection::Defaults => {}
            other => panic!("expected Defaults, got {other:?}"),
        }
    }

    #[test]
    fn shared_selection_is_used_without_explicit_mode() {
        let rules = crate::engine::default_rules();
        // 共享为“全部关闭”时保持 None，而不是回落默认规则。
        match build_selection(&flags(&[]), &rules, Some(RuleSelection::None)) {
            RuleSelection::None => {}
            other => panic!("expected None, got {other:?}"),
        }
    }

    #[test]
    fn enable_disable_adjust_base_set() {
        let rules = crate::engine::default_rules();
        // 从 defaults 出发关闭全部 → None。
        let disabled: Vec<String> = rules.iter().map(|r| r.key.clone()).collect();
        let mut args = vec!["--rules".to_string(), "defaults".to_string()];
        for key in &disabled {
            args.push("--disable".to_string());
            args.push(key.clone());
        }
        match build_selection(&parse(&args).unwrap(), &rules, None) {
            RuleSelection::None => {}
            other => panic!("expected None, got {other:?}"),
        }
    }

    #[test]
    fn unknown_filter_keys_are_reported_once() {
        let rules = crate::engine::default_rules();
        let cli = flags(&[
            "--enable",
            "ghost-a",
            "--enable",
            "ghost-a",
            "--disable",
            "ghost-b",
        ]);
        assert_eq!(
            unknown_filter_keys(&cli, &rules),
            vec!["ghost-a".to_string(), "ghost-b".to_string()]
        );
    }

    /// 系统临时目录内的唯一文件路径（进程 ID + 计数器），避免并行测试互相覆盖。
    fn temp_cli_file(tag: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "copypolish-cli-{}-{tag}-{n}.txt",
            std::process::id()
        ))
    }

    #[test]
    fn run_once_formats_file_to_file_and_leaves_config_untouched() {
        let input_path = temp_cli_file("input");
        let output_path = temp_cli_file("output");
        fs::write(&input_path, "在LeanCloud上，花了5000元").expect("write input fixture");

        let cli = flags(&[
            "--input",
            &*input_path.to_string_lossy(),
            "--output",
            &*output_path.to_string_lossy(),
            "--no-config",
        ]);
        assert_eq!(run_once(&cli), Ok(()));

        let formatted = fs::read_to_string(&output_path).expect("output file exists");
        assert_eq!(formatted.trim(), "在 LeanCloud 上，花了 5000 元");

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn run_once_reports_missing_input_file_with_exit_code_ready_message() {
        let missing = temp_cli_file("missing");
        let cli = flags(&[
            "--input",
            &*missing.to_string_lossy(),
            "--no-config",
            "--output",
            &*temp_cli_file("never").to_string_lossy(),
        ]);
        let error = run_once(&cli).expect_err("missing file must fail");
        assert!(error.contains("读取输入文件"), "got: {error}");
        assert_eq!(run_non_interactive(&cli), 1);
    }
}
