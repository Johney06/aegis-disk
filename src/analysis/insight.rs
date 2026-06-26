//! 诊断建议模块。
//!
//! `InsightAnalyzer` 不直接产生清理项，而是把扫描报告和分析结果转化为人类可读建议。
//! 它适合在报告导出和命令行摘要中使用，帮助用户决定下一步应该先看哪类问题。

use crate::{
    analysis::{Finding, FindingKind, RiskLevel},
    fs::ScanReport,
    utils::format::bytes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsightSeverity {
    Info,
    Notice,
    Warning,
}

impl InsightSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Notice => "NOTICE",
            Self::Warning => "WARNING",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskInsight {
    pub severity: InsightSeverity,
    pub title: String,
    pub message: String,
    pub suggested_command: Option<String>,
}

impl DiskInsight {
    pub fn new(
        severity: InsightSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
        suggested_command: Option<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            message: message.into(),
            suggested_command,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsightAnalyzer {
    large_file_ratio_threshold: f64,
    residue_ratio_threshold: f64,
    duplicate_ratio_threshold: f64,
}

impl Default for InsightAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl InsightAnalyzer {
    pub fn new() -> Self {
        Self {
            large_file_ratio_threshold: 0.35,
            residue_ratio_threshold: 0.15,
            duplicate_ratio_threshold: 0.10,
        }
    }

    pub fn analyze(&self, report: &ScanReport, findings: &[Finding]) -> Vec<DiskInsight> {
        let mut insights = Vec::new();
        self.push_scan_health(report, &mut insights);
        self.push_risk_summary(report, findings, &mut insights);
        self.push_large_file_advice(report, findings, &mut insights);
        self.push_residue_advice(report, findings, &mut insights);
        self.push_duplicate_advice(report, findings, &mut insights);
        self.push_next_step(report, findings, &mut insights);
        insights
    }

    fn push_scan_health(&self, report: &ScanReport, insights: &mut Vec<DiskInsight>) {
        if report.stats.errors == 0 {
            insights.push(DiskInsight::new(
                InsightSeverity::Info,
                "Scan completed without access errors",
                "All accessible entries were scanned successfully.",
                None,
            ));
        } else {
            insights.push(DiskInsight::new(
                InsightSeverity::Warning,
                "Some entries could not be scanned",
                format!(
                    "The scanner recorded {} access error(s). Results are still usable, but totals may be incomplete.",
                    report.stats.errors
                ),
                Some(format!("aegis-disk scan {}", report.root.display())),
            ));
        }
    }

    fn push_risk_summary(
        &self,
        report: &ScanReport,
        findings: &[Finding],
        insights: &mut Vec<DiskInsight>,
    ) {
        let safe = findings
            .iter()
            .filter(|item| item.risk == RiskLevel::Safe)
            .count();
        let review = findings
            .iter()
            .filter(|item| item.risk == RiskLevel::Review)
            .count();
        let dangerous = findings
            .iter()
            .filter(|item| item.risk == RiskLevel::Dangerous)
            .count();
        insights.push(DiskInsight::new(
            InsightSeverity::Notice,
            "Finding risk distribution",
            format!(
                "Detected {safe} safe item(s), {review} review item(s), and {dangerous} dangerous item(s) under {}.",
                report.root.display()
            ),
            None,
        ));
    }

    fn push_large_file_advice(
        &self,
        report: &ScanReport,
        findings: &[Finding],
        insights: &mut Vec<DiskInsight>,
    ) {
        let large_bytes: u64 = findings
            .iter()
            .filter(|item| matches!(item.kind, FindingKind::LargeFile))
            .map(|item| item.size)
            .sum();
        if large_bytes == 0 {
            insights.push(DiskInsight::new(
                InsightSeverity::Info,
                "No large-file pressure detected",
                "Large files do not dominate this scan result.",
                None,
            ));
            return;
        }
        let ratio = ratio(large_bytes, report.stats.total_size);
        let severity = if ratio >= self.large_file_ratio_threshold {
            InsightSeverity::Warning
        } else {
            InsightSeverity::Notice
        };
        insights.push(DiskInsight::new(
            severity,
            "Large files deserve manual review",
            format!(
                "Large-file candidates account for about {:.1}% of scanned space ({}).",
                ratio * 100.0,
                bytes(large_bytes)
            ),
            Some(format!("aegis-disk large {}", report.root.display())),
        ));
    }

    fn push_residue_advice(
        &self,
        report: &ScanReport,
        findings: &[Finding],
        insights: &mut Vec<DiskInsight>,
    ) {
        let residue_bytes: u64 = findings
            .iter()
            .filter(|item| matches!(item.kind, FindingKind::DevResidue))
            .map(|item| item.size)
            .sum();
        if residue_bytes == 0 {
            return;
        }
        let ratio = ratio(residue_bytes, report.stats.total_size);
        let severity = if ratio >= self.residue_ratio_threshold {
            InsightSeverity::Notice
        } else {
            InsightSeverity::Info
        };
        insights.push(DiskInsight::new(
            severity,
            "Development residue can be cleaned safely first",
            format!(
                "Development residue accounts for about {:.1}% of scanned space ({}). Start with dry-run.",
                ratio * 100.0,
                bytes(residue_bytes)
            ),
            Some(format!(
                "aegis-disk clean {} --dry-run --target residue",
                report.root.display()
            )),
        ));
    }

    fn push_duplicate_advice(
        &self,
        report: &ScanReport,
        findings: &[Finding],
        insights: &mut Vec<DiskInsight>,
    ) {
        let duplicate_bytes: u64 = findings
            .iter()
            .filter(|item| {
                matches!(
                    item.kind,
                    FindingKind::DuplicateCandidate { keep: false, .. }
                )
            })
            .map(|item| item.size)
            .sum();
        if duplicate_bytes == 0 {
            return;
        }
        let ratio = ratio(duplicate_bytes, report.stats.total_size);
        let severity = if ratio >= self.duplicate_ratio_threshold {
            InsightSeverity::Notice
        } else {
            InsightSeverity::Info
        };
        insights.push(DiskInsight::new(
            severity,
            "Duplicate files may release extra space",
            format!(
                "Duplicate removal candidates account for about {:.1}% of scanned space ({}).",
                ratio * 100.0,
                bytes(duplicate_bytes)
            ),
            Some(format!("aegis-disk duplicates {}", report.root.display())),
        ));
    }

    fn push_next_step(
        &self,
        report: &ScanReport,
        findings: &[Finding],
        insights: &mut Vec<DiskInsight>,
    ) {
        if findings.is_empty() {
            insights.push(DiskInsight::new(
                InsightSeverity::Info,
                "No immediate cleanup candidates",
                "The current rules did not identify large files, residue directories, or duplicates.",
                Some(format!("aegis-disk types {}", report.root.display())),
            ));
        } else {
            insights.push(DiskInsight::new(
                InsightSeverity::Notice,
                "Recommended review order",
                "Review SAFE residue first, then duplicate candidates, then large files that require manual judgement.",
                Some(format!("aegis-disk tui {}", report.root.display())),
            ));
        }
    }
}

fn ratio(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        analysis::{Finding, FindingKind, InsightAnalyzer, RiskLevel},
        fs::{ScanReport, ScanStats},
    };

    #[test]
    fn produces_actionable_insights() {
        let report = ScanReport {
            root: PathBuf::from("demo"),
            entries: Vec::new(),
            stats: ScanStats {
                files: 1,
                dirs: 0,
                total_size: 1000,
                errors: 0,
            },
            errors: Vec::new(),
        };
        let findings = vec![Finding {
            path: PathBuf::from("demo/target"),
            kind: FindingKind::DevResidue,
            size: 500,
            risk: RiskLevel::Safe,
            reason: "test".to_owned(),
        }];
        let insights = InsightAnalyzer::new().analyze(&report, &findings);
        assert!(insights.iter().any(|item| item.suggested_command.is_some()));
    }
}
