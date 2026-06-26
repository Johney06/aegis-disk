//! TUI 状态管理。
//!
//! 界面渲染函数只负责画图，当前选中项、焦点面板等交互状态都集中放在这里。

use crate::{analysis::Finding, fs::ScanReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Overview,
    Findings,
    Details,
}

#[derive(Debug, Clone)]
pub struct TuiState {
    pub report: ScanReport,
    pub findings: Vec<Finding>,
    pub selected: usize,
    pub focus: FocusPanel,
}

impl TuiState {
    pub fn new(report: ScanReport, findings: Vec<Finding>) -> Self {
        Self {
            report,
            findings,
            selected: 0,
            focus: FocusPanel::Overview,
        }
    }

    pub fn selected_finding(&self) -> Option<&Finding> {
        self.findings.get(self.selected)
    }

    /// 选择下一条发现项，使用 `min` 防止越界。
    pub fn next(&mut self) {
        if !self.findings.is_empty() {
            self.selected = (self.selected + 1).min(self.findings.len() - 1);
        }
    }

    /// 选择上一条发现项，使用 `saturating_sub` 防止 usize 下溢。
    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn next_panel(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Overview => FocusPanel::Findings,
            FocusPanel::Findings => FocusPanel::Details,
            FocusPanel::Details => FocusPanel::Overview,
        };
    }
}
