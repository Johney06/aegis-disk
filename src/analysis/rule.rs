//! 清理规则模块。
//!
//! 分析器会发现多种问题，但不是所有发现项都应该自动清理。例如大文件需要
//! 人工判断，重复文件中推荐保留的文件也不能清理。本模块负责从发现项中过滤
//! 真正可以进入清理计划的候选项。

use crate::analysis::{Finding, FindingKind, RiskLevel};

pub fn cleanable_findings(findings: &[Finding]) -> Vec<Finding> {
    findings
        .iter()
        .filter(|finding| match &finding.kind {
            // 开发残留通常可以重新生成，且必须是 Safe 才加入清理计划。
            FindingKind::DevResidue => finding.risk == RiskLevel::Safe,
            // 重复文件只清理非保留项，保留项用于防止整组文件被全部移走。
            FindingKind::DuplicateCandidate { keep, .. } => {
                !keep && finding.risk != RiskLevel::Dangerous
            }
            // 大文件只提示不自动清理，避免误删用户重要数据。
            FindingKind::LargeFile => false,
        })
        .cloned()
        .collect()
}
