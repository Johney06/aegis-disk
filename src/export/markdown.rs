//! Markdown 报告渲染器。
//!
//! Markdown 格式适合直接提交到作业附件、GitHub Issue 或实验报告补充材料中。

use crate::{
    analysis::{Finding, FindingKind},
    export::ExportContext,
    utils::format::bytes,
};

pub fn render_markdown(context: &ExportContext) -> String {
    let mut out = String::new();
    push_title(&mut out, context);
    push_summary(&mut out, context);
    push_insights(&mut out, context);
    push_findings(&mut out, context);
    push_file_types(&mut out, context);
    push_footer(&mut out);
    out
}

fn push_title(out: &mut String, context: &ExportContext) {
    out.push_str("# AegisDisk Scan Report\n\n");
    out.push_str(&format!("- Root: `{}`\n", context.root().display()));
    out.push_str("- Format: Markdown\n\n");
}

fn push_summary(out: &mut String, context: &ExportContext) {
    let stats = &context.report.stats;
    out.push_str("## 1. Scan Summary\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|---|---:|\n");
    out.push_str(&format!("| Files | {} |\n", stats.files));
    out.push_str(&format!("| Directories | {} |\n", stats.dirs));
    out.push_str(&format!("| Total Size | {} |\n", bytes(stats.total_size)));
    out.push_str(&format!("| Access Errors | {} |\n", stats.errors));
    out.push_str(&format!("| Findings | {} |\n\n", context.findings.len()));
}

fn push_insights(out: &mut String, context: &ExportContext) {
    out.push_str("## 2. Diagnostic Insights\n\n");
    if context.insights.is_empty() {
        out.push_str("No diagnostic insight was generated.\n\n");
        return;
    }
    for insight in &context.insights {
        out.push_str(&format!(
            "### [{}] {}\n\n{}\n\n",
            insight.severity.label(),
            insight.title,
            insight.message
        ));
        if let Some(command) = &insight.suggested_command {
            out.push_str(&format!(
                "Suggested command:\n\n```bash\n{command}\n```\n\n"
            ));
        }
    }
}

fn push_findings(out: &mut String, context: &ExportContext) {
    out.push_str("## 3. Findings\n\n");
    if context.findings.is_empty() {
        out.push_str("No findings were detected by the enabled analyzers.\n\n");
        return;
    }
    out.push_str("| Risk | Size | Kind | Path | Reason |\n");
    out.push_str("|---|---:|---|---|---|\n");
    for finding in context.findings.iter().take(100) {
        out.push_str(&finding_row(finding));
    }
    if context.findings.len() > 100 {
        out.push_str(&format!(
            "\nOnly the first 100 findings are listed. Total findings: {}.\n",
            context.findings.len()
        ));
    }
    out.push('\n');
}

fn push_file_types(out: &mut String, context: &ExportContext) {
    out.push_str("## 4. File Type Distribution\n\n");
    if context.file_types.is_empty() {
        out.push_str("No file type statistics were generated.\n\n");
        return;
    }
    out.push_str("| Extension | Files | Total Size | Average Size | Largest File |\n");
    out.push_str("|---|---:|---:|---:|---|\n");
    for stat in context.file_types.iter().take(50) {
        let largest = stat
            .largest_file
            .as_ref()
            .map(|path| escape_cell(&path.display().to_string()))
            .unwrap_or_else(|| "-".to_owned());
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            escape_cell(stat.display_extension()),
            stat.files,
            bytes(stat.total_size),
            bytes(stat.average_size()),
            largest
        ));
    }
    out.push('\n');
}

fn push_footer(out: &mut String) {
    out.push_str("## 5. Safety Note\n\n");
    out.push_str("This report is read-only. Before executing cleanup, run `clean --dry-run` first and review all paths manually.\n");
}

fn finding_row(finding: &Finding) -> String {
    format!(
        "| {} | {} | {} | `{}` | {} |\n",
        finding.risk.label(),
        bytes(finding.size),
        kind_label(&finding.kind),
        escape_cell(&finding.path.display().to_string()),
        escape_cell(&finding.reason)
    )
}

fn kind_label(kind: &FindingKind) -> String {
    match kind {
        FindingKind::LargeFile => "large-file".into(),
        FindingKind::DevResidue => "dev-residue".into(),
        FindingKind::DuplicateCandidate { group_id, keep } => {
            if *keep {
                format!("duplicate-{group_id}-keep")
            } else {
                format!("duplicate-{group_id}-remove")
            }
        }
    }
}

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
