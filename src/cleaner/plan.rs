//! 清理计划定义。
//!
//! 分析器只负责发现问题，清理模块会把可清理的 `Finding` 转换成具体动作。
//! 这样可以在真正修改文件系统前，把计划完整展示给用户。

use std::path::PathBuf;

use crate::analysis::Finding;

#[derive(Debug, Clone)]
pub enum CleanAction {
    /// 把文件或目录移动到项目内的隔离回收站。
    MoveToTrash {
        from: PathBuf,
        to: PathBuf,
        estimated_bytes: u64,
    },
    /// 显式跳过保护路径，保留 reason 便于输出说明。
    SkipProtected { path: PathBuf, reason: String },
}

#[derive(Debug, Clone)]
pub struct CleanPlan {
    pub root: PathBuf,
    pub actions: Vec<CleanAction>,
}

#[derive(Debug, Default, Clone)]
pub struct CleanSummary {
    pub planned: usize,
    pub moved: usize,
    pub skipped: usize,
    pub failed: usize,
    pub reclaimed_bytes: u64,
    pub messages: Vec<String>,
}

impl CleanPlan {
    /// 将发现项转换成移动计划。
    ///
    /// 目标路径保持相对目录结构，例如 `root/target` 会移动到
    /// `root/.aegis_disk_trash/target`，方便用户手动恢复。
    pub fn from_findings(root: PathBuf, findings: &[Finding], trash_dir: PathBuf) -> Self {
        let actions = findings
            .iter()
            .map(|finding| {
                let relative = finding
                    .path
                    .strip_prefix(&root)
                    .unwrap_or(&finding.path)
                    .to_path_buf();
                CleanAction::MoveToTrash {
                    from: finding.path.clone(),
                    to: trash_dir.join(relative),
                    estimated_bytes: finding.size,
                }
            })
            .collect();
        Self { root, actions }
    }

    pub fn estimated_count(&self) -> usize {
        self.actions.len()
    }

    /// 统计 dry-run 中展示的预计可释放空间。
    pub fn estimated_bytes(&self) -> u64 {
        self.actions
            .iter()
            .map(|action| match action {
                CleanAction::MoveToTrash {
                    estimated_bytes, ..
                } => *estimated_bytes,
                CleanAction::SkipProtected { .. } => 0,
            })
            .sum()
    }
}
