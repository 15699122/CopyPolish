// parity_dump：从 stdin 读入 JSON 数组 [{"text": "...", "enabled": [...] | null}]，
// 逐条调用 Rust 引擎并输出 JSON 数组，供 test/compare_rust_parity.py 与
// Python 引擎输出做逐字对比。
use std::io::{self, Read};

use chinese_copywriting_formatter_lib::rust_engine::format_text;
use chinese_copywriting_formatter_lib::rust_engine::FormatRequest;
use serde_json::Value;

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read stdin");
    let cases: Vec<Value> = serde_json::from_str(&input).expect("invalid json array on stdin");

    let mut outputs: Vec<String> = Vec::with_capacity(cases.len());
    for case in &cases {
        let text = case["text"].as_str().unwrap_or_default().to_string();
        // enabled 缺失/null 表示全部启用（与 Python format_text(text, None) 对齐，
        // Rust 引擎约定 enabled 为空数组即全启用）。
        let enabled: Vec<String> = match case.get("enabled") {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => Vec::new(),
        };
        let result = format_text(&FormatRequest { text, enabled })
            .expect("rust engine failed on parity case");
        outputs.push(result);
    }
    println!(
        "{}",
        serde_json::to_string(&outputs).expect("serialize outputs")
    );
}
