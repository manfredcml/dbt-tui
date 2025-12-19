//! Quit confirmation dialog component

use crate::action::Action;
use crate::component::Component;
use crate::components::centered_popup;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Quit confirmation dialog
pub struct QuitDialog;

impl Default for QuitDialog {
    fn default() -> Self {
        Self
    }
}

impl Component for QuitDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(Action::ForceQuit),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::CloseModal),
            _ => None,
        };
        Ok(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let popup_area = centered_popup(area, 40, 7);

        frame.render_widget(Clear, popup_area);

        let content = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Are you sure you want to quit?",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " y ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Yes, quit  "),
                Span::styled(
                    " n/Esc ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("No, cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(" Quit? ")
                    .title_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, popup_area);
        Ok(())
    }
}
