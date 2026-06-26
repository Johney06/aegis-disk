//! 重复文件分析器。
//!
//! 重复文件检测不能直接对所有文件做哈希，否则会产生大量磁盘 I/O。
//! 本模块先按大小分组，只有大小相同的文件才继续计算 BLAKE3 哈希。
//! 哈希计算使用 Rayon 并行执行，用来体现 Rust 在系统工具中的性能优势。

use std::{cmp::Reverse, collections::HashMap, fs::File, io::Read, path::PathBuf};

use rayon::prelude::*;

use crate::fs::FileEntry;

use super::{Analyzer, Finding, FindingKind, RiskLevel};

#[derive(Debug, Clone)]
pub struct DuplicateAnalyzer {
    min_size: u64,
}

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub id: usize,
    pub hash: String,
    pub size: u64,
    pub files: Vec<PathBuf>,
    pub recommended_keep: PathBuf,
}

impl DuplicateAnalyzer {
    pub fn new(min_size: u64) -> Self {
        Self { min_size }
    }

    /// 构建重复文件组，采用两阶段策略：
    /// 1. 先按文件大小分组，不同大小的文件不可能重复；
    /// 2. 对候选文件并行计算 BLAKE3 哈希，哈希相同才认为内容重复。
    pub fn groups(&self, entries: &[FileEntry]) -> Vec<DuplicateGroup> {
        let mut by_size: HashMap<u64, Vec<&FileEntry>> = HashMap::new();
        for entry in entries
            .iter()
            .filter(|entry| entry.is_file() && entry.size >= self.min_size)
        {
            by_size.entry(entry.size).or_default().push(entry);
        }

        let candidates: Vec<&FileEntry> = by_size
            .values()
            .filter(|group| group.len() > 1)
            .flat_map(|group| group.iter().copied())
            .collect();

        let hashed: Vec<(String, u64, PathBuf)> = candidates
            .par_iter()
            .filter_map(|entry| {
                hash_file(&entry.path)
                    .ok()
                    .map(|hash| (hash, entry.size, entry.path.clone()))
            })
            .collect();

        let mut by_hash: HashMap<(String, u64), Vec<PathBuf>> = HashMap::new();
        for (hash, size, path) in hashed {
            by_hash.entry((hash, size)).or_default().push(path);
        }

        let mut groups: Vec<_> = by_hash
            .into_iter()
            .filter_map(|((hash, size), mut files)| {
                if files.len() <= 1 {
                    return None;
                }
                // 推荐保留路径层级更浅、路径更短的文件，便于用户理解和恢复。
                files.sort_by_key(|path| (path.components().count(), path.to_string_lossy().len()));
                let recommended_keep = files[0].clone();
                Some((hash, size, files, recommended_keep))
            })
            .enumerate()
            .map(
                |(id, (hash, size, files, recommended_keep))| DuplicateGroup {
                    id,
                    hash,
                    size,
                    files,
                    recommended_keep,
                },
            )
            .collect();

        groups.sort_by_key(|group| Reverse(group.size * group.files.len() as u64));
        groups
    }
}

impl Analyzer for DuplicateAnalyzer {
    fn name(&self) -> &'static str {
        "duplicate"
    }

    fn analyze(&self, entries: &[FileEntry]) -> Vec<Finding> {
        self.groups(entries)
            .into_iter()
            .flat_map(|group| {
                let keep_path = group.recommended_keep.clone();
                group.files.into_iter().map(move |path| {
                    let keep = path == keep_path;
                    Finding {
                        path,
                        kind: FindingKind::DuplicateCandidate {
                            group_id: group.id,
                            keep,
                        },
                        size: group.size,
                        risk: if keep {
                            RiskLevel::Review
                        } else {
                            RiskLevel::Safe
                        },
                        reason: if keep {
                            format!("duplicate group {}, recommended file to keep", group.id)
                        } else {
                            format!("duplicate group {}, same blake3 hash", group.id)
                        },
                    }
                })
            })
            .collect()
    }
}

pub fn hash_file(path: &PathBuf) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    // 固定大小的流式缓冲区可以让超大文件哈希时保持稳定内存占用。
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::{analysis::Analyzer, fs::Scanner};

    use super::DuplicateAnalyzer;

    #[test]
    fn detects_duplicate_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "same").unwrap();
        fs::write(dir.path().join("b.txt"), "same").unwrap();
        fs::write(dir.path().join("c.txt"), "different").unwrap();
        let report = Scanner::new(Default::default()).scan(dir.path());
        let analyzer = DuplicateAnalyzer::new(1);
        let findings = analyzer.analyze(&report.entries);
        assert_eq!(findings.len(), 2);
    }
}
