//! 报告导出模块。
//!
//! 导出模块把扫描报告、发现项、文件类型分布和诊断建议组合成稳定的数据结构，
//! 再交给不同格式的渲染器输出。这样 Markdown 和 JSON 可以共享同一份上下文。

pub mod json;
pub mod markdown;

use std::path::PathBuf;

use crate::{
    analysis::{DiskInsight, FileTypeStat, Finding},
    fs::ScanReport,
};

#[derive(Debug, Clone)]
pub struct ExportContext {
    pub report: ScanReport,
    pub findings: Vec<Finding>,
    pub file_types: Vec<FileTypeStat>,
    pub insights: Vec<DiskInsight>,
}

impl ExportContext {
    pub fn new(
        report: ScanReport,
        findings: Vec<Finding>,
        file_types: Vec<FileTypeStat>,
        insights: Vec<DiskInsight>,
    ) -> Self {
        Self {
            report,
            findings,
            file_types,
            insights,
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.report.root
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Json,
}

impl ExportFormat {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Json => "json",
        }
    }
}

pub fn render(context: &ExportContext, format: ExportFormat) -> anyhow::Result<String> {
    match format {
        ExportFormat::Markdown => Ok(markdown::render_markdown(context)),
        ExportFormat::Json => json::render_json(context),
    }
}
