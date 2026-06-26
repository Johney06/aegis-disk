//! TUI 视图渲染模块。
//!
//! 仪表盘布局：标题栏、指标卡、空间占比、发现项列表、详情面板和底部帮助栏。

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Padding, Paragraph, Wrap},
};

use crate::{
    analysis::{Finding, FindingKind, RiskLevel},
    tui::state::{FocusPanel, TuiState},
    utils::format::bytes,
};

pub fn draw(frame: &mut Frame, state: &TuiState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(frame.area());
    draw_title(frame, state, rows[0]);
    draw_cards(frame, state, rows[1]);
    draw_workspace(frame, state, rows[2]);
    draw_help(frame, state, rows[3]);
}

fn draw_title(frame: &mut Frame, state: &TuiState, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " AegisDisk ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw("  Terminal Disk Intelligence Dashboard  |  "),
        Span::styled(
            state.report.root.display().to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn draw_cards(frame: &mut Frame, state: &TuiState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25); 4])
        .split(area);
    let stats = &state.report.stats;
    draw_card(
        frame,
        cols[0],
        "Scan",
        vec![
            ("Files", stats.files.to_string(), Color::Cyan),
            ("Dirs", stats.dirs.to_string(), Color::Blue),
            (
                "Errors",
                stats.errors.to_string(),
                if stats.errors == 0 {
                    Color::Green
                } else {
                    Color::Red
                },
            ),
        ],
    );
    draw_card(
        frame,
        cols[1],
        "Space",
        vec![
            ("Total", bytes(stats.total_size), Color::Green),
            ("Safe", bytes(state.reclaimable_estimate()), Color::Yellow),
            (
                "Largest",
                state
                    .largest_finding()
                    .map(|f| bytes(f.size))
                    .unwrap_or_else(|| "0 B".into()),
                Color::Magenta,
            ),
        ],
    );
    let risk = state.risk_summary();
    draw_card(
        frame,
        cols[2],
        "Risk",
        vec![
            ("SAFE", risk.safe.to_string(), Color::Green),
            ("REVIEW", risk.review.to_string(), Color::Yellow),
            ("DANGER", risk.dangerous.to_string(), Color::Red),
        ],
    );
    draw_card(
        frame,
        cols[3],
        "View",
        vec![
            ("Filter", state.filter.label().to_owned(), Color::Yellow),
            ("Sort", state.sort.label().to_owned(), Color::Cyan),
            (
                "Shown",
                format!("{}/{}", state.visible_len(), risk.total()),
                Color::White,
            ),
        ],
    );
}

fn draw_workspace(frame: &mut Frame, state: &TuiState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(42),
            Constraint::Percentage(28),
        ])
        .split(area);
    draw_overview(frame, state, cols[0]);
    draw_findings(frame, state, cols[1]);
    draw_details(frame, state, cols[2]);
}

fn draw_card(frame: &mut Frame, area: Rect, title: &'static str, rows: Vec<(&str, String, Color)>) {
    let lines = rows
        .into_iter()
        .map(|(k, v, c)| {
            Line::from(vec![
                Span::styled(format!("{k:<9}"), Style::default().fg(Color::Gray)),
                Span::styled(v, Style::default().fg(c).add_modifier(Modifier::BOLD)),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines).block(card_block(title)), area);
}

fn draw_overview(frame: &mut Frame, state: &TuiState, area: Rect) {
    let block = focused_block("Overview", state.focus == FocusPanel::Overview);
    frame.render_widget(block, area);
    let inner = padded(area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .split(inner);
    let sizes = state.size_summary();
    let total = sizes.total().max(1);
    let info = vec![
        Line::from(Span::styled(
            "Space by risk",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Safe:   {}", bytes(sizes.safe_bytes))),
        Line::from(format!("Review: {}", bytes(sizes.review_bytes))),
        Line::from(format!("Danger: {}", bytes(sizes.dangerous_bytes))),
    ];
    frame.render_widget(Paragraph::new(info), rows[0]);
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title("Safe reclaim ratio"))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio((sizes.safe_bytes as f64 / total as f64).clamp(0.0, 1.0)),
        rows[1],
    );
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title("Manual review ratio"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio((sizes.review_bytes as f64 / total as f64).clamp(0.0, 1.0)),
        rows[2],
    );
    let tips = vec![
        Line::from("Workflow:"),
        Line::from("1. Filter SAFE items"),
        Line::from("2. Run clean --dry-run"),
        Line::from("3. Review large files manually"),
    ];
    frame.render_widget(Paragraph::new(tips).wrap(Wrap { trim: true }), rows[3]);
}

fn draw_findings(frame: &mut Frame, state: &TuiState, area: Rect) {
    let visible = state.visible_findings();
    let items = if visible.is_empty() {
        vec![ListItem::new(
            "No findings under current filter. Press r to reset.",
        )]
    } else {
        visible
            .iter()
            .enumerate()
            .map(|(i, f)| finding_item(i, state.selected, f))
            .collect()
    };
    let title = format!(
        "Findings | {} | {}",
        state.filter.label(),
        state.sort.label()
    );
    frame.render_widget(
        List::new(items).block(dynamic_block(title, state.focus == FocusPanel::Findings)),
        area,
    );
}

fn draw_details(frame: &mut Frame, state: &TuiState, area: Rect) {
    let lines = if let Some(f) = state.selected_finding() {
        vec![
            kv("Selected", state.selected_position_label(), Color::Yellow),
            kv("Risk", f.risk.label().to_owned(), risk_color(f.risk)),
            kv("Kind", kind_label(&f.kind), Color::Cyan),
            kv("Size", bytes(f.size), Color::Green),
            Line::from(""),
            Line::from(Span::styled("Path", Style::default().fg(Color::Yellow))),
            Line::from(f.path.display().to_string()),
            Line::from(""),
            Line::from(Span::styled("Reason", Style::default().fg(Color::Yellow))),
            Line::from(f.reason.clone()),
            Line::from(""),
            Line::from(Span::styled(
                recommendation(f),
                Style::default().fg(Color::LightCyan),
            )),
        ]
    } else {
        vec![
            Line::from("No finding selected"),
            Line::from("Press f/s/r to adjust view."),
        ]
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(focused_block("Details", state.focus == FocusPanel::Details))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_help(frame: &mut Frame, state: &TuiState, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " Focus ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ),
        Span::raw(format!(" {}  ", focus_label(state.focus))),
        Span::styled(" Move ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw("↑/↓ j/k Home/End  "),
        Span::styled(
            " View ",
            Style::default().fg(Color::Black).bg(Color::Magenta),
        ),
        Span::raw("←/→ Tab f filter s sort r reset  "),
        Span::styled(" Quit ", Style::default().fg(Color::Black).bg(Color::Red)),
        Span::raw("q Esc Ctrl+C"),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn finding_item(idx: usize, selected: usize, f: &Finding) -> ListItem<'static> {
    let mark = if idx == selected { "▶" } else { " " };
    let action = match &f.kind {
        FindingKind::DuplicateCandidate { keep: true, .. } => "KEEP",
        FindingKind::DuplicateCandidate { keep: false, .. } => "DROP",
        FindingKind::LargeFile => "CHECK",
        FindingKind::DevResidue => "CLEAN",
    };
    let line = Line::from(vec![
        Span::styled(format!("{mark} "), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:<7}", f.risk.label()),
            Style::default()
                .fg(risk_color(f.risk))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>10}", bytes(f.size)),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" "),
        Span::styled(format!("{action:<6}"), Style::default().fg(Color::Gray)),
        Span::raw(" "),
        Span::raw(compact(&f.path.display().to_string(), 44)),
    ]);
    if idx == selected {
        ListItem::new(line).style(Style::default().bg(Color::DarkGray))
    } else {
        ListItem::new(line)
    }
}

fn kv(key: &'static str, value: String, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key:<9}"), Style::default().fg(Color::Gray)),
        Span::styled(value, Style::default().fg(color)),
    ])
}

fn card_block(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1))
}

fn focused_block(title: &'static str, focused: bool) -> Block<'static> {
    dynamic_block(title.to_owned(), focused)
}

fn dynamic_block(title: String, focused: bool) -> Block<'static> {
    let style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = if focused {
        format!("* {title} *")
    } else {
        title
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
        .padding(Padding::horizontal(1))
}

fn padded(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn compact(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let suffix: String = text
        .chars()
        .rev()
        .take(max.saturating_sub(3))
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("...{suffix}")
}

fn recommendation(f: &Finding) -> &'static str {
    match (&f.kind, f.risk) {
        (FindingKind::DevResidue, RiskLevel::Safe) => "Tip: run clean --dry-run before execute.",
        (FindingKind::DuplicateCandidate { keep: true, .. }, _) => {
            "Tip: recommended to keep this duplicate source."
        }
        (FindingKind::DuplicateCandidate { keep: false, .. }, _) => {
            "Tip: duplicate candidate, verify before cleaning."
        }
        (FindingKind::LargeFile, _) => "Tip: large files require manual review.",
        (_, RiskLevel::Dangerous) => "Tip: dangerous item, avoid automatic cleaning.",
        _ => "Tip: review details before cleaning.",
    }
}

fn focus_label(focus: FocusPanel) -> &'static str {
    match focus {
        FocusPanel::Overview => "Overview",
        FocusPanel::Findings => "Findings",
        FocusPanel::Details => "Details",
    }
}

fn risk_color(risk: RiskLevel) -> Color {
    match risk {
        RiskLevel::Safe => Color::Green,
        RiskLevel::Review => Color::Yellow,
        RiskLevel::Dangerous => Color::Red,
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
