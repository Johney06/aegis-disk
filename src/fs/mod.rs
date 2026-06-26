//! 文件系统相关模块入口。
//!
//! 这里统一导出扫描器、扫描报告和安全保护器，其他模块可以直接通过
//! `crate::fs::Scanner` 等名称使用，减少路径层级暴露。

pub mod metadata;
pub mod safety;
pub mod scanner;

pub use metadata::{FileEntry, ScanReport, ScanStats};
pub use safety::SafetyGuard;
pub use scanner::Scanner;
