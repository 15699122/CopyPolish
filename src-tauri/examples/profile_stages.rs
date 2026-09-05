// 分阶段性能剖析示例（仅本地手动运行，不参与打包/门禁）：
//   cargo run --release --manifest-path src-tauri/Cargo.toml \
//     --features profile-stages --example profile_stages
//
// 输出 Markdown/LaTeX 密集语料在 format_text 各阶段的平均耗时与占比，
// 用于决策 2（placeholder 重构）的 P1.2 profiling 基线；结果应记录在
// docs/benchmarks/unicode-baseline.md 的分阶段剖析章节。

use chinese_copywriting_formatter_lib::engine::{
    format_text_stage_timings, per_rule_timings, scan_split_timings, scan_structure_timings,
    FormatRequest, RuleSelection,
};
use std::time::Instant;

fn repeat_to_size(seed: &str, target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + seed.len());
    while out.len() < target_bytes {
        out.push_str(seed);
    }
    out
}

fn main() {
    const SIZE: usize = 1024 * 1024;
    const ROUNDS: u32 = 5;
    let corpora: [(&str, &str); 2] = [
        (
            "Markdown/LaTeX 密集",
            "请看[链接](https://example.com/a;b?x=$1|y)和公式 $E=mc^2$ 以及 `code`。",
        ),
        (
            "纯中文（对照）",
            "这是一段纯中文的文案内容，没有任何其他字符。",
        ),
    ];

    println!("=== 分阶段剖析 @{}KB，{} 轮平均 ===", SIZE / 1024, ROUNDS);
    for (label, seed) in corpora {
        let text = repeat_to_size(seed, SIZE);
        let request = FormatRequest {
            text: text.clone(),
            selection: RuleSelection::All,
            ..Default::default()
        };
        // 预热。
        let _ = format_text_stage_timings(&request);

        let mut sums: Vec<(&'static str, u128)> = Vec::new();
        let mut total = 0u128;
        for _ in 0..ROUNDS {
            let request = FormatRequest {
                text: text.clone(),
                selection: RuleSelection::All,
                ..Default::default()
            };
            let stage_start = Instant::now();
            let timings = match format_text_stage_timings(&request) {
                Ok(t) => t,
                Err(e) => {
                    println!("{label}: ERR {e}");
                    return;
                }
            };
            total += stage_start.elapsed().as_nanos();
            for (name, d) in timings {
                let entry = sums.iter_mut().find(|(n, _)| *n == name);
                match entry {
                    Some((_, v)) => *v += d.as_nanos(),
                    None => sums.push((name, d.as_nanos())),
                }
            }
        }

        println!("\n--- {label} ---");
        for (name, v) in &sums {
            let avg_ms = *v as f64 / f64::from(ROUNDS) / 1_000_000.0;
            let pct = 100.0 * *v as f64 / total as f64;
            println!("  {name:<22} {avg_ms:9.2} ms  {pct:5.1}%");
        }
        println!(
            "  {:<22} {:9.2} ms",
            "TOTAL",
            total as f64 / f64::from(ROUNDS) / 1_000_000.0
        );
    }

    // ---- 二级归因（Markdown/LaTeX 密集 @1MB） ----
    let md_text = repeat_to_size(corpora[0].1, SIZE);

    println!("\n=== 二级归因：扫描拆分（1 轮） ===");
    for (name, d) in scan_split_timings(&md_text) {
        println!("  {name:<22} {:9.2} ms", d.as_secs_f64() * 1000.0);
    }

    println!("\n=== 三级归因：结构扫描器逐个计时（1 轮，降序） ===");
    let mut scanners: Vec<(&'static str, f64)> = scan_structure_timings(&md_text)
        .into_iter()
        .map(|(name, d)| (name, d.as_secs_f64() * 1000.0))
        .collect();
    scanners.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (name, ms) in &scanners {
        println!("  {name:<22} {ms:9.2} ms");
    }

    println!("\n=== 二级归因：逐规则计时（整篇应用，1 轮） ===");
    let request = FormatRequest {
        text: md_text,
        selection: RuleSelection::All,
        ..Default::default()
    };
    let mut rules: Vec<(&'static str, f64)> = per_rule_timings(&request)
        .into_iter()
        .map(|(key, d)| (key, d.as_secs_f64() * 1000.0))
        .collect();
    rules.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (key, ms) in &rules {
        println!("  {key:<38} {ms:9.2} ms");
    }
}
