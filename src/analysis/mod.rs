//! 分析模块公共定义。
//!
//! 所有具体分析器都会接收扫描得到的 `FileEntry` 切片，并输出统一的 `Finding`。
//! 这种 trait 设计让大文件分析、残留目录分析和重复文件分析可以被统一调度。

pub mod duplicate;
pub mod large;
pub mod residue;
pub mod rule;

use std::path::PathBuf;

use crate::fs::FileEntry;

/// 风险等级用于提示用户是否可以直接清理。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// 通常是缓存或构建产物，相对安全。
    Safe,
    /// 需要人工复核，例如大文件不一定能删除。
    Review,
    /// 高风险路径，默认不允许清理。
    Dangerous,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Safe => "SAFE",
            Self::Review => "REVIEW",
            Self::Dangerous => "DANGER",
        }
    }
}

/// 发现项类型，不同类型会影响清理规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    LargeFile,
    DevResidue,
    DuplicateCandidate { group_id: usize, keep: bool },
}

/// 分析器输出的统一结构。
#[derive(Debug, Clone)]
pub struct Finding {
    pub path: PathBuf,
    pub kind: FindingKind,
    pub size: u64,
    pub risk: RiskLevel,
    pub reason: String,
}

/// 统一分析器接口。
///
/// `entries` 使用切片借用，避免每个分析器复制一份完整扫描结果。
pub trait Analyzer {
    fn name(&self) -> &'static str;
    fn analyze(&self, entries: &[FileEntry]) -> Vec<Finding>;
}

pub use duplicate::{DuplicateAnalyzer, DuplicateGroup};
pub use large::LargeFileAnalyzer;
pub use residue::ResidueAnalyzer;
