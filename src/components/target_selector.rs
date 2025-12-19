//! Target selector dialog component
//!
//! Two-panel layout:
//! - Left panel: List of target names
//! - Right panel: YAML content for selected target

use crate::action::Action;
use crate::component::Component;
use crate::services::TargetInfo;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Target selector dialog
pub struct TargetSelectorDialog {
    pub selected_index: usize,
    pub targets: Vec<TargetInfo>,
    pub current_target: String,
    pub list_state: ListState,
    /// Whether profiles.yml was found
    pub profiles_found: bool,
}

impl Default for TargetSelectorDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetSelectorDialog {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            selected_index: 0,
            targets: Vec::new(),
            current_target: "dev".to_string(),
            list_state,
            profiles_found: false,
        }
    }

    /// Set targets from parsed profiles
    pub fn set_targets_with_info(&mut self, current: &str, targets: Vec<TargetInfo>) {
        self.current_target = current.to_string();
        self.targets = targets;
        self.profiles_found = true;

        // Set selected index to current target
        self.selected_index = self
            .targets
            .iter()
            .position(|t| t.name == current)
            .unwrap_or(0);
        self.list_state.select(Some(self.selected_index));
    }

    /// Mark that no profiles were found
    pub fn set_no_profiles(&mut self, current: &str) {
        self.current_target = current.to_string();
        self.targets.clear();
        self.profiles_found = false;
        self.selected_index = 0;
        self.list_state.select(None);
    }

    /// Get the currently selected target name
    pub fn get_selected_target(&self) -> &str {
        self.targets
            .get(self.selected_index)
            .map(|t| t.name.as_str())
            .unwrap_or(&self.current_target)
    }

    fn select_next(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        if self.selected_index < self.targets.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn select_prev(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Get the YAML content for the selected target
    fn get_selected_yaml(&self) -> Option<&str> {
        self.targets
            .get(self.selected_index)
            .map(|t| t.yaml_content.as_str())
    }
}

impl Component for TargetSelectorDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc | KeyCode::Char('t') => Some(Action::CloseModal),
            KeyCode::Enter if self.profiles_found && !self.targets.is_empty() => {
                Some(Action::ConfirmModal)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                Some(Action::ModalUp)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Some(Action::ModalDown)
            }
            KeyCode::Char('1') if !self.targets.is_empty() => {
                self.selected_index = 0;
                self.list_state.select(Some(0));
                Some(Action::ConfirmModal)
            }
            KeyCode::Char('2') if self.targets.len() > 1 => {
                self.selected_index = 1;
                self.list_state.select(Some(1));
                Some(Action::ConfirmModal)
            }
            KeyCode::Char('3') if self.targets.len() > 2 => {
                self.selected_index = 2;
                self.list_state.select(Some(2));
                Some(Action::ConfirmModal)
            }
            KeyCode::Char('4') if self.targets.len() > 3 => {
                self.selected_index = 3;
                self.list_state.select(Some(3));
                Some(Action::ConfirmModal)
            }
            _ => None,
        };
        Ok(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Clear entire background
        frame.render_widget(Clear, area);

        // Calculate popup dimensions
        let popup_width = 80u16.min(area.width.saturating_sub(4));
        let popup_height = 20u16.min(area.height.saturating_sub(4));

        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        // Main layout: header, content, help
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // Content (two panels)
                Constraint::Length(3), // Help bar
            ])
            .split(popup_area);

        // Header
        let header_text = if self.profiles_found {
            format!("Current: {}", self.current_target)
        } else {
            "No profiles.yml found".to_string()
        };

        let header = Paragraph::new(Line::from(vec![Span::styled(
            header_text,
            if self.profiles_found {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Red)
            },
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Target ")
                .title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
        );
        frame.render_widget(header, main_chunks[0]);

        if !self.profiles_found {
            // Show message when no profiles found
            let message = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Could not find profiles.yml",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Looked in:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "  • <project>/profiles.yml",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "  • ~/.dbt/profiles.yml",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(message, main_chunks[1]);
        } else {
            // Two-panel layout: left (targets) | right (yaml)
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(20), // Left panel - target list
                    Constraint::Min(30),    // Right panel - YAML content
                ])
                .split(main_chunks[1]);

            // Left panel: Target list
            let items: Vec<ListItem> = self
                .targets
                .iter()
                .enumerate()
                .map(|(i, target)| {
                    let is_current = target.name == self.current_target;
                    let prefix = if is_current { "● " } else { "  " };
                    let shortcut = format!("[{}] ", i + 1);

                    ListItem::new(Line::from(vec![
                        Span::styled(shortcut, Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            prefix,
                            Style::default().fg(if is_current {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(&target.name),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Targets ")
                        .title_style(Style::default().fg(Color::Cyan))
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list, content_chunks[0], &mut self.list_state);

            // Right panel: YAML content
            let yaml_lines: Vec<Line> = if let Some(yaml) = self.get_selected_yaml() {
                yaml.lines()
                    .map(|line| {
                        // Simple YAML syntax highlighting
                        if line.trim().starts_with('#') {
                            Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)))
                        } else if let Some((key, value)) = line.split_once(':') {
                            let key_part = format!("{}:", key);
                            Line::from(vec![
                                Span::styled(key_part, Style::default().fg(Color::Cyan)),
                                Span::styled(value.to_string(), Style::default().fg(Color::Yellow)),
                            ])
                        } else {
                            Line::from(line.to_string())
                        }
                    })
                    .collect()
            } else {
                vec![Line::from(Span::styled(
                    "No target selected",
                    Style::default().fg(Color::DarkGray),
                ))]
            };

            let yaml_panel = Paragraph::new(yaml_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Configuration ")
                    .title_style(Style::default().fg(Color::Cyan))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(yaml_panel, content_chunks[1]);
        }

        // Help bar
        let help_text = if self.profiles_found {
            vec![
                Span::styled(" Enter ", Style::default().fg(Color::Yellow)),
                Span::raw("Select  "),
                Span::styled(" j/k ", Style::default().fg(Color::Cyan)),
                Span::raw("Navigate  "),
                Span::styled(" Esc ", Style::default().fg(Color::Yellow)),
                Span::raw("Cancel"),
            ]
        } else {
            vec![
                Span::styled(" Esc ", Style::default().fg(Color::Yellow)),
                Span::raw("Close"),
            ]
        };

        let help = Paragraph::new(Line::from(help_text))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, main_chunks[2]);

        Ok(())
    }
}
