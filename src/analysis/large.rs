//! 大文件分析器。
//!
//! 大文件不一定能安全删除，因此本模块只负责发现和提示，风险等级标记为
//! `Review`，最终清理规则不会自动清理大文件。

use crate::fs::FileEntry;

use super::{Analyzer, Finding, FindingKind, RiskLevel};

#[derive(Debug, Clone)]
pub struct LargeFileAnalyzer {
    min_size: u64,
}

impl LargeFileAnalyzer {
    pub fn new(min_size: u64) -> Self {
        Self { min_size }
    }
}

impl Analyzer for LargeFileAnalyzer {
    fn name(&self) -> &'static str {
        "large-file"
    }

    fn analyze(&self, entries: &[FileEntry]) -> Vec<Finding> {
        let mut findings: Vec<_> = entries
            .iter()
            // 只分析普通文件，目录大小由其他模块单独估算。
            .filter(|entry| entry.is_file() && entry.size >= self.min_size)
            .map(|entry| Finding {
                path: entry.path.clone(),
                kind: FindingKind::LargeFile,
                size: entry.size,
                risk: RiskLevel::Review,
                reason: format!(
                    "file size is greater than or equal to {} bytes",
                    self.min_size
                ),
            })
            .collect();
        // 大文件按体积从大到小排列，便于用户优先查看最占空间的文件。
        findings.sort_by(|a, b| b.size.cmp(&a.size));
        findings
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{analysis::Analyzer, fs::FileEntry};

    use super::LargeFileAnalyzer;

    #[test]
    fn detects_large_files() {
        let entries = vec![FileEntry::file(PathBuf::from("a.bin"), 10, None, 0)];
        let analyzer = LargeFileAnalyzer::new(5);
        assert_eq!(analyzer.analyze(&entries).len(), 1);
    }
}
