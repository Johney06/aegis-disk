//! 安全清理执行模块。
//!
//! 本项目不直接永久删除文件，而是把候选项移动到项目本地的
//! `.aegis_disk_trash/` 目录中。这样即使用户误操作，也可以手动恢复。

use std::{fs, path::PathBuf};

use crate::{
    cleaner::{CleanAction, CleanPlan, CleanSummary},
    fs::SafetyGuard,
};

#[derive(Debug, Clone)]
pub struct TrashCleaner {
    guard: SafetyGuard,
}

impl TrashCleaner {
    pub fn new(guard: SafetyGuard) -> Self {
        Self { guard }
    }

    /// 仿真执行清理计划，不修改文件系统。
    /// 返回的 summary 会包含预计释放空间，便于用户在真正清理前确认。
    pub fn dry_run(&self, plan: &CleanPlan) -> CleanSummary {
        let mut summary = CleanSummary {
            planned: plan.actions.len(),
            ..CleanSummary::default()
        };
        for action in &plan.actions {
            match action {
                CleanAction::MoveToTrash {
                    from,
                    to,
                    estimated_bytes,
                } => {
                    if self.guard.is_protected(from) {
                        summary.skipped += 1;
                        summary
                            .messages
                            .push(format!("SKIP protected path: {}", from.display()));
                    } else {
                        summary.reclaimed_bytes =
                            summary.reclaimed_bytes.saturating_add(*estimated_bytes);
                        summary.messages.push(format!(
                            "DRY-RUN move {} -> {} ({})",
                            from.display(),
                            to.display(),
                            crate::utils::format::bytes(*estimated_bytes)
                        ));
                    }
                }
                CleanAction::SkipProtected { path, reason } => {
                    summary.skipped += 1;
                    summary
                        .messages
                        .push(format!("SKIP {}: {reason}", path.display()));
                }
            }
        }
        summary
    }

    /// 真正执行清理计划，把文件或目录移动到回收站。
    /// 即使上层已经过滤过路径，这里仍再次检查保护路径，作为最后一道安全防线。
    pub fn execute(&self, plan: &CleanPlan) -> CleanSummary {
        let mut summary = CleanSummary {
            planned: plan.actions.len(),
            ..CleanSummary::default()
        };
        for action in &plan.actions {
            match action {
                CleanAction::MoveToTrash { from, to, .. } => {
                    if self.guard.is_protected(from) {
                        summary.skipped += 1;
                        summary
                            .messages
                            .push(format!("SKIP protected path: {}", from.display()));
                        continue;
                    }
                    match move_path(from, to) {
                        Ok(bytes) => {
                            summary.moved += 1;
                            summary.reclaimed_bytes = summary.reclaimed_bytes.saturating_add(bytes);
                            summary.messages.push(format!(
                                "MOVED {} -> {}",
                                from.display(),
                                to.display()
                            ));
                        }
                        Err(err) => {
                            summary.failed += 1;
                            summary.messages.push(format!(
                                "FAILED {} -> {}: {err}",
                                from.display(),
                                to.display()
                            ));
                        }
                    }
                }
                CleanAction::SkipProtected { path, reason } => {
                    summary.skipped += 1;
                    summary
                        .messages
                        .push(format!("SKIP {}: {reason}", path.display()));
                }
            }
        }
        summary
    }
}

fn move_path(from: &PathBuf, to: &PathBuf) -> std::io::Result<u64> {
    // 移动前先计算大小，移动成功后用于统计本次释放的空间。
    let bytes = path_size(from).unwrap_or(0);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(from, to)?;
    Ok(bytes)
}

fn path_size(path: &PathBuf) -> std::io::Result<u64> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut size = 0_u64;
    for item in walkdir::WalkDir::new(path) {
        let item = item?;
        let metadata = item.metadata()?;
        if metadata.is_file() {
            size = size.saturating_add(metadata.len());
        }
    }
    Ok(size)
}
