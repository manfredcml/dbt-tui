//! Documentation component
//!
//! Displays documentation for the selected node including description and columns.

use crate::action::Action;
use crate::component::Component;
use crate::model::Node;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Documentation component for displaying node documentation
pub struct DocumentationComponent {
    /// Current scroll offset
    scroll: usize,
    /// Cached content lines
    content: Vec<Line<'static>>,
}

impl Default for DocumentationComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentationComponent {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            content: Vec::new(),
        }
    }

    /// Update content based on the selected node
    pub fn set_node(&mut self, node: Option<&Node>) {
        self.content = match node {
            Some(n) => self.render_node_documentation(n),
            None => vec![Line::from(Span::styled(
                "No node selected",
                Style::default().fg(Color::DarkGray),
            ))],
        };
    }

    fn render_node_documentation(&self, node: &Node) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Description section
        if let Some(ref desc) = node.description {
            if !desc.trim().is_empty() {
                lines.push(Line::from(Span::styled(
                    "Description:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));

                for line in desc.lines() {
                    lines.push(Line::from(line.to_string()));
                }
                lines.push(Line::from(""));
            }
        }

        // Columns section
        if !node.columns.is_empty() {
            lines.push(Line::from(Span::styled(
                "Columns:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Sort columns by name
            let mut column_vec: Vec<_> = node.columns.values().collect();
            column_vec.sort_by(|a, b| a.name.cmp(&b.name));

            for col in column_vec {
                let type_info = col
                    .data_type
                    .as_ref()
                    .map(|t| format!(" ({})", t))
                    .unwrap_or_default();

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", col.name),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(type_info, Style::default().fg(Color::DarkGray)),
                ]));

                if let Some(ref desc) = col.description {
                    if !desc.trim().is_empty() {
                        for line in desc.lines() {
                            lines.push(Line::from(format!("    {}", line)));
                        }
                    }
                }
                lines.push(Line::from(""));
            }
        }

        // If no documentation available
        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "No documentation available",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from("Add documentation to your models in schema.yml files."));
        }

        lines
    }
}

impl Component for DocumentationComponent {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::ScrollDown)
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::ScrollUp)
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::PageDown)
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::PageUp)
            }
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::PageUp => Some(Action::PageUp),
            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let max_scroll = self.content.len().saturating_sub(1);

        match action {
            Action::ScrollDown => {
                if self.scroll < max_scroll {
                    self.scroll += 1;
                }
            }
            Action::ScrollUp => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            Action::PageDown => {
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
        let visible_height = area.height.saturating_sub(2) as usize;

        let paragraph = Paragraph::new(self.content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Documentation ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .scroll((self.scroll as u16, 0));

        frame.render_widget(paragraph, area);

        // Render scrollbar if content exceeds visible area
        let total = self.content.len();
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

