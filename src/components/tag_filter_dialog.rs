//! Tag filter dialog component
//!
//! Allows selecting a tag to filter nodes by.

use crate::action::Action;
use crate::component::Component;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Tag filter dialog
pub struct TagFilterDialog {
    /// Available tags
    pub tags: Vec<String>,
    /// Selected tag index
    pub selected_index: usize,
    /// List state for rendering
    pub list_state: ListState,
    /// Current tag filter (to show which is active)
    pub current_filter: String,
}

impl Default for TagFilterDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl TagFilterDialog {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            tags: Vec::new(),
            selected_index: 0,
            list_state,
            current_filter: String::new(),
        }
    }

    /// Set available tags
    pub fn set_tags(&mut self, tags: Vec<String>, current_filter: &str) {
        self.tags = tags;
        self.current_filter = current_filter.to_string();

        // Try to select the current filter if it exists
        if let Some(idx) = self.tags.iter().position(|t| t == current_filter) {
            self.selected_index = idx + 1; // +1 because of "Clear filter" option
        } else {
            self.selected_index = 0;
        }
        self.list_state.select(Some(self.selected_index));
    }

    /// Get the selected tag (None means clear filter)
    pub fn get_selected_tag(&self) -> Option<&str> {
        if self.selected_index == 0 {
            None // "Clear filter" option
        } else {
            self.tags.get(self.selected_index - 1).map(|s| s.as_str())
        }
    }

    fn select_next(&mut self) {
        let max_index = self.tags.len(); // +1 for "Clear filter" but 0-indexed so just len
        if self.selected_index < max_index {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }
}

impl Component for TagFilterDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc | KeyCode::Char('f') => Some(Action::CloseModal),
            KeyCode::Enter => {
                if let Some(tag) = self.get_selected_tag() {
                    Some(Action::SetTagFilter(tag.to_string()))
                } else {
                    Some(Action::ClearTagFilter)
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                Some(Action::ModalUp)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Some(Action::ModalDown)
            }
            _ => None,
        };
        Ok(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Clear entire background
        frame.render_widget(Clear, area);

        // Calculate popup dimensions - minimum height for empty state
        let popup_width = 50u16.min(area.width.saturating_sub(4));
        let content_height = if self.tags.is_empty() { 6 } else { self.tags.len() as u16 + 3 };
        let popup_height = (content_height + 6).min(area.height.saturating_sub(4)).max(12);

        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(3),    // Tag list / empty message
                Constraint::Length(3), // Help bar
            ])
            .split(popup_area);

        // Header
        let header_text = if self.current_filter.is_empty() {
            "No filter active".to_string()
        } else {
            format!("Current: tag:{}", self.current_filter)
        };

        let header = Paragraph::new(Line::from(vec![Span::styled(
            header_text,
            Style::default().fg(Color::Cyan),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Filter by Tag ")
                .title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        frame.render_widget(header, main_chunks[0]);

        if self.tags.is_empty() {
            // Show empty state message
            let empty_message = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No tags found in your dbt project",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Add tags in dbt_project.yml or model configs:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "  +tags: [nightly, hourly]",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(empty_message, main_chunks[1]);
        } else {
            // Tag list
            let mut items: Vec<ListItem> = vec![ListItem::new(Line::from(vec![
                Span::styled(
                    if self.current_filter.is_empty() {
                        "● "
                    } else {
                        "  "
                    },
                    Style::default().fg(Color::Green),
                ),
                Span::styled("Clear filter", Style::default().fg(Color::DarkGray)),
            ]))];

            for tag in &self.tags {
                let is_current = *tag == self.current_filter;
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(
                        if is_current { "● " } else { "  " },
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        format!("tag:{}", tag),
                        if is_current {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                ])));
            }

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list, main_chunks[1], &mut self.list_state);
        }

        // Help bar - always visible
        let help_text = if self.tags.is_empty() {
            vec![
                Span::styled(" Esc/f ", Style::default().fg(Color::Yellow)),
                Span::raw("Close"),
            ]
        } else {
            vec![
                Span::styled(" Enter ", Style::default().fg(Color::Yellow)),
                Span::raw("Select  "),
                Span::styled(" j/k ", Style::default().fg(Color::Cyan)),
                Span::raw("Navigate  "),
                Span::styled(" Esc/f ", Style::default().fg(Color::Yellow)),
                Span::raw("Cancel"),
            ]
        };

        let help = Paragraph::new(Line::from(help_text))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, main_chunks[2]);

        Ok(())
    }
}
