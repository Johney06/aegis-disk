//! TUI 状态管理。
//!
//! 这里集中保存交互状态，包括当前焦点、过滤条件、排序方式和列表选中项。
//! 渲染层只读取这些状态，不直接修改数据，从而让事件处理和界面绘制保持分离。

use std::cmp::Reverse;

use crate::{
    analysis::{Finding, FindingKind, RiskLevel},
    fs::ScanReport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Overview,
    Findings,
    Details,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskFilter {
    All,
    Safe,
    Review,
    Dangerous,
}

impl RiskFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Safe => "SAFE",
            Self::Review => "REVIEW",
            Self::Dangerous => "DANGER",
        }
    }

    fn matches(self, risk: RiskLevel) -> bool {
        match self {
            Self::All => true,
            Self::Safe => risk == RiskLevel::Safe,
            Self::Review => risk == RiskLevel::Review,
            Self::Dangerous => risk == RiskLevel::Dangerous,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::All => Self::Safe,
            Self::Safe => Self::Review,
            Self::Review => Self::Dangerous,
            Self::Dangerous => Self::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    SizeDesc,
    RiskDesc,
    KindAsc,
    PathAsc,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::SizeDesc => "SIZE DESC",
            Self::RiskDesc => "RISK DESC",
            Self::KindAsc => "KIND ASC",
            Self::PathAsc => "PATH ASC",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::SizeDesc => Self::RiskDesc,
            Self::RiskDesc => Self::KindAsc,
            Self::KindAsc => Self::PathAsc,
            Self::PathAsc => Self::SizeDesc,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RiskSummary {
    pub safe: usize,
    pub review: usize,
    pub dangerous: usize,
}

impl RiskSummary {
    pub fn total(self) -> usize {
        self.safe + self.review + self.dangerous
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizeSummary {
    pub safe_bytes: u64,
    pub review_bytes: u64,
    pub dangerous_bytes: u64,
}

impl SizeSummary {
    pub fn total(self) -> u64 {
        self.safe_bytes + self.review_bytes + self.dangerous_bytes
    }
}

#[derive(Debug, Clone)]
pub struct TuiState {
    pub report: ScanReport,
    pub findings: Vec<Finding>,
    pub selected: usize,
    pub focus: FocusPanel,
    pub filter: RiskFilter,
    pub sort: SortMode,
    visible_indices: Vec<usize>,
}

impl TuiState {
    pub fn new(report: ScanReport, findings: Vec<Finding>) -> Self {
        let mut state = Self {
            report,
            findings,
            selected: 0,
            focus: FocusPanel::Overview,
            filter: RiskFilter::All,
            sort: SortMode::SizeDesc,
            visible_indices: Vec::new(),
        };
        state.refresh_visible_indices();
        state
    }

    pub fn visible_len(&self) -> usize {
        self.visible_indices.len()
    }

    pub fn visible_findings(&self) -> Vec<&Finding> {
        self.visible_indices
            .iter()
            .filter_map(|idx| self.findings.get(*idx))
            .collect()
    }

    pub fn selected_finding(&self) -> Option<&Finding> {
        let finding_idx = self.visible_indices.get(self.selected)?;
        self.findings.get(*finding_idx)
    }

    pub fn selected_position_label(&self) -> String {
        if self.visible_indices.is_empty() {
            "none".to_owned()
        } else {
            format!("{}/{}", self.selected + 1, self.visible_indices.len())
        }
    }

    pub fn next(&mut self) {
        if !self.visible_indices.is_empty() {
            self.selected = (self.selected + 1).min(self.visible_indices.len() - 1);
        }
    }

    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn first(&mut self) {
        self.selected = 0;
    }

    pub fn last(&mut self) {
        if !self.visible_indices.is_empty() {
            self.selected = self.visible_indices.len() - 1;
        }
    }

    pub fn next_panel(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Overview => FocusPanel::Findings,
            FocusPanel::Findings => FocusPanel::Details,
            FocusPanel::Details => FocusPanel::Overview,
        };
    }

    pub fn previous_panel(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Overview => FocusPanel::Details,
            FocusPanel::Findings => FocusPanel::Overview,
            FocusPanel::Details => FocusPanel::Findings,
        };
    }

    pub fn next_filter(&mut self) {
        self.filter = self.filter.next();
        self.selected = 0;
        self.refresh_visible_indices();
    }

    pub fn next_sort(&mut self) {
        self.sort = self.sort.next();
        self.selected = 0;
        self.refresh_visible_indices();
    }

    pub fn reset_view(&mut self) {
        self.filter = RiskFilter::All;
        self.sort = SortMode::SizeDesc;
        self.selected = 0;
        self.refresh_visible_indices();
    }

    pub fn risk_summary(&self) -> RiskSummary {
        let mut summary = RiskSummary::default();
        for finding in &self.findings {
            match finding.risk {
                RiskLevel::Safe => summary.safe += 1,
                RiskLevel::Review => summary.review += 1,
                RiskLevel::Dangerous => summary.dangerous += 1,
            }
        }
        summary
    }

    pub fn size_summary(&self) -> SizeSummary {
        let mut summary = SizeSummary::default();
        for finding in &self.findings {
            match finding.risk {
                RiskLevel::Safe => {
                    summary.safe_bytes = summary.safe_bytes.saturating_add(finding.size)
                }
                RiskLevel::Review => {
                    summary.review_bytes = summary.review_bytes.saturating_add(finding.size);
                }
                RiskLevel::Dangerous => {
                    summary.dangerous_bytes = summary.dangerous_bytes.saturating_add(finding.size);
                }
            }
        }
        summary
    }

    pub fn reclaimable_estimate(&self) -> u64 {
        self.findings
            .iter()
            .filter(|finding| finding.risk == RiskLevel::Safe)
            .map(|finding| finding.size)
            .sum()
    }

    pub fn largest_finding(&self) -> Option<&Finding> {
        self.findings.iter().max_by_key(|finding| finding.size)
    }

    fn refresh_visible_indices(&mut self) {
        let mut indices: Vec<usize> = self
            .findings
            .iter()
            .enumerate()
            .filter_map(|(idx, finding)| self.filter.matches(finding.risk).then_some(idx))
            .collect();

        match self.sort {
            SortMode::SizeDesc => {
                indices.sort_by_key(|idx| Reverse(self.findings[*idx].size));
            }
            SortMode::RiskDesc => {
                indices.sort_by_key(|idx| Reverse(self.findings[*idx].risk));
            }
            SortMode::KindAsc => {
                indices.sort_by_key(|idx| kind_rank(&self.findings[*idx].kind));
            }
            SortMode::PathAsc => {
                indices.sort_by_key(|idx| self.findings[*idx].path.display().to_string());
            }
        }

        self.visible_indices = indices;
        if self.selected >= self.visible_indices.len() {
            self.selected = self.visible_indices.len().saturating_sub(1);
        }
    }
}

fn kind_rank(kind: &FindingKind) -> usize {
    match kind {
        FindingKind::LargeFile => 0,
        FindingKind::DevResidue => 1,
        FindingKind::DuplicateCandidate { keep, .. } => {
            if *keep {
                2
            } else {
                3
            }
        }
    }
}
