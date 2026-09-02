// unicode 基线测量示例（仅本地/CI 手动运行，不参与打包）：
//   cargo run --release --manifest-path src-tauri/Cargo.toml --example unicode_baseline
//
// 按 10 KB / 100 KB / 1 MB × {纯中文、中英数混排、Markdown/URL/LaTeX 密集、
// emoji 与组合字符密集、CJK 扩展区密集} 输出 format_text 耗时，
// 用于记录引入 unicode-segmentation 前后的性能对比（见 docs/benchmarks/unicode-baseline.md）。

use chinese_copywriting_formatter_lib::engine::{format_text, FormatRequest, RuleSelection};
use std::time::Instant;

fn repeat_to_size(seed: &str, target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + seed.len());
    while out.len() < target_bytes {
        out.push_str(seed);
    }
    out
}

fn bench(name: &str, text: &str) {
    // 预热 + 多轮取平均，降低单次噪声。
    let request = FormatRequest {
        text: text.to_string(),
        selection: RuleSelection::All,
        ..Default::default()
    };
    let _ = format_text(&request);
    const ROUNDS: u32 = 5;
    let start = Instant::now();
    for _ in 0..ROUNDS {
        let request = FormatRequest {
            text: text.to_string(),
            selection: RuleSelection::All,
            ..Default::default()
        };
        if format_text(&request).is_err() {
            println!("{name:>28}: ERR (engine regex backtracking limit)");
            return;
        }
    }
    let avg_ms = start.elapsed().as_millis() as f64 / f64::from(ROUNDS);
    println!("{name:>28}: {avg_ms:8.2} ms/round");
}

fn main() {
    println!("=== unicode baseline ===");
    for size in [10 * 1024, 100 * 1024, 1024 * 1024] {
        println!("--- {} KB ---", size / 1024);
        let mixed_seed = "中文Mixed文本123与English混排，数字456789结尾。";
        let markup_seed = "请看[链接](https://example.com/a;b?x=$1|y)和公式 $E=mc^2$ 以及 `code`。";
        let emoji_seed = "表情👍🏽家庭👨‍👩‍👧‍👦组合é（e+U+0301）序列。";
        let extb_seed = "扩展区汉字𠀀𠮷野家与LeanCloud混排。";
        for (label, seed) in [
            ("纯中文", "这是一段纯中文的文案内容，没有任何其他字符。"),
            ("中英数混排", mixed_seed),
            ("Markdown/LaTeX 密集", markup_seed),
            ("emoji/组合字符密集", emoji_seed),
            ("CJK Ext-B 密集", extb_seed),
        ] {
            bench(
                &format!("{label} @{}KB", size / 1024),
                &repeat_to_size(seed, size),
            );
        }
    }
}
