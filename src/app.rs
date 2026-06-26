//! 应用编排层。
//!
//! 这一层负责把 CLI 命令转换为具体业务流程：加载配置、扫描目录、运行分析器、
//! 生成清理计划、输出结果或启动 TUI。算法细节放在各自模块中，这样测试和维护更方便。

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};

use crate::{
    analysis::{
        Analyzer, DiskInsight, DuplicateAnalyzer, FileTypeAnalyzer, FileTypeStat, Finding,
        InsightAnalyzer, LargeFileAnalyzer, ResidueAnalyzer, rule::cleanable_findings,
    },
    cleaner::{CleanPlan, TrashCleaner},
    cli::{Cli, Command, ConfigCommand},
    config::AppConfig,
    error::SentinelError,
    export::{ExportContext, ExportFormat, render},
    fs::{SafetyGuard, Scanner},
    tui,
    utils::format::{bytes, parse_size},
};

type AnalysisContext = (
    crate::fs::ScanReport,
    Vec<Finding>,
    Vec<FileTypeStat>,
    Vec<DiskInsight>,
);

pub fn run(cli: Cli) -> Result<()> {
    // 配置先于命令执行加载，使扫描器、分析器和 TUI 都能共享同一份规则。
    let config = AppConfig::load(cli.config)?;
    match cli.command {
        Command::Scan { path, limit } => {
            validate_scan_root(&path)?;
            let report = Scanner::new(config).scan(&path);
            print_report(&report, limit);
        }
        Command::Large {
            path,
            min_size,
            limit,
        } => {
            let min_size = parse_size(&min_size)?;
            validate_scan_root(&path)?;
            let report = Scanner::new(config).scan(&path);
            print_report_summary(&report);
            let findings = LargeFileAnalyzer::new(min_size).analyze(&report.entries);
            print_findings(&findings, limit);
        }
        Command::Residue { path, limit } => {
            validate_scan_root(&path)?;
            let report = Scanner::new(config.clone()).scan(&path);
            print_report_summary(&report);
            let findings = ResidueAnalyzer::new(&config).analyze(&report.entries);
            print_findings(&findings, limit);
        }
        Command::Duplicates { path, limit } => {
            validate_scan_root(&path)?;
            let report = Scanner::new(config.clone()).scan(&path);
            print_report_summary(&report);
            let findings =
                DuplicateAnalyzer::new(config.duplicate_min_size).analyze(&report.entries);
            print_findings(&findings, limit);
        }
        Command::Types { path, limit } => {
            validate_scan_root(&path)?;
            let report = Scanner::new(config).scan(&path);
            print_report_summary(&report);
            let stats = FileTypeAnalyzer::new().top_n(&report.entries, limit);
            print_file_type_stats(&stats);
        }
        Command::Insights { path, limit } => {
            validate_scan_root(&path)?;
            let (report, findings, _, insights) = build_analysis_context(path, &config)?;
            print_report_summary(&report);
            print_insights(&insights, limit);
            if findings.is_empty() {
                println!("\nNo cleanup-oriented findings were detected.");
            }
        }
        Command::Export {
            path,
            format,
            output,
        } => {
            validate_scan_root(&path)?;
            let format = ExportFormat::parse(&format).ok_or_else(|| {
                anyhow::anyhow!("unsupported export format: {format}; use markdown or json")
            })?;
            let (report, findings, file_types, insights) = build_analysis_context(path, &config)?;
            let context = ExportContext::new(report, findings, file_types, insights);
            let rendered = render(&context, format)?;
            write_export(output, &rendered, format)?;
        }
        Command::Clean {
            path,
            dry_run,
            execute,
            yes,
            target,
        } => {
            // clean 命令是高风险操作，因此要求用户明确选择 dry-run 或 execute。
            if dry_run && execute {
                return Err(SentinelError::ConflictingCleanMode.into());
            }
            if !dry_run && !execute {
                return Err(SentinelError::MissingCleanMode.into());
            }
            let guard = SafetyGuard::new(&config);
            if guard.is_dangerous_root(&path) {
                return Err(SentinelError::ProtectedPath(path).into());
            }
            validate_scan_root(&path)?;
            let report = Scanner::new(config.clone()).scan(&path);
            let findings = match target.as_str() {
                "residue" => ResidueAnalyzer::new(&config).analyze(&report.entries),
                "duplicates" | "dups" => {
                    DuplicateAnalyzer::new(config.duplicate_min_size).analyze(&report.entries)
                }
                other => bail!("unsupported clean target: {other}; use residue or duplicates"),
            };
            let cleanable = cleanable_findings(&findings);
            let trash_dir = path.join(".aegis_disk_trash");
            let plan = CleanPlan::from_findings(path.clone(), &cleanable, trash_dir);
            println!(
                "Clean plan: {} item(s), estimated {}",
                plan.estimated_count(),
                bytes(plan.estimated_bytes())
            );
            if execute && !yes && !confirm_execute(&target, plan.estimated_count())? {
                println!("Cancelled by user.");
                return Ok(());
            }
            let cleaner = TrashCleaner::new(guard);
            let summary = if dry_run {
                cleaner.dry_run(&plan)
            } else {
                cleaner.execute(&plan)
            };
            print_clean_summary(&summary);
        }
        Command::Config { command } => match command {
            ConfigCommand::PrintDefault => {
                println!("{}", AppConfig::default().to_pretty_toml()?);
            }
        },
        Command::Tui { path } => {
            validate_scan_root(&path)?;
            let report = Scanner::new(config.clone()).scan(&path);
            let mut findings = Vec::new();
            let large_threshold = parse_size(&config.default_large_file_threshold)?;
            findings.extend(LargeFileAnalyzer::new(large_threshold).analyze(&report.entries));
            findings.extend(ResidueAnalyzer::new(&config).analyze(&report.entries));
            findings
                .extend(DuplicateAnalyzer::new(config.duplicate_min_size).analyze(&report.entries));
            tui::run(report, findings)?;
        }
    }
    Ok(())
}

fn validate_scan_root(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(SentinelError::ScanRootNotFound(path.to_path_buf()).into());
    }
    if !path.is_dir() {
        return Err(SentinelError::ScanRootNotDirectory(path.to_path_buf()).into());
    }
    Ok(())
}

fn build_analysis_context(path: PathBuf, config: &AppConfig) -> Result<AnalysisContext> {
    let report = Scanner::new(config.clone()).scan(&path);
    let large_threshold = parse_size(&config.default_large_file_threshold)?;
    let mut findings = Vec::new();
    findings.extend(LargeFileAnalyzer::new(large_threshold).analyze(&report.entries));
    findings.extend(ResidueAnalyzer::new(config).analyze(&report.entries));
    findings.extend(DuplicateAnalyzer::new(config.duplicate_min_size).analyze(&report.entries));
    findings.sort_by(|a, b| b.size.cmp(&a.size));
    let file_types = FileTypeAnalyzer::new().top_n(&report.entries, 50);
    let insights = InsightAnalyzer::new().analyze(&report, &findings);
    Ok((report, findings, file_types, insights))
}

fn write_export(output: Option<PathBuf>, rendered: &str, format: ExportFormat) -> Result<()> {
    if let Some(path) = output {
        fs::write(&path, rendered)?;
        println!("Exported {} report to {}", format.label(), path.display());
    } else {
        println!("{rendered}");
    }
    Ok(())
}

fn print_report(report: &crate::fs::ScanReport, limit: usize) {
    print_report_summary(report);
    println!("\nTop entries:");
    let mut entries = report.entries.clone();
    entries.sort_by(|a, b| b.size.cmp(&a.size));
    for entry in entries.into_iter().take(limit) {
        let kind = if entry.is_dir { "DIR" } else { "FILE" };
        println!(
            "{kind:<4} {:>12}  {}",
            bytes(entry.size),
            entry.path.display()
        );
    }
}

fn print_report_summary(report: &crate::fs::ScanReport) {
    println!("Root: {}", report.root.display());
    println!("Files: {}", report.stats.files);
    println!("Dirs: {}", report.stats.dirs);
    println!("Total size: {}", bytes(report.stats.total_size));
    println!("Errors: {}", report.stats.errors);
}

fn print_findings(findings: &[crate::analysis::Finding], limit: usize) {
    println!("\nFindings: {}", findings.len());
    for finding in findings.iter().take(limit) {
        println!(
            "{:<8} {:>12}  {:<20} {}",
            finding.risk.label(),
            bytes(finding.size),
            kind_label(&finding.kind),
            finding.path.display()
        );
        println!("  reason: {}", finding.reason);
    }
}

fn print_file_type_stats(stats: &[crate::analysis::FileTypeStat]) {
    println!("\nFile type distribution: {} type(s)", stats.len());
    println!(
        "{:<18} {:>8} {:>14} {:>14}  Largest file",
        "Extension", "Files", "Total", "Average"
    );
    for stat in stats {
        let largest = stat
            .largest_file
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_owned());
        println!(
            "{:<18} {:>8} {:>14} {:>14}  {}",
            stat.display_extension(),
            stat.files,
            bytes(stat.total_size),
            bytes(stat.average_size()),
            largest
        );
    }
}

fn print_insights(insights: &[crate::analysis::DiskInsight], limit: usize) {
    println!("\nInsights: {}", insights.len());
    for insight in insights.iter().take(limit) {
        println!("[{}] {}", insight.severity.label(), insight.title);
        println!("  {}", insight.message);
        if let Some(command) = &insight.suggested_command {
            println!("  suggested: {command}");
        }
    }
}

fn kind_label(kind: &crate::analysis::FindingKind) -> String {
    match kind {
        crate::analysis::FindingKind::LargeFile => "large-file".into(),
        crate::analysis::FindingKind::DevResidue => "dev-residue".into(),
        crate::analysis::FindingKind::DuplicateCandidate { group_id, keep } => {
            if *keep {
                format!("dup-{group_id}-keep")
            } else {
                format!("dup-{group_id}-remove")
            }
        }
    }
}

fn print_clean_summary(summary: &crate::cleaner::CleanSummary) {
    println!("Planned: {}", summary.planned);
    println!("Moved: {}", summary.moved);
    println!("Skipped: {}", summary.skipped);
    println!("Failed: {}", summary.failed);
    println!("Reclaimed estimate: {}", bytes(summary.reclaimed_bytes));
    for message in &summary.messages {
        println!("{message}");
    }
}

fn confirm_execute(target: &str, count: usize) -> Result<bool> {
    print!("About to execute clean target '{target}' for {count} item(s). Type YES to continue: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim() == "YES")
}
