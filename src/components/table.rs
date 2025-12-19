//! Table component for CSV data display
//!
//! Renders tabular data with headers, rows, and column alignment.

use crate::action::Action;
use crate::component::Component;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Table component for displaying CSV/tabular data
pub struct TableComponent {
    /// Column headers
    headers: Vec<String>,
    /// Data rows
    rows: Vec<Vec<String>>,
    /// Scroll offset
    scroll: usize,
}

impl Default for TableComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl TableComponent {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            scroll: 0,
        }
    }

    /// Set the table data (headers and rows)
    pub fn set_data(&mut self, headers: Vec<String>, rows: Vec<Vec<String>>) {
        self.headers = headers;
        self.rows = rows;
        self.scroll = 0;
    }

    /// Render table content as lines (for embedding in other panels)
    pub fn render_lines(&self) -> Vec<Line<'static>> {
        Self::build_table_lines(&self.headers, &self.rows)
    }

    /// Build table lines from headers and rows
    pub fn build_table_lines(headers: &[String], rows: &[Vec<String>]) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if headers.is_empty() {
            return vec![Line::from("Empty CSV file")];
        }

        // Calculate column widths
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Limit column widths to 50 characters max to prevent overly wide columns
        // while still showing reasonable content (e.g., email addresses, UUIDs)
        for width in &mut col_widths {
            *width = (*width).min(50);
        }

        // Render header
        let header_spans: Vec<Span> = headers
            .iter()
            .enumerate()
            .flat_map(|(i, h)| {
                let width = col_widths[i];
                let truncated = if h.len() > width {
                    format!("{}...", &h[..width.saturating_sub(3)])
                } else {
                    h.clone()
                };
                vec![
                    Span::styled(
                        format!("{:width$}", truncated, width = width),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" │ "),
                ]
            })
            .collect();
        lines.push(Line::from(header_spans));

        // Render separator
        let separator: String = col_widths
            .iter()
            .map(|w| "─".repeat(*w))
            .collect::<Vec<_>>()
            .join("─┼─");
        lines.push(Line::from(Span::styled(
            separator,
            Style::default().fg(Color::DarkGray),
        )));

        // Render rows
        for row in rows {
            let row_spans: Vec<Span> = row
                .iter()
                .enumerate()
                .flat_map(|(i, cell)| {
                    let width = col_widths.get(i).copied().unwrap_or(10);
                    let truncated = if cell.len() > width {
                        format!("{}...", &cell[..width.saturating_sub(3)])
                    } else {
                        cell.clone()
                    };
                    vec![
                        Span::styled(
                            format!("{:width$}", truncated, width = width),
                            Style::default().fg(Color::White),
                        ),
                        Span::raw(" │ "),
                    ]
                })
                .collect();
            lines.push(Line::from(row_spans));
        }

        // Add row count
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Total rows: {}", rows.len()),
            Style::default().fg(Color::Yellow),
        )));

        lines
    }
}

impl Component for TableComponent {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollUp),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::PageDown)
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::PageUp)
            }
            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ScrollDown => {
                let max_scroll = self.rows.len().saturating_sub(1);
                if self.scroll < max_scroll {
                    self.scroll += 1;
                }
            }
            Action::ScrollUp => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            Action::PageDown => {
                let max_scroll = self.rows.len().saturating_sub(1);
                self.scroll = (self.scroll + 10).min(max_scroll);
            }
            Action::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let content = self.render_lines();
        let visible_height = area.height.saturating_sub(2) as usize;

        let paragraph = Paragraph::new(content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Table ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .scroll((self.scroll as u16, 0));

        frame.render_widget(paragraph, area);

        // Render scrollbar if content exceeds visible area
        let total = content.len();
        if total > visible_height {
            let mut scrollbar_state =
                ScrollbarState::new(total.saturating_sub(visible_height)).position(self.scroll);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        Ok(())
    }
}
