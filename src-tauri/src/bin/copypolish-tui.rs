#![cfg(feature = "tui")]

use chinese_copywriting_formatter_lib::tui::{cli, settings, App};

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let parsed = match cli::parse(&args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("文案净排：{error}\n\n{}", cli::usage());
            return std::process::ExitCode::from(2);
        }
    };

    if parsed.help {
        println!("{}", cli::usage());
        return std::process::ExitCode::SUCCESS;
    }

    // 非交互模式：stdin / 文件 → 引擎 → stdout / 文件，不进入 raw mode。
    if parsed.wants_non_interactive() {
        return std::process::ExitCode::from(cli::run_non_interactive(&parsed) as u8);
    }

    // 交互模式：优先使用共享设置中的规则与最近输入（--no-config 时跳过）。
    let shared = settings::load_shared(parsed.no_config);
    if let Err(error) =
        chinese_copywriting_formatter_lib::tui::run(App::with_config(shared, parsed.no_config))
    {
        eprintln!("文案净排：终端界面运行失败：{error}");
        return std::process::ExitCode::from(1);
    }
    std::process::ExitCode::SUCCESS
}
