//! Setup wizard component
//!
//! Interactive setup for first-time configuration of dbt-tui.

use crate::action::Action;
use crate::component::Component;
use crate::config::Config;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Setup wizard step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStep {
    Welcome,
    ProjectPath,
    DbtBinaryPath,
    Confirm,
}

impl SetupStep {
    fn next(&self) -> Option<SetupStep> {
        match self {
            SetupStep::Welcome => Some(SetupStep::ProjectPath),
            SetupStep::ProjectPath => Some(SetupStep::DbtBinaryPath),
            SetupStep::DbtBinaryPath => Some(SetupStep::Confirm),
            SetupStep::Confirm => None,
        }
    }

    fn prev(&self) -> Option<SetupStep> {
        match self {
            SetupStep::Welcome => None,
            SetupStep::ProjectPath => Some(SetupStep::Welcome),
            SetupStep::DbtBinaryPath => Some(SetupStep::ProjectPath),
            SetupStep::Confirm => Some(SetupStep::DbtBinaryPath),
        }
    }

    fn title(&self) -> &str {
        match self {
            SetupStep::Welcome => "Welcome",
            SetupStep::ProjectPath => "Project Path",
            SetupStep::DbtBinaryPath => "dbt Binary",
            SetupStep::Confirm => "Confirm",
        }
    }

    fn step_number(&self) -> usize {
        match self {
            SetupStep::Welcome => 1,
            SetupStep::ProjectPath => 2,
            SetupStep::DbtBinaryPath => 3,
            SetupStep::Confirm => 4,
        }
    }
}

/// Setup wizard component
pub struct SetupComponent {
    /// Current step
    pub step: SetupStep,
    /// Config being built
    pub config: Config,
    /// Current input text
    pub input: String,
    /// Error message to display
    pub error: Option<String>,
    /// Whether setup is complete
    pub complete: bool,
}

impl Default for SetupComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupComponent {
    pub fn new() -> Self {
        Self {
            step: SetupStep::Welcome,
            config: Config::default(),
            input: String::new(),
            error: None,
            complete: false,
        }
    }

    /// Get the saved config if setup completed successfully
    pub fn get_config(&self) -> Option<&Config> {
        if self.complete {
            Some(&self.config)
        } else {
            None
        }
    }

    fn validate_current_step(&mut self) -> bool {
        self.error = None;

        match self.step {
            SetupStep::Welcome => true,
            SetupStep::ProjectPath => {
                if self.input.is_empty() {
                    self.error = Some("Project path is required".to_string());
                    return false;
                }
                let path = std::path::PathBuf::from(&self.input);
                if !path.exists() {
                    self.error = Some(format!("Path does not exist: {}", self.input));
                    return false;
                }
                if !path.is_dir() {
                    self.error = Some("Path must be a directory".to_string());
                    return false;
                }
                let dbt_project = path.join("dbt_project.yml");
                if !dbt_project.exists() {
                    self.error = Some("No dbt_project.yml found in this directory".to_string());
                    return false;
                }
                self.config.project_path = self.input.clone();
                true
            }
            SetupStep::DbtBinaryPath => {
                if self.input.is_empty() {
                    self.error = Some("dbt binary path is required".to_string());
                    return false;
                }
                let path = std::path::PathBuf::from(&self.input);
                if !path.exists() {
                    self.error = Some(format!("dbt binary not found: {}", self.input));
                    return false;
                }
                self.config.dbt_binary_path = self.input.clone();
                true
            }
            SetupStep::Confirm => true,
        }
    }

    fn advance_step(&mut self) {
        if self.validate_current_step() {
            if let Some(next) = self.step.next() {
                self.step = next;
                // Pre-populate input for next step
                self.input = match self.step {
                    SetupStep::ProjectPath => self.config.project_path.clone(),
                    SetupStep::DbtBinaryPath => {
                        if self.config.dbt_binary_path.is_empty() {
                            // Try to find dbt in common locations
                            Self::find_dbt_binary().unwrap_or_default()
                        } else {
                            self.config.dbt_binary_path.clone()
                        }
                    }
                    _ => String::new(),
                };
            } else {
                // On confirm step, save the config
                self.save_config();
            }
        }
    }

    fn go_back(&mut self) {
        if let Some(prev) = self.step.prev() {
            self.step = prev;
            self.error = None;
            // Restore input for previous step
            self.input = match self.step {
                SetupStep::Welcome => String::new(),
                SetupStep::ProjectPath => self.config.project_path.clone(),
                SetupStep::DbtBinaryPath => self.config.dbt_binary_path.clone(),
                SetupStep::Confirm => String::new(),
            };
        }
    }

    fn save_config(&mut self) {
        match self.config.save() {
            Ok(()) => {
                self.complete = true;
            }
            Err(e) => {
                self.error = Some(format!("Failed to save config: {}", e));
            }
        }
    }

    /// Try to find dbt binary in common locations
    fn find_dbt_binary() -> Option<String> {
        let candidates = [
            // Check if dbt is in PATH via which
            std::process::Command::new("which")
                .arg("dbt")
                .output()
                .ok()
                .and_then(|out| {
                    if out.status.success() {
                        String::from_utf8(out.stdout).ok().map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                }),
        ];

        for candidate in candidates.into_iter().flatten() {
            if !candidate.is_empty() && std::path::Path::new(&candidate).exists() {
                return Some(candidate);
            }
        }

        // Common default locations
        let home = std::env::var("HOME").ok()?;
        let common_paths = [
            format!("{}/.local/bin/dbt", home),
            format!("{}/venv/bin/dbt", home),
            format!("{}/.venv/bin/dbt", home),
            "/usr/local/bin/dbt".to_string(),
            "/opt/homebrew/bin/dbt".to_string(),
        ];

        common_paths
            .into_iter()
            .find(|path| std::path::Path::new(path).exists())
    }
}

impl Component for SetupComponent {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match self.step {
            SetupStep::Welcome => match key.code {
                KeyCode::Enter => {
                    self.advance_step();
                    Ok(None)
                }
                KeyCode::Esc => Ok(Some(Action::ForceQuit)),
                _ => Ok(None),
            },
            SetupStep::ProjectPath | SetupStep::DbtBinaryPath => match key.code {
                KeyCode::Enter => {
                    self.advance_step();
                    Ok(None)
                }
                KeyCode::Esc => {
                    self.go_back();
                    Ok(None)
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    self.error = None;
                    Ok(None)
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    self.error = None;
                    Ok(None)
                }
                _ => Ok(None),
            },
            SetupStep::Confirm => match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.save_config();
                    if self.complete {
                        Ok(Some(Action::SetupConfirm))
                    } else {
                        Ok(None)
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.go_back();
                    Ok(None)
                }
                KeyCode::Backspace => {
                    self.go_back();
                    Ok(None)
                }
                _ => Ok(None),
            },
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Clear the screen
        frame.render_widget(Clear, area);
        let background = Block::default().style(Style::default().bg(Color::Reset));
        frame.render_widget(background, area);

        let margin = 4;
        let content_area = Rect::new(
            margin,
            margin,
            area.width.saturating_sub(margin * 2),
            area.height.saturating_sub(margin * 2),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Progress
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Help
            ])
            .split(content_area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                " dbt-tui Setup ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Progress indicator
        let progress = format!(
            "Step {} of 4: {}",
            self.step.step_number(),
            self.step.title()
        );
        let progress_widget = Paragraph::new(Line::from(vec![
            Span::styled(progress, Style::default().fg(Color::DarkGray)),
        ]));
        frame.render_widget(progress_widget, chunks[1]);

        // Content based on step
        self.draw_step_content(frame, chunks[2]);

        // Help bar
        let help_text = match self.step {
            SetupStep::Welcome => " Enter  Continue   Esc  Quit",
            SetupStep::ProjectPath | SetupStep::DbtBinaryPath => {
                " Enter  Continue   Esc  Back   Type to edit"
            }
            SetupStep::Confirm => " Enter/y  Save & Continue   Esc/n  Go Back",
        };
        let help = Paragraph::new(Line::from(vec![Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )]))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);

        Ok(())
    }
}

impl SetupComponent {
    fn draw_step_content(&self, frame: &mut Frame, area: Rect) {
        match self.step {
            SetupStep::Welcome => self.draw_welcome(frame, area),
            SetupStep::ProjectPath => self.draw_project_path(frame, area),
            SetupStep::DbtBinaryPath => self.draw_dbt_binary_path(frame, area),
            SetupStep::Confirm => self.draw_confirm(frame, area),
        }
    }

    fn draw_welcome(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Welcome to dbt-tui!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("This wizard will help you configure dbt-tui for your dbt project."),
            Line::from(""),
            Line::from("You will need to provide:"),
            Line::from(vec![Span::styled(
                "  1. Path to your dbt project directory",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(vec![Span::styled(
                "  2. Path to your dbt binary",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Enter to begin...",
                Style::default().fg(Color::Yellow),
            )]),
        ];

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Welcome ")
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(paragraph, area);
    }

    fn draw_project_path(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(""),
            Line::from("Enter the path to your dbt project directory:"),
            Line::from("(The directory containing dbt_project.yml)"),
            Line::from(""),
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}_", &self.input),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        if let Some(ref error) = self.error {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!("Error: {}", error),
                Style::default().fg(Color::Red),
            )]));
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Project Path ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(paragraph, area);
    }

    fn draw_dbt_binary_path(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(""),
            Line::from("Enter the path to your dbt binary:"),
            Line::from("(e.g., /usr/local/bin/dbt or ~/.venv/bin/dbt)"),
            Line::from(""),
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}_", &self.input),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        if !self.input.is_empty() {
            let path = std::path::Path::new(&self.input);
            if path.exists() {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "✓ Path exists",
                    Style::default().fg(Color::Green),
                )]));
            }
        }

        if let Some(ref error) = self.error {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!("Error: {}", error),
                Style::default().fg(Color::Red),
            )]));
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" dbt Binary Path ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(paragraph, area);
    }

    fn draw_confirm(&self, frame: &mut Frame, area: Rect) {
        let config_dir = Config::config_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.dbt-tui".to_string());

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Review your configuration:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Project Path: ", Style::default().fg(Color::Cyan)),
                Span::raw(&self.config.project_path),
            ]),
            Line::from(vec![
                Span::styled("dbt Binary:   ", Style::default().fg(Color::Cyan)),
                Span::raw(&self.config.dbt_binary_path),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Config will be saved to: ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{}/config.json", config_dir)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Enter or 'y' to save and continue...",
                Style::default().fg(Color::Yellow),
            )]),
        ];

        if let Some(ref error) = self.error {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!("Error: {}", error),
                Style::default().fg(Color::Red),
            )]));
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Configuration ")
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(paragraph, area);
    }
}
