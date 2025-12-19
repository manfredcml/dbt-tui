//! Splash screen component
//!
//! Displays the dbt logo briefly before transitioning to the main app.

use crate::action::Action;
use crate::component::Component;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

/// Splash screen component
pub struct SplashComponent {
    /// When the splash screen was shown
    start_time: Option<Instant>,
    /// Duration to show splash before auto-advancing
    duration: Duration,
}

impl Default for SplashComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl SplashComponent {
    pub fn new() -> Self {
        Self {
            start_time: None,
            duration: Duration::from_millis(1500),
        }
    }

    /// Check if splash duration has elapsed
    pub fn is_complete(&self) -> bool {
        self.start_time
            .map(|t| t.elapsed() >= self.duration)
            .unwrap_or(false)
    }

    /// Get the dbt logo as ASCII art
    fn get_logo() -> Vec<&'static str> {
        vec![
            " *====                     =====                                                                    ",
            "===  =====             ======  ==                                                                   ",
            "===  ========       ========   ==                    @@@@@@     @@@@@                               ",
            " ================================                    @@@@@@     @@@@@                               ",
            "  =============================                      @@@@@@     @@@@@                   @@@@@       ",
            "   ==================== ======                       @@@@@@     @@@@@                   @@@@@       ",
            "    =====================  ==               @@@@@@@@@@@@@@@     @@@@@  @@@@@@@@@     @@@@@@@@@@@@@@ ",
            "     =========     =======                @@@@@@@@@@@@@@@@@     @@@@@ @@@@@@@@@@@@   @@@@@@@@@@@@@@ ",
            "      =======   ===========              @@@@@@      @@@@@@     @@@@@       @@@@@@      @@@@@       ",
            "      =======  =============            @@@@@@       @@@@@@     @@@@@        @@@@@@     @@@@@       ",
            "     =========  =============           @@@@@@       @@@@@@     @@@@@        @@@@@@     @@@@@       ",
            "    ==========================          @@@@@@       @@@@@@     @@@@@        @@@@@      @@@@@       ",
            "   ======= ====================         @@@@@@       @@@@@@     @@@@@        @@@@@      @@@@@       ",
            "  =========   ==================        @@@@@@@      @@@@@@     @@@@@       @@@@@@      @@@@@@      ",
            " ============    ================        @@@@@@@@@@@@@@@@@@     @@@@@@@@@@@@@@@@@       @@@@@@@@@@@ ",
            "===  ========       ========   ===         @@@@@@@@*  @@@@@     @@@@@@@@@@@@@@@          @@@@@@@@@@ ",
            "===  =====              ===== ===                                                                   ",
            "  ====                      ====                                                                    ",
        ]
    }
}

impl Component for SplashComponent {
    fn init(&mut self) -> Result<()> {
        self.start_time = Some(Instant::now());
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // Any key press skips the splash screen
        match key.code {
            KeyCode::Char('q') => Ok(Some(Action::ForceQuit)),
            _ => Ok(Some(Action::SplashComplete)),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if action == Action::Tick && self.is_complete() {
            return Ok(Some(Action::SplashComplete));
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Use true black RGB for consistent appearance across terminal themes
        let bg_black = Color::Rgb(0, 0, 0);

        // Clear the area first, then fill with true black background
        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::default().style(Style::default().bg(bg_black)),
            area,
        );

        let logo_lines = Self::get_logo();
        let logo_height = logo_lines.len() as u16;
        let logo_width = logo_lines.first().map(|l| l.len()).unwrap_or(0) as u16;

        // Center the logo
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(logo_height + 6)) / 2),
                Constraint::Length(logo_height),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let logo_area = chunks[1];

        // Render logo with color - all characters get true black background
        let logo_paragraph: Vec<Line> = logo_lines
            .iter()
            .map(|line| {
                let styled_line: Vec<Span> = line
                    .chars()
                    .map(|c| {
                        let style = match c {
                            '=' | '*' => Style::default().fg(Color::Rgb(255, 107, 53)).bg(bg_black), // Orange for dbt logo
                            '@' => Style::default().fg(Color::White).bg(bg_black), // White for "dbt" text
                            _ => Style::default().fg(bg_black).bg(bg_black), // Black on black for spaces
                        };
                        Span::styled(c.to_string(), style)
                    })
                    .collect();
                Line::from(styled_line)
            })
            .collect();

        let centered_x = (area.width.saturating_sub(logo_width)) / 2;
        let logo_rect = Rect::new(centered_x, logo_area.y, logo_width, logo_height);

        frame.render_widget(Paragraph::new(logo_paragraph), logo_rect);

        // Render title
        let title = Line::from(vec![
            Span::styled(
                "dbt",
                Style::default()
                    .fg(Color::Rgb(255, 107, 53))
                    .bg(bg_black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "-tui",
                Style::default()
                    .fg(Color::White)
                    .bg(bg_black)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let title_width = 7; // "dbt-tui"
        let title_x = (area.width.saturating_sub(title_width)) / 2;
        let title_rect = Rect::new(title_x, chunks[3].y, title_width, 1);

        frame.render_widget(Paragraph::new(title), title_rect);

        // Render subtitle
        let subtitle = Line::from(Span::styled(
            "A terminal UI for dbt",
            Style::default().fg(Color::DarkGray).bg(bg_black),
        ));

        let subtitle_width = 21;
        let subtitle_x = (area.width.saturating_sub(subtitle_width)) / 2;
        let subtitle_rect = Rect::new(subtitle_x, chunks[4].y, subtitle_width, 1);

        frame.render_widget(Paragraph::new(subtitle), subtitle_rect);

        Ok(())
    }
}
