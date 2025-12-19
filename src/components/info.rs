//! Project information dialog component
//!
//! Displays dbt project info including version, profile, and resource counts.

use crate::action::Action;
use crate::component::Component;
use crate::model::ProjectInfo;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Project information dialog component
pub struct ProjectInfoDialog {
    /// Cached content lines
    content: Vec<Line<'static>>,
}

impl Default for ProjectInfoDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectInfoDialog {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    /// Update content based on project info
    pub fn set_project_info(&mut self, info: Option<&ProjectInfo>) {
        self.content = match info {
            Some(info) => render_project_info(info),
            None => vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Project information not available",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from("Press 'i' or Esc to close"),
            ],
        };
    }
}

impl Component for ProjectInfoDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('i') | KeyCode::Esc => Some(Action::CloseModal),
            _ => None,
        };
        Ok(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(overlay_area);

        let paragraph = Paragraph::new(self.content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Project Info ")
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            );

        frame.render_widget(paragraph, main_chunks[0]);

        // Help bar at bottom
        let help = Paragraph::new(Line::from(vec![
            Span::styled(
                " i/Esc ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Close"),
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

/// Render project information content
fn render_project_info(info: &ProjectInfo) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "dbt Project Information",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "═══════════════════════════════════════════════════════════",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    // dbt Version
    lines.push(Line::from(vec![
        Span::styled(
            "dbt Version: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(info.dbt_version.clone()),
    ]));
    lines.push(Line::from(""));

    // Project Info
    lines.push(Line::from(Span::styled(
        "Project Configuration",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::raw("  Name: "),
        Span::styled(info.project_name.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Profile: "),
        Span::styled(info.profile_name.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Target: "),
        Span::styled(info.target.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Path: "),
        Span::styled(info.project_path.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));

    // Resource Counts
    lines.push(Line::from(Span::styled(
        "Resources",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::raw("  Models: "),
        Span::styled(
            info.models_count.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Tests: "),
        Span::styled(
            info.tests_count.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Seeds: "),
        Span::styled(
            info.seeds_count.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(""));

    // Profile Details
    lines.push(Line::from(Span::styled(
        format!("Profile Details (Target: {})", info.target),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::raw("  Type: "),
        Span::styled(info.profile_type.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Host: "),
        Span::styled(info.profile_host.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Port: "),
        Span::styled(info.profile_port.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Database: "),
        Span::styled(
            info.profile_database.clone(),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Schema: "),
        Span::styled(
            info.profile_schema.clone(),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  User: "),
        Span::styled(info.profile_user.clone(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Threads: "),
        Span::styled(
            info.profile_threads.clone(),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "═══════════════════════════════════════════════════════════",
        Style::default().fg(Color::DarkGray),
    )));

    lines
}
