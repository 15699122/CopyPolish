//! 终端剪贴板：基于 OSC 52 转义序列，无需额外系统依赖。
//!
//! 大多数现代终端（Windows Terminal、kitty、alacritty、wezterm 等）支持
//! OSC 52；不支持或禁用该序列的终端会在写入后由用户粘贴得到空内容，
//! 属于可接受的降级行为。超长输出直接拒绝，避免超过终端解析缓冲。

use std::io::Write;

/// 标准 base64 字母表。
const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// 允许复制的最大原始字节数。多数终端对 OSC 52 载荷有约 100 KB 的上限，
/// 这里取更保守的 48 KB（编码后 64 KB），确保常见模拟器都能接收。
pub const MAX_CLIPBOARD_BYTES: usize = 48 * 1024;

/// 纯函数 base64 编码（带填充），便于测试且避免引入新依赖。
pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(BASE64_TABLE[(triple >> 18) as usize & 63] as char);
        out.push(BASE64_TABLE[(triple >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            BASE64_TABLE[(triple >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64_TABLE[triple as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// 将文本编码为完整的 OSC 52 序列；超出大小限制时返回 None。
pub fn encode_osc52(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    if bytes.len() > MAX_CLIPBOARD_BYTES {
        return None;
    }
    Some(format!("\x1b]52;c;{}\x07", base64_encode(bytes)))
}

/// 把 OSC 52 序列写入任意 writer（可注入，便于测试）。
pub fn copy_to(writer: &mut impl Write, text: &str) -> Result<(), String> {
    let sequence = encode_osc52(text).ok_or_else(|| {
        format!(
            "输出过大（{} 字节），终端剪贴板最多支持 {} 字节",
            text.len(),
            MAX_CLIPBOARD_BYTES
        )
    })?;
    writer
        .write_all(sequence.as_bytes())
        .and_then(|()| writer.flush())
        .map_err(|error| format!("写入剪贴板序列失败：{error}"))
}

/// 复制到真实终端（标准输出）。
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    copy_to(&mut std::io::stdout().lock(), text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_preserves_utf8_multibyte_roundtrip_via_decode_shape() {
        // 中文字符是 3 字节 UTF-8；编码长度必须是 4 的倍数。
        let encoded = base64_encode("中文".as_bytes());
        assert_eq!(encoded.len() % 4, 0);
        assert!(!encoded.contains(' '));
    }

    #[test]
    fn osc52_sequence_wraps_base64_payload() {
        let sequence = encode_osc52("在LeanCloud上").expect("small text should fit");
        assert!(sequence.starts_with("\x1b]52;c;"));
        assert!(sequence.ends_with('\x07'));
        assert!(sequence.contains(base64_encode("在LeanCloud上".as_bytes()).as_str()));
    }

    #[test]
    fn oversized_payload_is_rejected_not_written() {
        let oversized = "a".repeat(MAX_CLIPBOARD_BYTES + 1);
        assert!(encode_osc52(&oversized).is_none());
        let mut sink = Vec::new();
        let error = copy_to(&mut sink, &oversized).expect_err("oversized copy must fail");
        assert!(error.contains("输出过大"));
        assert!(sink.is_empty());
    }

    #[test]
    fn copy_writes_full_sequence_to_writer() {
        let mut sink = Vec::new();
        copy_to(&mut sink, "hi").expect("copy should succeed");
        assert_eq!(sink, encode_osc52("hi").unwrap().into_bytes());
    }
}
