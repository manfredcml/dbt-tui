//! Sample data dialog component
//!
//! Displays sample data from dbt show command output.

use crate::action::Action;
use crate::component::Component;
use crate::components::TableComponent;
use crate::model::{RunStatus, SampleDataOutput};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Sample data dialog
#[derive(Default)]
pub struct SampleDataDialog {
    pub scroll_offset: usize,
}

impl Component for SampleDataDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollUp),
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Esc | KeyCode::Char('q') => Some(Action::CloseModal),
            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            Action::ScrollDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            Action::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            Action::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(20);
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, _frame: &mut Frame, _area: Rect) -> Result<()> {
        // This needs sample data output, so we use draw_with_output
        Ok(())
    }
}

impl SampleDataDialog {
    pub fn draw_with_output(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        output: &SampleDataOutput,
    ) -> Result<()> {
        // Clear the entire area first
        frame.render_widget(Clear, area);

        let margin = 2;
        let overlay_area = Rect::new(
            margin,
            margin,
            area.width.saturating_sub(margin * 2),
            area.height.saturating_sub(margin * 2),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(overlay_area);

        let content_area = chunks[0];
        let help_area = chunks[1];

        // Get status indicator
        let (status_text, status_color) = match output.status {
            RunStatus::Running => ("Loading...", Color::Yellow),
            RunStatus::Success => ("Ready", Color::Green),
            RunStatus::Failed => ("Error", Color::Red),
        };

        // Generate content lines based on status
        let content_lines: Vec<Line<'static>> = match output.status {
            RunStatus::Running => {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Loading sample data...",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  Running: dbt show --select {} --limit 100", output.model_name),
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            }
            RunStatus::Failed => {
                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Failed to fetch sample data",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                ];

                if let Some(ref err) = output.error_message {
                    lines.push(Line::from(Span::styled(
                        format!("  Error: {}", err),
                        Style::default().fg(Color::Red),
                    )));
                }

                // Show raw output for debugging
                if !output.raw_output.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  Output:",
                        Style::default().fg(Color::DarkGray),
                    )));
                    for line in output.raw_output.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                lines
            }
            RunStatus::Success => {
                if output.headers.is_empty() {
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "  No data returned",
                            Style::default().fg(Color::Yellow),
                        )),
                    ]
                } else {
                    TableComponent::build_table_lines(&output.headers, &output.rows)
                }
            }
        };

        let total = content_lines.len();
        let visible_height = content_area.height.saturating_sub(2) as usize;

        // Clamp scroll offset
        let max_scroll = total.saturating_sub(visible_height);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }

        // Create block with title showing model name and status
        let title = format!(
            " Sample Data: {} [{}] ",
            output.model_name, status_text
        );

        let paragraph = Paragraph::new(content_lines.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(status_color)),
            )
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, content_area);

        // Render scrollbar if content exceeds visible area
        if total > visible_height {
            let mut scrollbar_state =
                ScrollbarState::new(total.saturating_sub(visible_height)).position(self.scroll_offset);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                content_area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        // Render help bar
        let help_text = Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Cyan)),
            Span::raw(" Scroll  "),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan)),
            Span::raw(" Page  "),
            Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" Close"),
        ]);

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(help, help_area);

        Ok(())
    }
}
