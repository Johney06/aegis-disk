//! TUI 视图渲染模块。
//!
//! 使用 ratatui 把界面拆成三栏：概览、发现项列表和详情。渲染函数只读取状态，
//! 不修改状态，这样可以保持“状态更新”和“界面绘制”职责分离。

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{
    analysis::FindingKind,
    tui::state::{FocusPanel, TuiState},
    utils::format::bytes,
};

pub fn draw(frame: &mut Frame, state: &TuiState) {
    // 顶部三栏展示主要内容，底部状态栏专门提示当前焦点和按键。
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(42),
            Constraint::Percentage(30),
        ])
        .split(rows[0]);

    draw_overview(frame, state, columns[0]);
    draw_findings(frame, state, columns[1]);
    draw_details(frame, state, columns[2]);
    draw_status(frame, state, rows[1]);
}

fn draw_overview(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
    let stats = &state.report.stats;
    let lines = vec![
        Line::from(vec![Span::styled(
            "AegisDisk",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!("Root: {}", state.report.root.display())),
        Line::from(format!("Files: {}", stats.files)),
        Line::from(format!("Dirs: {}", stats.dirs)),
        Line::from(format!("Total: {}", bytes(stats.total_size))),
        Line::from(format!("Errors: {}", stats.errors)),
        Line::from(""),
        Line::from("Keys:"),
        Line::from("  q/Esc/Ctrl+C quit"),
        Line::from("  ↑/↓ or n/p move selected finding"),
        Line::from("  ←/→ or Enter/Space switch panel"),
    ];
    let paragraph = Paragraph::new(lines)
        .block(focused_block(
            "Overview",
            state.focus == FocusPanel::Overview,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_findings(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = if state.findings.is_empty() {
        vec![ListItem::new("No findings. Try: cargo run -- tui demo")]
    } else {
        state
            .findings
            .iter()
            .enumerate()
            .map(|(idx, finding)| {
                let prefix = if idx == state.selected { ">" } else { " " };
                let label = kind_label(&finding.kind);
                let line = format!(
                    "{prefix} {:<8} {:>10} {}",
                    finding.risk.label(),
                    bytes(finding.size),
                    label
                );
                // 当前选中项用黄色加粗显示，方便终端中定位。
                ListItem::new(line).style(if idx == state.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
            })
            .collect()
    };
    let list = List::new(items).block(focused_block(
        "Findings - j/k changes selection",
        state.focus == FocusPanel::Findings,
    ));
    frame.render_widget(list, area);
}

fn draw_details(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
    let lines = if let Some(finding) = state.selected_finding() {
        vec![
            Line::from(format!(
                "Selected: {}/{}",
                state.selected + 1,
                state.findings.len()
            )),
            Line::from(format!("Risk: {}", finding.risk.label())),
            Line::from(format!("Kind: {}", kind_label(&finding.kind))),
            Line::from(format!("Size: {}", bytes(finding.size))),
            Line::from(""),
            Line::from("Path:"),
            Line::from(finding.path.display().to_string()),
            Line::from(""),
            Line::from("Reason:"),
            Line::from(finding.reason.clone()),
            Line::from(""),
            Line::from("Use CLI clean --dry-run first."),
        ]
    } else {
        vec![
            Line::from("No finding selected."),
            Line::from(""),
            Line::from("If the list is empty, create demo data or scan a directory with"),
            Line::from("large files, duplicate files, target/, node_modules/, etc."),
        ]
    };
    let paragraph = Paragraph::new(lines)
        .block(focused_block("Details", state.focus == FocusPanel::Details))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_status(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
    let selected = if state.findings.is_empty() {
        "selected: none".to_owned()
    } else {
        format!("selected: {}/{}", state.selected + 1, state.findings.len())
    };
    let line = Line::from(vec![
        Span::styled("Focus: ", Style::default().fg(Color::Gray)),
        Span::styled(focus_label(state.focus), Style::default().fg(Color::Yellow)),
        Span::raw("  |  "),
        Span::styled(selected, Style::default().fg(Color::Cyan)),
        Span::raw("  |  ↑/↓ or n/p: move  ←/→ or Enter/Space: panel  q/Esc/Ctrl+C: quit"),
    ]);
    let paragraph =
        Paragraph::new(vec![line]).block(Block::default().title("Status").borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn focused_block(title: &'static str, focused: bool) -> Block<'static> {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = if focused {
        format!("* {title} *")
    } else {
        title.to_owned()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
}

fn focus_label(focus: FocusPanel) -> &'static str {
    match focus {
        FocusPanel::Overview => "Overview",
        FocusPanel::Findings => "Findings",
        FocusPanel::Details => "Details",
    }
}

fn kind_label(kind: &FindingKind) -> String {
    match kind {
        FindingKind::LargeFile => "large-file".into(),
        FindingKind::DevResidue => "dev-residue".into(),
        FindingKind::DuplicateCandidate { group_id, keep } => {
            if *keep {
                format!("dup-{group_id}-keep")
            } else {
                format!("dup-{group_id}-remove")
            }
        }
    }
}
