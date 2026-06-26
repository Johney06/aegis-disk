//! 文件类型分布分析器。
//!
//! 该模块按照文件扩展名统计数量和占用空间，用于回答“磁盘空间主要被哪类文件占用”。
//! 它不是清理模块，而是辅助用户理解目录内容结构，适合作为磁盘分析工具的补充功能。

use std::{cmp::Reverse, collections::HashMap, path::Path};

use crate::fs::FileEntry;

/// 单个文件类型的统计结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTypeStat {
    /// 扩展名，例如 `rs`、`toml`、`bin`；无扩展名文件统一记为 `[no extension]`。
    pub extension: String,
    /// 该类型文件数量。
    pub files: usize,
    /// 该类型文件累计大小。
    pub total_size: u64,
    /// 该类型中最大的单个文件路径，用于帮助用户定位空间来源。
    pub largest_file: Option<std::path::PathBuf>,
    /// 最大单个文件大小。
    pub largest_size: u64,
}

impl FileTypeStat {
    pub fn average_size(&self) -> u64 {
        if self.files == 0 {
            0
        } else {
            self.total_size / self.files as u64
        }
    }

    pub fn display_extension(&self) -> &str {
        &self.extension
    }
}

#[derive(Debug, Default, Clone)]
struct FileTypeAccumulator {
    files: usize,
    total_size: u64,
    largest_file: Option<std::path::PathBuf>,
    largest_size: u64,
}

impl FileTypeAccumulator {
    fn add_file(&mut self, entry: &FileEntry) {
        self.files += 1;
        self.total_size = self.total_size.saturating_add(entry.size);
        if entry.size >= self.largest_size {
            self.largest_size = entry.size;
            self.largest_file = Some(entry.path.clone());
        }
    }

    fn into_stat(self, extension: String) -> FileTypeStat {
        FileTypeStat {
            extension,
            files: self.files,
            total_size: self.total_size,
            largest_file: self.largest_file,
            largest_size: self.largest_size,
        }
    }
}

/// 文件类型分析器。
#[derive(Debug, Clone)]
pub struct FileTypeAnalyzer {
    include_hidden: bool,
}

impl Default for FileTypeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTypeAnalyzer {
    pub fn new() -> Self {
        Self {
            include_hidden: true,
        }
    }

    pub fn without_hidden_files() -> Self {
        Self {
            include_hidden: false,
        }
    }

    /// 按扩展名聚合文件统计结果，并按累计大小从大到小排序。
    pub fn analyze(&self, entries: &[FileEntry]) -> Vec<FileTypeStat> {
        let mut map: HashMap<String, FileTypeAccumulator> = HashMap::new();
        for entry in entries.iter().filter(|entry| entry.is_file()) {
            if !self.include_hidden && is_hidden_file(&entry.path) {
                continue;
            }
            let extension = extension_label(&entry.path);
            map.entry(extension).or_default().add_file(entry);
        }

        let mut stats: Vec<_> = map
            .into_iter()
            .map(|(extension, accumulator)| accumulator.into_stat(extension))
            .collect();
        stats.sort_by_key(|stat| Reverse((stat.total_size, stat.files)));
        stats
    }

    pub fn top_n(&self, entries: &[FileEntry], limit: usize) -> Vec<FileTypeStat> {
        self.analyze(entries).into_iter().take(limit).collect()
    }
}

fn extension_label(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| !ext.trim().is_empty())
        .unwrap_or_else(|| "[no extension]".to_owned())
}

fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::fs::FileEntry;

    use super::FileTypeAnalyzer;

    #[test]
    fn groups_files_by_extension() {
        let entries = vec![
            FileEntry::file(PathBuf::from("src/main.rs"), 100, None, 1),
            FileEntry::file(PathBuf::from("src/lib.rs"), 50, None, 1),
            FileEntry::file(PathBuf::from("Cargo.toml"), 30, None, 0),
        ];
        let stats = FileTypeAnalyzer::new().analyze(&entries);
        let rust = stats.iter().find(|stat| stat.extension == "rs").unwrap();
        assert_eq!(rust.files, 2);
        assert_eq!(rust.total_size, 150);
    }

    #[test]
    fn handles_files_without_extension() {
        let entries = vec![FileEntry::file(PathBuf::from("LICENSE"), 10, None, 0)];
        let stats = FileTypeAnalyzer::new().analyze(&entries);
        assert_eq!(stats[0].extension, "[no extension]");
    }
}
