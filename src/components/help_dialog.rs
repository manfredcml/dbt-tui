//! Help dialog component
//!
//! Displays all keyboard shortcuts available in the application.

use crate::action::Action;
use crate::component::Component;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Help dialog showing all keyboard shortcuts
#[derive(Default)]
pub struct HelpDialog {
    pub scroll_offset: usize,
}

impl Component for HelpDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Some(Action::CloseModal),
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                None
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
                None
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                None
            }
            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, _action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Clear the area
        frame.render_widget(Clear, area);

        let margin = 4;
        let dialog_area = Rect::new(
            margin,
            margin,
            area.width.saturating_sub(margin * 2),
            area.height.saturating_sub(margin * 2),
        );

        let content = build_help_content();
        let total = content.len();
        let visible_height = dialog_area.height.saturating_sub(2) as usize;

        // Clamp scroll offset
        let max_scroll = total.saturating_sub(visible_height);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }

        let paragraph = Paragraph::new(content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard Shortcuts ")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, dialog_area);

        // Render scrollbar if content exceeds visible area
        if total > visible_height {
            let mut scrollbar_state =
                ScrollbarState::new(total.saturating_sub(visible_height)).position(self.scroll_offset);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                dialog_area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        Ok(())
    }
}

/// Build the help content with all keyboard shortcuts
fn build_help_content() -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Helper to add a section header
    let add_section = |lines: &mut Vec<Line<'static>>, title: &str| {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {} ", title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {}", "─".repeat(title.len() + 2)),
            Style::default().fg(Color::DarkGray),
        )));
    };

    // Helper to add a shortcut line
    let add_shortcut = |lines: &mut Vec<Line<'static>>, key: &str, description: &str| {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:12}", key),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(description.to_string(), Style::default().fg(Color::White)),
        ]));
    };

    // Navigation
    add_section(&mut lines, "Navigation");
    add_shortcut(&mut lines, "j / ↓", "Move to next item");
    add_shortcut(&mut lines, "k / ↑", "Move to previous item");
    add_shortcut(&mut lines, "g", "Jump to first item");
    add_shortcut(&mut lines, "G", "Jump to last item");
    add_shortcut(&mut lines, "Tab", "Next tab (Models/Tests/Seeds)");
    add_shortcut(&mut lines, "Shift+Tab", "Previous tab");

    // Scrolling
    add_section(&mut lines, "Scrolling (Detail Panel)");
    add_shortcut(&mut lines, "Ctrl+e", "Scroll down one line");
    add_shortcut(&mut lines, "Ctrl+y", "Scroll up one line");
    add_shortcut(&mut lines, "Ctrl+d", "Scroll down half page");
    add_shortcut(&mut lines, "Ctrl+u", "Scroll up half page");

    // View Toggles
    add_section(&mut lines, "View Toggles");
    add_shortcut(&mut lines, "c", "Toggle compiled/original SQL");
    add_shortcut(&mut lines, "l", "Toggle lineage panel");
    add_shortcut(&mut lines, "d", "Toggle documentation panel");

    // Modals & Dialogs
    add_section(&mut lines, "Dialogs");
    add_shortcut(&mut lines, "r / Enter", "Open run options");
    add_shortcut(&mut lines, "h", "Open run history");
    add_shortcut(&mut lines, "i", "Open project info");
    add_shortcut(&mut lines, "t", "Open target selector");
    add_shortcut(&mut lines, "f", "Open tag filter");
    add_shortcut(&mut lines, "?", "Show this help");
    add_shortcut(&mut lines, "q", "Quit / Close dialog");

    // Model Actions
    add_section(&mut lines, "Model Actions");
    add_shortcut(&mut lines, "e", "Edit file in $EDITOR");
    add_shortcut(&mut lines, "p", "Preview sample data (dbt show)");

    // Search
    add_section(&mut lines, "Search");
    add_shortcut(&mut lines, "/", "Enter search mode");
    add_shortcut(&mut lines, "Esc", "Exit search / Cancel");
    add_shortcut(&mut lines, "Enter", "Confirm search");

    // Selection
    add_section(&mut lines, "Multi-Select");
    add_shortcut(&mut lines, "Space", "Toggle node selection");
    add_shortcut(&mut lines, "Ctrl+a", "Select all visible nodes");
    add_shortcut(&mut lines, "Esc", "Clear selection");

    // Git Operations
    add_section(&mut lines, "Git Operations");
    add_shortcut(&mut lines, "D", "View git diff");
    add_shortcut(&mut lines, "A", "Stage file (git add)");
    add_shortcut(&mut lines, "K", "Open commit dialog");
    add_shortcut(&mut lines, "L", "View git log");

    // Project
    add_section(&mut lines, "Project");
    add_shortcut(&mut lines, "R", "Refresh manifest");

    // Footer
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press q, Esc, or ? to close",
        Style::default().fg(Color::DarkGray),
    )));

    lines
}
