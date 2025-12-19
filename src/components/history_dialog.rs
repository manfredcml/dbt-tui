//! History dialog component
//!
//! Displays run history with details for selected entry.

use crate::action::Action;
use crate::component::Component;
use crate::model::{RunHistoryEntry, RunStatus};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match ANSI escape codes
static ANSI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap()
});

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

/// Run history dialog
#[derive(Default)]
pub struct HistoryDialog {
    pub selected_index: usize,
    pub detail_scroll: usize,
}

impl Component for HistoryDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Up => Some(Action::ModalUp),
            KeyCode::Down => Some(Action::ModalDown),
            KeyCode::Char('j') => Some(Action::ScrollDown),
            KeyCode::Char('k') => Some(Action::ScrollUp),
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Esc | KeyCode::Char('h') => Some(Action::CloseModal),
            KeyCode::Enter => Some(Action::OpenRunOutput),
            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ModalUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.detail_scroll = 0;
                }
            }
            Action::ModalDown => {
                self.selected_index += 1;
                self.detail_scroll = 0;
            }
            Action::ScrollUp => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            Action::ScrollDown => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            Action::PageUp => {
                self.detail_scroll = self.detail_scroll.saturating_sub(10);
            }
            Action::PageDown => {
                self.detail_scroll = self.detail_scroll.saturating_add(10);
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, _frame: &mut Frame, _area: Rect) -> Result<()> {
        // This needs history data, so we use draw_with_history
        Ok(())
    }
}

impl HistoryDialog {
    pub fn draw_with_history(
        &self,
        frame: &mut Frame,
        area: Rect,
        history: &[RunHistoryEntry],
    ) -> Result<()> {
        // Clear the entire screen and fill with terminal default background
        frame.render_widget(Clear, area);
        let background = Block::default().style(Style::default().bg(Color::Reset));
        frame.render_widget(background, area);

        let margin = 2;
        let overlay_area = Rect::new(
            margin,
            margin,
            area.width.saturating_sub(margin * 2),
            area.height.saturating_sub(margin * 2),
        );

        if history.is_empty() {
            let paragraph =
                Paragraph::new("No run history yet. Run a model, test, or seed to see history here.")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Run History ")
                            .title_style(
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            ),
                    );
            frame.render_widget(paragraph, overlay_area);
            return Ok(());
        }

        // Clamp selected index
        let selected_idx = self.selected_index.min(history.len().saturating_sub(1));

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(overlay_area);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(main_chunks[0]);

        // Render list
        let items: Vec<ListItem> = history
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let (icon, color) = match entry.status {
                    RunStatus::Running => ("⏳", Color::Yellow),
                    RunStatus::Success => ("✓", Color::Green),
                    RunStatus::Failed => ("✗", Color::Red),
                };

                let short_cmd = entry
                    .command
                    .split("--select ")
                    .nth(1)
                    .unwrap_or(&entry.command)
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown");

                let style = if i == selected_idx {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(
                        format!("{} ", entry.formatted_time()),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(short_cmd.to_string(), style),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" History ")
                    .title_style(
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
            );

        frame.render_widget(list, content_chunks[0]);

        // Render detail
        if let Some(entry) = history.get(selected_idx) {
            let detail_lines = render_history_detail(entry);
            let total = detail_lines.len();
            let visible_height = content_chunks[1].height.saturating_sub(2) as usize;
            let scroll = self.detail_scroll.min(total.saturating_sub(visible_height));

            let detail = Paragraph::new(detail_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Details ")
                        .title_style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .scroll((scroll as u16, 0));

            frame.render_widget(detail, content_chunks[1]);

            if total > visible_height {
                let mut scrollbar_state =
                    ScrollbarState::new(total.saturating_sub(visible_height)).position(scroll);
                frame.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight),
                    content_chunks[1].inner(ratatui::layout::Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut scrollbar_state,
                );
            }
        }

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled(
                " Esc/h ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Close  "),
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Select  "),
            Span::styled(
                " j/k ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Scroll"),
        ]))
        .alignment(ratatui::layout::Alignment::Left)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(help, main_chunks[1]);

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

fn render_history_detail(entry: &RunHistoryEntry) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();

    // Header info
    lines.push(Line::from(vec![
        Span::styled(
            "Time: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "Duration: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(entry.formatted_duration()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "Status: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} {:?}", entry.status_icon(), entry.status),
            match entry.status {
                RunStatus::Success => Style::default().fg(Color::Green),
                RunStatus::Failed => Style::default().fg(Color::Red),
                RunStatus::Running => Style::default().fg(Color::Yellow),
            },
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Command: ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(Span::raw(entry.command.clone())));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─".repeat(60),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    // Output content with color coding - strip ANSI codes first
    for line in entry.output.lines() {
        let clean_line = strip_ansi_codes(line);
        let styled_line = if clean_line.contains("error") || clean_line.contains("Error") || clean_line.contains("FAILED")
        {
            Line::from(Span::styled(
                clean_line,
                Style::default().fg(Color::Red),
            ))
        } else if clean_line.contains("warning") || clean_line.contains("Warning") {
            Line::from(Span::styled(
                clean_line,
                Style::default().fg(Color::Yellow),
            ))
        } else if clean_line.contains("SUCCESS") || clean_line.contains("PASS") {
            Line::from(Span::styled(
                clean_line,
                Style::default().fg(Color::Green),
            ))
        } else {
            Line::from(clean_line)
        };
        lines.push(styled_line);
    }

    lines
}
