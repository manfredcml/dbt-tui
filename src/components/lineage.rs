//! Lineage visualization component
//!
//! Displays upstream and downstream dependencies for the selected node.

use crate::action::Action;
use crate::component::Component;
use crate::model::lineage::LineageGraph;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Lineage component for displaying node dependencies
pub struct LineageComponent {
    /// Current scroll offset
    scroll: usize,
    /// Cached content lines
    content: Vec<Line<'static>>,
}

impl Default for LineageComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl LineageComponent {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            content: Vec::new(),
        }
    }

    /// Update content based on the selected node and lineage graph
    pub fn set_node(&mut self, node_unique_id: Option<&str>, lineage_graph: Option<&LineageGraph>) {
        self.content = match (node_unique_id, lineage_graph) {
            (Some(unique_id), Some(graph)) => self.render_lineage(unique_id, graph),
            _ => vec![Line::from(Span::styled(
                "No lineage data available",
                Style::default().fg(Color::DarkGray),
            ))],
        };
    }

    fn render_lineage(&self, node_unique_id: &str, graph: &LineageGraph) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        let upstream = graph.get_upstream(node_unique_id);
        let downstream = graph.get_downstream(node_unique_id);

        if upstream.is_empty() && downstream.is_empty() {
            lines.push(Line::from(Span::styled(
                "No dependencies found",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        // Upstream dependencies
        if !upstream.is_empty() {
            lines.push(Line::from(Span::styled(
                "Upstream (depends on):",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            for (i, dep) in upstream.iter().enumerate() {
                let is_last = i == upstream.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };

                let color = match dep.resource_type.as_str() {
                    "model" => Color::Blue,
                    "source" => Color::Green,
                    "seed" => Color::Yellow,
                    "test" => Color::Magenta,
                    _ => Color::White,
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{} {} ", prefix, dep.icon()), Style::default().fg(Color::DarkGray)),
                    Span::styled(dep.name.clone(), Style::default().fg(color)),
                    Span::styled(
                        format!(" ({})", dep.resource_type),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            if !downstream.is_empty() {
                lines.push(Line::from(""));
            }
        }

        // Downstream dependencies
        if !downstream.is_empty() {
            lines.push(Line::from(Span::styled(
                "Downstream (used by):",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            for (i, dep) in downstream.iter().enumerate() {
                let is_last = i == downstream.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };

                let color = match dep.resource_type.as_str() {
                    "model" => Color::Blue,
                    "source" => Color::Green,
                    "seed" => Color::Yellow,
                    "test" => Color::Magenta,
                    _ => Color::White,
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{} {} ", prefix, dep.icon()), Style::default().fg(Color::DarkGray)),
                    Span::styled(dep.name.clone(), Style::default().fg(color)),
                    Span::styled(
                        format!(" ({})", dep.resource_type),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }

        lines
    }
}

impl Component for LineageComponent {
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
                    .title(" Lineage ")
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

