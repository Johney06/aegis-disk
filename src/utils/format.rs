//! 格式化工具。
//!
//! CLI 和 TUI 都需要把字节数显示成人类可读格式，同时也需要解析用户输入的
//! `100MB`、`1GiB` 等大小表达式，因此将这些逻辑集中在本模块。

use humansize::{DECIMAL, format_size};

use crate::error::{SentinelError, SentinelResult};

/// 将字节数格式化为十进制单位，例如 `1000000` -> `1 MB`。
pub fn bytes(size: u64) -> String {
    format_size(size, DECIMAL)
}

/// 解析用户输入的大小字符串。
///
/// 支持十进制单位 `KB/MB/GB` 和二进制单位 `KiB/MiB/GiB`。
/// 返回 `Result` 是为了把非法输入交给上层统一展示错误。
pub fn parse_size(input: &str) -> SentinelResult<u64> {
    let trimmed = input.trim().to_ascii_lowercase();
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit() && ch != '.')
        .unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(split_at);
    let value: f64 = number
        .parse()
        .map_err(|_| SentinelError::InvalidSize(input.to_owned()))?;
    let multiplier = match unit.trim() {
        "" | "b" => 1_f64,
        "k" | "kb" => 1_000_f64,
        "m" | "mb" => 1_000_000_f64,
        "g" | "gb" => 1_000_000_000_f64,
        "t" | "tb" => 1_000_000_000_000_f64,
        "ki" | "kib" => 1024_f64,
        "mi" | "mib" => 1024_f64.powi(2),
        "gi" | "gib" => 1024_f64.powi(3),
        other => return Err(SentinelError::InvalidSize(other.to_owned())),
    };
    Ok((value * multiplier) as u64)
}

#[cfg(test)]
mod tests {
    use super::parse_size;

    #[test]
    fn parses_megabytes() {
        assert_eq!(parse_size("2MB").unwrap(), 2_000_000);
    }
}
