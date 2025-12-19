//! Run output dialog component
//!
//! Displays the output of a running or completed dbt command.

use crate::action::Action;
use crate::component::Component;
use crate::model::{ModelRun, ModelRunStatus, RunOutput, RunOutputViewMode, RunStatus};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use unicode_width::UnicodeWidthStr;

/// Run output dialog
#[derive(Default)]
pub struct RunOutputDialog {
    pub scroll_offset: usize,
}

impl Component for RunOutputDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollUp),
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Char('v') => Some(Action::ToggleOutputView),
            KeyCode::Esc | KeyCode::Char('q') => Some(Action::CloseModal),
            KeyCode::Char('h') => Some(Action::OpenHistory),
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
        // This needs run output data, so we use draw_with_output
        Ok(())
    }
}

impl RunOutputDialog {
    pub fn draw_with_output(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        run_output: &RunOutput,
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

        // Get status
        let (status_text, status_color) = get_status_indicator(run_output.status);
        let view_mode_text = get_view_mode_text(run_output.view_mode);

        // Generate content
        let content_lines = match run_output.view_mode {
            RunOutputViewMode::Raw => render_raw_output(run_output),
            RunOutputViewMode::Graphical => render_graphical_output(run_output, content_area.width),
        };

        let total = content_lines.len();
        let visible_height = content_area.height.saturating_sub(2) as usize;
        let scroll = self.scroll_offset.min(total.saturating_sub(visible_height));

        let paragraph = Paragraph::new(content_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(status_color))
                    .title(format!(" Run Output [{}] [{}] ", status_text, view_mode_text))
                    .title_style(
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .scroll((scroll as u16, 0));

        frame.render_widget(paragraph, content_area);

        // Scrollbar
        if total > visible_height {
            let mut scrollbar_state =
                ScrollbarState::new(total.saturating_sub(visible_height)).position(scroll);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                content_area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }

        // Help bar
        let is_running = run_output.status == RunStatus::Running;
        let close_label = if is_running {
            "Back (runs in bg)"
        } else {
            "Close"
        };

        let help = Paragraph::new(Line::from(vec![
            Span::styled(
                " Esc/q ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}  ", close_label)),
            Span::styled(
                " v ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Toggle View  "),
            Span::styled(
                " j/k ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Scroll"),
        ]))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(help, chunks[1]);

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

/// Get status indicator text and color
fn get_status_indicator(status: RunStatus) -> (&'static str, Color) {
    match status {
        RunStatus::Running => ("⏳ RUNNING", Color::Yellow),
        RunStatus::Success => ("✓ SUCCESS", Color::Green),
        RunStatus::Failed => ("✗ FAILED", Color::Red),
    }
}

/// Get view mode text
fn get_view_mode_text(view_mode: RunOutputViewMode) -> &'static str {
    match view_mode {
        RunOutputViewMode::Raw => "Raw",
        RunOutputViewMode::Graphical => "Graph",
    }
}

/// Render the raw text view of run output
fn render_raw_output(run_output: &RunOutput) -> Vec<Line<'static>> {
    let mut output_lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                "Command: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(run_output.command.clone()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "─".repeat(80),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    // Output content
    for line in run_output.output.lines() {
        let styled_line = if line.contains("error") || line.contains("Error") || line.contains("FAILED")
        {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Red),
            ))
        } else if line.contains("warning") || line.contains("Warning") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Yellow),
            ))
        } else if line.contains("SUCCESS")
            || line.contains("Completed successfully")
            || line.contains("PASS")
        {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Green),
            ))
        } else {
            Line::from(line.to_string())
        };
        output_lines.push(styled_line);
    }

    output_lines
}

/// Render the graphical view of run output with model boxes
fn render_graphical_output(run_output: &RunOutput, area_width: u16) -> Vec<Line<'static>> {
    let mut graphical_lines: Vec<Line<'static>> = Vec::new();

    // Command info at top
    graphical_lines.push(Line::from(vec![
        Span::styled(
            "Command: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(run_output.command.clone()),
    ]));
    graphical_lines.push(Line::from(""));

    // Models section
    if run_output.model_runs.is_empty() {
        graphical_lines.push(Line::from(Span::styled(
            "Waiting for models to start...",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Get models organized by layer
        let layers = run_output.get_models_by_layer();
        let box_width = area_width.saturating_sub(4) as usize;

        // Render models by layer with connecting lines
        for (layer_idx, layer_models) in layers.iter().enumerate() {
            if layer_models.is_empty() {
                continue;
            }

            // Add layer label
            let layer_label = if layer_idx == 0 {
                "Layer 0 (Root)".to_string()
            } else {
                format!("Layer {}", layer_idx)
            };
            graphical_lines.push(Line::from(vec![
                Span::styled(
                    format!("─── {} ", layer_label),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    "─".repeat(box_width.saturating_sub(layer_label.len() + 6)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            graphical_lines.push(Line::from(""));

            // Render each model in this layer
            for (model_idx, model) in layer_models.iter().enumerate() {
                // Show upstream dependencies if any
                if !model.upstream_deps.is_empty() {
                    let deps_text = format!("  ↑ depends on: {}", model.upstream_deps.join(", "));
                    graphical_lines.push(Line::from(Span::styled(
                        deps_text,
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                // Render the model box
                let indent = "  ";
                let model_lines = render_model_box(model, box_width, indent);
                graphical_lines.extend(model_lines);

                // Add spacing between models in same layer
                if model_idx < layer_models.len() - 1 {
                    graphical_lines.push(Line::from(""));
                }
            }

            // Draw connector lines to next layer
            if layer_idx < layers.len() - 1 {
                let next_layer = &layers[layer_idx + 1];
                if !next_layer.is_empty() {
                    graphical_lines.push(Line::from(""));

                    // Check if any models in next layer depend on models in current layer
                    let has_connections = next_layer.iter().any(|m| !m.upstream_deps.is_empty());
                    if has_connections {
                        graphical_lines.push(Line::from(Span::styled(
                            "      │",
                            Style::default().fg(Color::Cyan),
                        )));
                        graphical_lines.push(Line::from(Span::styled(
                            "      ▼",
                            Style::default().fg(Color::Cyan),
                        )));
                    }
                    graphical_lines.push(Line::from(""));
                }
            }
        }
    }

    graphical_lines
}

/// Render a single model box for the graphical view
fn render_model_box(
    model: &ModelRun,
    box_width: usize,
    indent: &str,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Determine colors based on status
    let (status_icon, border_color, status_style) = match model.status {
        ModelRunStatus::Running => ("⏳", Color::Yellow, Style::default().fg(Color::Yellow)),
        ModelRunStatus::Success => ("✓", Color::Green, Style::default().fg(Color::Green)),
        ModelRunStatus::Failed => ("✗", Color::Red, Style::default().fg(Color::Red)),
        ModelRunStatus::Skipped => ("⊘", Color::DarkGray, Style::default().fg(Color::DarkGray)),
    };

    let indent_width = indent.width();
    let adjusted_box_width = box_width.saturating_sub(indent_width);

    // Top border (dotted)
    let top_border = format!(
        "{}┌{}┐",
        indent,
        "╌".repeat(adjusted_box_width.saturating_sub(2))
    );
    lines.push(Line::from(Span::styled(
        top_border,
        Style::default().fg(border_color),
    )));

    // Model name line with status icon
    // Structure: │ + space + icon + space + step_info + badge + space + name + padding + │
    // Total fixed chars: 2 (│ + space) + 1 (space after icon) + 1 (space before name) + 1 (│) = 5
    let step_info = model
        .step
        .as_ref()
        .map(|s| format!("[{}] ", s))
        .unwrap_or_default();
    let model_type_badge = format!("[{}]", model.model_type);
    let content_width =
        status_icon.width() + step_info.width() + model_type_badge.width() + model.name.width();
    // padding = box_width - 5 fixed chars - content
    let padding = adjusted_box_width.saturating_sub(content_width + 5);
    lines.push(Line::from(vec![
        Span::raw(indent.to_string()),
        Span::styled("│ ", Style::default().fg(border_color)),
        Span::styled(status_icon, status_style),
        Span::styled(format!(" {}", step_info), Style::default().fg(Color::DarkGray)),
        Span::styled(model_type_badge, Style::default().fg(Color::Cyan)),
        Span::styled(
            format!(" {}", model.name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(padding)),
        Span::styled("│", Style::default().fg(border_color)),
    ]));

    // Result info line (if available)
    // Structure: │ + 3 spaces + result_text + padding + │
    // Total fixed chars: 1 (│) + 3 (spaces) + 1 (│) = 5
    if model.status != ModelRunStatus::Running {
        let result_text = match (&model.result_info, model.duration) {
            (Some(info), Some(dur)) => format!("{} in {:.2}s", info, dur),
            (Some(info), None) => info.clone(),
            (None, Some(dur)) => format!("Completed in {:.2}s", dur),
            (None, None) => "Completed".to_string(),
        };
        let result_padding = adjusted_box_width.saturating_sub(result_text.width() + 5);
        lines.push(Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled("│   ", Style::default().fg(border_color)),
            Span::styled(result_text, Style::default().fg(Color::DarkGray)),
            Span::raw(" ".repeat(result_padding)),
            Span::styled("│", Style::default().fg(border_color)),
        ]));
    } else {
        // Running indicator
        let running_text = "Running...";
        let result_padding = adjusted_box_width.saturating_sub(running_text.width() + 5);
        lines.push(Line::from(vec![
            Span::raw(indent.to_string()),
            Span::styled("│   ", Style::default().fg(border_color)),
            Span::styled(running_text, Style::default().fg(Color::Yellow)),
            Span::raw(" ".repeat(result_padding)),
            Span::styled("│", Style::default().fg(border_color)),
        ]));
    }

    // Bottom border (dotted)
    let bottom_border = format!(
        "{}└{}┘",
        indent,
        "╌".repeat(adjusted_box_width.saturating_sub(2))
    );
    lines.push(Line::from(Span::styled(
        bottom_border,
        Style::default().fg(border_color),
    )));

    lines
}
