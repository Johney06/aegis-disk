//! 扫描结果的数据结构定义。
//!
//! 文件系统扫描会产生大量条目，本模块用结构体把路径、大小、目录层级等信息
//! 固定下来，后续分析器只需要借用这些数据即可，不需要再次访问磁盘。

use std::{path::PathBuf, time::SystemTime};

#[derive(Debug, Clone)]
pub struct FileEntry {
    /// 文件或目录的完整路径。
    pub path: PathBuf,
    /// 文件大小；目录本身大小统一记为 0，目录真实占用由分析器按子文件累加。
    pub size: u64,
    /// 修改时间可能因为权限或文件系统差异无法读取，所以使用 Option 表示。
    pub modified: Option<SystemTime>,
    /// 用布尔值区分文件和目录，便于后续快速过滤。
    pub is_dir: bool,
    /// 相对扫描根目录的深度，可用于展示或限制扫描层级。
    pub depth: usize,
}

impl FileEntry {
    pub fn file(path: PathBuf, size: u64, modified: Option<SystemTime>, depth: usize) -> Self {
        Self {
            path,
            size,
            modified,
            is_dir: false,
            depth,
        }
    }

    pub fn dir(path: PathBuf, modified: Option<SystemTime>, depth: usize) -> Self {
        Self {
            path,
            size: 0,
            modified,
            is_dir: true,
            depth,
        }
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir
    }
}

#[derive(Debug, Default, Clone)]
pub struct ScanStats {
    pub files: usize,
    pub dirs: usize,
    pub total_size: u64,
    pub errors: usize,
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub root: PathBuf,
    pub entries: Vec<FileEntry>,
    pub stats: ScanStats,
    pub errors: Vec<String>,
}

impl ScanReport {
    /// 根据扫描条目自动汇总统计信息。
    ///
    /// `saturating_add` 可以避免极端情况下文件大小累加溢出。
    pub fn new(root: PathBuf, entries: Vec<FileEntry>, errors: Vec<String>) -> Self {
        let mut stats = ScanStats::default();
        for entry in &entries {
            if entry.is_dir {
                stats.dirs += 1;
            } else {
                stats.files += 1;
                stats.total_size = stats.total_size.saturating_add(entry.size);
            }
        }
        stats.errors = errors.len();
        Self {
            root,
            entries,
            stats,
            errors,
        }
    }
}
