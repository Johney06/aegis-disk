//! JSON 报告渲染器。
//!
//! JSON 格式适合后续脚本处理或接入其他可视化工具。这里使用专门的 DTO，避免直接把内部结构绑定到外部格式。

use serde::Serialize;

use crate::{
    analysis::{DiskInsight, FileTypeStat, Finding, FindingKind},
    export::ExportContext,
};

#[derive(Debug, Serialize)]
struct JsonReport {
    root: String,
    summary: JsonSummary,
    insights: Vec<JsonInsight>,
    findings: Vec<JsonFinding>,
    file_types: Vec<JsonFileType>,
    safety_note: String,
}

#[derive(Debug, Serialize)]
struct JsonSummary {
    files: usize,
    dirs: usize,
    total_size: u64,
    errors: usize,
    findings: usize,
}

#[derive(Debug, Serialize)]
struct JsonInsight {
    severity: String,
    title: String,
    message: String,
    suggested_command: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonFinding {
    risk: String,
    size: u64,
    kind: String,
    path: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct JsonFileType {
    extension: String,
    files: usize,
    total_size: u64,
    average_size: u64,
    largest_size: u64,
    largest_file: Option<String>,
}

pub fn render_json(context: &ExportContext) -> anyhow::Result<String> {
    let report = JsonReport {
        root: context.root().display().to_string(),
        summary: JsonSummary {
            files: context.report.stats.files,
            dirs: context.report.stats.dirs,
            total_size: context.report.stats.total_size,
            errors: context.report.stats.errors,
            findings: context.findings.len(),
        },
        insights: context.insights.iter().map(json_insight).collect(),
        findings: context.findings.iter().map(json_finding).collect(),
        file_types: context.file_types.iter().map(json_file_type).collect(),
        safety_note: "Run clean --dry-run before executing any cleanup action.".to_owned(),
    };
    Ok(serde_json::to_string_pretty(&report)?)
}

fn json_insight(insight: &DiskInsight) -> JsonInsight {
    JsonInsight {
        severity: insight.severity.label().to_owned(),
        title: insight.title.clone(),
        message: insight.message.clone(),
        suggested_command: insight.suggested_command.clone(),
    }
}

fn json_finding(finding: &Finding) -> JsonFinding {
    JsonFinding {
        risk: finding.risk.label().to_owned(),
        size: finding.size,
        kind: kind_label(&finding.kind),
        path: finding.path.display().to_string(),
        reason: finding.reason.clone(),
    }
}

fn json_file_type(stat: &FileTypeStat) -> JsonFileType {
    JsonFileType {
        extension: stat.extension.clone(),
        files: stat.files,
        total_size: stat.total_size,
        average_size: stat.average_size(),
        largest_size: stat.largest_size,
        largest_file: stat
            .largest_file
            .as_ref()
            .map(|path| path.display().to_string()),
    }
}

fn kind_label(kind: &FindingKind) -> String {
    match kind {
        FindingKind::LargeFile => "large_file".to_owned(),
        FindingKind::DevResidue => "development_residue".to_owned(),
        FindingKind::DuplicateCandidate { group_id, keep } => {
            if *keep {
                format!("duplicate_group_{group_id}_keep")
            } else {
                format!("duplicate_group_{group_id}_remove")
            }
        }
    }
}
