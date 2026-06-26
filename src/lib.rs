//! 库入口文件。
//!
//! 项目同时包含二进制入口和库入口。把各模块从 `lib.rs` 导出后，
//! 集成测试可以直接调用扫描器、分析器和清理器，而不需要通过命令行间接测试。

pub mod analysis;
pub mod app;
pub mod cleaner;
pub mod cli;
pub mod config;
pub mod error;
pub mod fs;
pub mod tui;
pub mod utils;
