//! 文件系统扫描器。
//!
//! 该模块把真实目录树转换为内存中的 `FileEntry` 列表。扫描过程中如果遇到
//! 权限不足或元数据读取失败，不会直接终止程序，而是把错误记录到报告中。
//! 这样更符合真实磁盘环境：少数文件失败不应该影响整体分析。

use std::path::Path;

use walkdir::WalkDir;

use crate::{config::AppConfig, fs::metadata::FileEntry};

use super::ScanReport;

#[derive(Debug, Clone)]
pub struct Scanner {
    config: AppConfig,
}

impl Scanner {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// 遍历目录树，并同时返回成功读取的条目和可恢复错误。
    ///
    /// `follow_links(false)` 表示不跟随符号链接，可以避免扫描过程通过软链接
    /// 意外跳出用户指定的根目录，这对磁盘清理工具尤其重要。
    pub fn scan(&self, root: &Path) -> ScanReport {
        let mut entries = Vec::new();
        let mut errors = Vec::new();
        let walker = WalkDir::new(root).follow_links(false);
        let walker = if let Some(max_depth) = self.config.max_depth {
            walker.max_depth(max_depth)
        } else {
            walker
        };

        for item in walker.into_iter().filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            // `filter_entry` 可以在进入子目录前跳过整棵目录树，比扫描后再过滤更省 I/O。
            !self
                .config
                .ignore_dirs
                .iter()
                .any(|ignored| ignored == &name)
        }) {
            match item {
                Ok(dir_entry) => match dir_entry.metadata() {
                    Ok(metadata) => {
                        let depth = dir_entry.depth();
                        let path = dir_entry.path().to_path_buf();
                        let modified = metadata.modified().ok();
                        if metadata.is_dir() {
                            entries.push(FileEntry::dir(path, modified, depth));
                        } else if metadata.is_file() {
                            entries.push(FileEntry::file(path, metadata.len(), modified, depth));
                        }
                    }
                    Err(err) => errors.push(format!(
                        "metadata failed for {}: {err}",
                        dir_entry.path().display()
                    )),
                },
                Err(err) => errors.push(err.to_string()),
            }
        }

        ScanReport::new(root.to_path_buf(), entries, errors)
    }
}
