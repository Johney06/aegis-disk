//! 清理模块入口。
//!
//! `plan` 负责描述“准备怎么清理”，`trash` 负责真正执行或 dry-run。
//! 将计划和执行拆开后，可以先展示计划，再由用户确认是否执行。

pub mod plan;
pub mod trash;

pub use plan::{CleanAction, CleanPlan, CleanSummary};
pub use trash::TrashCleaner;
