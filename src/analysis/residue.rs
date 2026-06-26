//! 开发残留分析器。
//!
//! 该模块用于识别常见构建缓存目录，例如 `target`、`node_modules` 和 `.cache`。
//! 目录大小直接从已有扫描结果中累加得到，因此不需要第二次访问文件系统。

use std::collections::HashSet;

use crate::{config::AppConfig, fs::FileEntry};

use super::{Analyzer, Finding, FindingKind, RiskLevel};

#[derive(Debug, Clone)]
pub struct ResidueAnalyzer {
    rules: HashSet<String>,
}

impl ResidueAnalyzer {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            rules: config.residue_dirs.iter().cloned().collect(),
        }
    }

    /// 如果目录名命中配置中的残留规则，则返回命中的规则名。
    fn residue_rule(&self, entry: &FileEntry) -> Option<String> {
        let name = entry.path.file_name()?.to_string_lossy().to_string();
        if entry.is_dir && self.rules.contains(&name) {
            return Some(name);
        }
        None
    }

    /// 估算残留目录大小。
    ///
    /// 这里借用完整扫描结果 `entries`，筛选出位于该目录下的普通文件并求和，
    /// 体现了 Rust 中“借用已有数据、避免重复拷贝和重复 I/O”的设计。
    fn directory_size(&self, dir: &FileEntry, entries: &[FileEntry]) -> u64 {
        entries
            .iter()
            .filter(|entry| entry.is_file() && entry.path.starts_with(&dir.path))
            .map(|entry| entry.size)
            .sum()
    }
}

impl Analyzer for ResidueAnalyzer {
    fn name(&self) -> &'static str {
        "dev-residue"
    }

    fn analyze(&self, entries: &[FileEntry]) -> Vec<Finding> {
        let mut findings: Vec<_> = entries
            .iter()
            .filter_map(|entry| {
                self.residue_rule(entry).map(|rule| Finding {
                    path: entry.path.clone(),
                    kind: FindingKind::DevResidue,
                    size: self.directory_size(entry, entries),
                    risk: RiskLevel::Safe,
                    reason: format!("matched development residue directory rule: {rule}"),
                })
            })
            .collect();
        findings.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.path.cmp(&b.path)));
        findings
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{analysis::Analyzer, config::AppConfig, fs::FileEntry};

    use super::ResidueAnalyzer;

    #[test]
    fn detects_target_directory() {
        let analyzer = ResidueAnalyzer::new(&AppConfig::default());
        let entries = vec![FileEntry::dir(PathBuf::from("project/target"), None, 1)];
        assert_eq!(analyzer.analyze(&entries).len(), 1);
    }

    #[test]
    fn estimates_residue_directory_size() {
        let analyzer = ResidueAnalyzer::new(&AppConfig::default());
        let entries = vec![
            FileEntry::dir(PathBuf::from("project/target"), None, 1),
            FileEntry::file(PathBuf::from("project/target/a.bin"), 10, None, 2),
            FileEntry::file(PathBuf::from("project/target/nested/b.bin"), 15, None, 3),
            FileEntry::file(PathBuf::from("project/src/main.rs"), 100, None, 2),
        ];
        let findings = analyzer.analyze(&entries);
        assert_eq!(findings[0].size, 25);
    }
}
