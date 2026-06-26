//! 命令行参数定义模块。
//!
//! 本项目使用 `clap` 的 derive 宏声明命令行结构。这样每个子命令的参数
//! 都有明确的 Rust 类型，比手动解析字符串更安全，也更容易扩展。

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aegis-disk")]
#[command(
    author,
    version,
    about = "A Rust terminal disk intelligence and safe cleanup tool"
)]
pub struct Cli {
    /// 可选的 TOML 配置文件路径。
    ///
    /// `global = true` 表示这个参数可以放在任意子命令前面，例如：
    /// `aegis-disk --config aegis-disk.toml scan .`。
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// 具体要执行的子命令。
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

/// 程序支持的所有子命令。
///
/// 使用枚举表达命令类型，可以让 `app` 层用 `match` 分发业务逻辑，
/// 同时避免无效参数组合在运行期到处传播。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// 扫描目录并输出文件、目录和总体积统计。
    Scan {
        path: PathBuf,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// 查找超过指定阈值的大文件。
    Large {
        path: PathBuf,
        #[arg(long, default_value = "100MB")]
        min_size: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// 识别开发残留目录，例如 target、node_modules、.cache 等。
    Residue {
        path: PathBuf,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// 检测重复文件，并给出推荐保留项。
    Duplicates {
        path: PathBuf,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// 按文件扩展名统计文件数量和占用空间。
    Types {
        path: PathBuf,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// 根据扫描结果生成诊断建议。
    Insights {
        path: PathBuf,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// 导出 Markdown 或 JSON 格式的扫描报告。
    Export {
        path: PathBuf,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// 根据分析结果生成清理计划，支持 dry-run 和真正执行。
    Clean {
        path: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        execute: bool,
        /// 跳过交互式二次确认，适合明确知道风险的脚本或演示场景。
        #[arg(long)]
        yes: bool,
        #[arg(long, default_value = "residue")]
        target: String,
    },
    /// 配置相关命令。
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// 启动三看板终端 UI。
    Tui { path: PathBuf },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// 打印默认配置，用户可以重定向保存为 aegis-disk.toml。
    PrintDefault,
}
