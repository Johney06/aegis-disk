//! TUI 模块入口。
//!
//! 这里负责终端模式切换和事件循环；具体界面布局在 `view` 中，状态管理在
//! `state` 中。拆分后主循环会比较清晰，也方便后续扩展更多按键操作。

pub mod events;
pub mod state;
pub mod view;

use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{analysis::Finding, fs::ScanReport};

use self::{state::TuiState, view::draw};

pub fn run(report: ScanReport, findings: Vec<Finding>) -> Result<()> {
    // raw mode 可以让程序直接接收按键事件，而不是等待用户按回车。
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // alternate screen 可以让 TUI 退出后恢复原来的终端内容。
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut state = TuiState::new(report, findings);

    let result = loop {
        terminal.draw(|frame| draw(frame, &state))?;
        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Down
                | KeyCode::Char('j')
                | KeyCode::Char('J')
                | KeyCode::Char('n')
                | KeyCode::Char('N') => state.next(),
                KeyCode::Up
                | KeyCode::Char('k')
                | KeyCode::Char('K')
                | KeyCode::Char('p')
                | KeyCode::Char('P') => state.previous(),
                KeyCode::Home => state.first(),
                KeyCode::End => state.last(),
                KeyCode::Char('f') | KeyCode::Char('F') => state.next_filter(),
                KeyCode::Char('s') | KeyCode::Char('S') => state.next_sort(),
                KeyCode::Char('r') | KeyCode::Char('R') => state.reset_view(),
                KeyCode::BackTab | KeyCode::Left => state.previous_panel(),
                KeyCode::Tab
                | KeyCode::Right
                | KeyCode::Enter
                | KeyCode::Char(' ')
                | KeyCode::Char('h')
                | KeyCode::Char('H')
                | KeyCode::Char('l')
                | KeyCode::Char('L')
                | KeyCode::Char('a')
                | KeyCode::Char('A')
                | KeyCode::Char('d')
                | KeyCode::Char('D') => state.next_panel(),
                _ => {}
            }
        }
    };

    // 无论用户如何退出，都尽量恢复终端状态，避免影响后续命令输入。
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
