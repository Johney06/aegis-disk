//! 项目自定义错误类型。
//!
//! 使用 `thiserror` 可以给错误枚举自动实现 `std::error::Error`，
//! 同时保留清晰的错误分类，方便上层决定如何展示或处理。

use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SentinelError {
    /// 用户尝试清理系统保护路径。
    #[error("path is protected and cannot be cleaned: {0}")]
    ProtectedPath(PathBuf),
    /// 大小表达式解析失败，例如无法识别的单位。
    #[error("invalid size expression: {0}")]
    InvalidSize(String),
    /// 扫描入口不存在，通常是用户命令中的路径写错。
    #[error("scan root does not exist: {0}")]
    ScanRootNotFound(PathBuf),
    /// 扫描入口不是目录，避免把单个文件误当成根目录处理。
    #[error("scan root is not a directory: {0}")]
    ScanRootNotDirectory(PathBuf),
    /// clean 命令必须明确选择 dry-run 或 execute。
    #[error("clean requires either --dry-run or --execute")]
    MissingCleanMode,
    /// dry-run 和 execute 互斥，不能同时使用。
    #[error("--dry-run and --execute cannot be used together")]
    ConflictingCleanMode,
    /// 带路径上下文的 I/O 错误，便于定位具体失败文件。
    #[error("io error at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

pub type SentinelResult<T> = Result<T, SentinelError>;
