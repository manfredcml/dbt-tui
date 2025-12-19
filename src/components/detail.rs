//! Detail panel component
//!
//! Displays SQL code, test info, or seed data based on the selected node.

use crate::action::Action;
use crate::component::Component;
use crate::model::{CodeViewMode, Node};
use super::sql_highlight;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use super::table::TableComponent;

/// Content type being displayed
#[derive(Debug, Clone, PartialEq)]
enum ContentType {
    /// SQL code (model or snapshot)
    Sql,
    /// Test information
    Test,
    /// Seed data - delegate to TableComponent
    Seed,
    /// Error message
    Error,
    /// No node selected
    Empty,
}

/// Detail panel component for displaying node information
pub struct DetailComponent {
    /// Current scroll offset (for non-seed content)
    scroll: usize,
    /// Cached content lines (header + content for non-seeds, just header for seeds)
    content: Vec<Line<'static>>,
    /// Child table component for seed data
    table: TableComponent,
    /// Current code view mode
    code_view_mode: CodeViewMode,
    /// Type of content being displayed
    content_type: ContentType,
}

impl Default for DetailComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl DetailComponent {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            content: Vec::new(),
            table: TableComponent::new(),
            code_view_mode: CodeViewMode::Compiled,
            content_type: ContentType::Empty,
        }
    }

    /// Update content based on the selected node
    pub fn set_node(&mut self, node: Option<&Node>, code_view_mode: CodeViewMode) {
        self.code_view_mode = code_view_mode;
        self.scroll = 0;

        match node {
            Some(n) => {
                if n.resource_type == "seed" {
                    self.content_type = ContentType::Seed;
                    self.content = self.render_seed_header();
                    // Load seed data into TableComponent
                    match n.read_seed_data() {
                        Ok((headers, rows)) => {
                            self.table.set_data(headers, rows);
                        }
                        Err(e) => {
                            // On error, fall back to showing error in content
                            self.content_type = ContentType::Error;
                            self.content.push(Line::from(Span::styled(
                                format!("Error reading seed data: {}", e),
                                Style::default().fg(Color::Red),
                            )));
                        }
                    }
                } else if n.resource_type == "test" {
                    self.content_type = ContentType::Test;
                    self.content = self.render_node_detail(n);
                } else {
                    self.content_type = ContentType::Sql;
                    self.content = self.render_node_detail(n);
                }
            }
            None => {
                self.content_type = ContentType::Empty;
                self.content = vec![Line::from("No node selected")];
            }
        }
    }

    /// Get panel title based on content type
    pub fn get_title(&self) -> &'static str {
        match self.content_type {
            ContentType::Seed => " Seed Data ",
            _ => match self.code_view_mode {
                CodeViewMode::Compiled => " Compiled SQL ",
                CodeViewMode::Original => " Original SQL ",
            },
        }
    }

    /// Render just the header for seed display
    fn render_seed_header(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(vec![
                Span::styled(
                    "'c'",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Toggle SQL  "),
                Span::styled(
                    "'l'",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Lineage  "),
                Span::styled(
                    "'d'",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Docs"),
            ]),
            Line::from(""),
        ]
    }

    fn render_node_detail(&mut self, node: &Node) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add shortcuts hint at the top
        lines.push(Line::from(vec![
            Span::styled(
                "'c'",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Toggle SQL  "),
            Span::styled(
                "'l'",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Lineage  "),
            Span::styled(
                "'d'",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Docs"),
        ]));
        lines.push(Line::from(""));

        if node.resource_type == "test" {
            // For tests, show test info and YAML definition
            self.render_test_detail(&mut lines, node);
        } else {
            // For models, show SQL
            self.render_model_detail(&mut lines, node);
        }

        lines
    }

    fn render_test_detail(&self, lines: &mut Vec<Line<'static>>, node: &Node) {
        lines.push(Line::from(Span::styled(
            "Test Information:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "═══════════════════════════════════════════════════════════",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        let test_type = extract_test_type(&node.name);
        lines.push(Line::from(vec![
            Span::styled("Test Type: ", Style::default().fg(Color::Cyan)),
            Span::raw(test_type),
        ]));

        let tested_models: Vec<&str> = node
            .depends_on
            .nodes
            .iter()
            .filter(|dep| dep.starts_with("model."))
            .map(|dep| dep.rsplit('.').next().unwrap_or(dep))
            .collect();

        let model_name = tested_models.first().copied().unwrap_or("");

        if !tested_models.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Tests Model: ", Style::default().fg(Color::Cyan)),
                Span::raw(tested_models.join(", ")),
            ]));
        }

        if let Some(ref path) = node.original_file_path {
            lines.push(Line::from(vec![
                Span::styled("Source File: ", Style::default().fg(Color::Cyan)),
                Span::styled(path.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }

        lines.push(Line::from(""));

        // Try to get YAML definition for schema tests
        if let Some(yaml_content) = node.get_test_yaml_definition(model_name) {
            lines.push(Line::from(Span::styled(
                "YAML Definition:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "═══════════════════════════════════════════════════════════",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));

            for yaml_line in yaml_content.lines() {
                lines.push(highlight_yaml_line(yaml_line));
            }
        } else {
            // Fall back to showing compiled SQL if available
            lines.push(Line::from(Span::styled(
                "Compiled SQL:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "═══════════════════════════════════════════════════════════",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));

            if let Some(sql) = node.get_compiled_sql() {
                let trimmed_sql = trim_sql(&sql);
                let highlighted = sql_highlight::highlight_sql(&trimmed_sql);
                lines.extend(highlighted);
            } else {
                lines.push(Line::from(Span::styled(
                    "No YAML definition or compiled SQL available.",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(Span::styled(
                    "Run 'dbt compile' to generate compiled SQL.",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    fn render_model_detail(&self, lines: &mut Vec<Line<'static>>, node: &Node) {
        let mode_label = match self.code_view_mode {
            CodeViewMode::Compiled => "Compiled SQL",
            CodeViewMode::Original => "Original SQL",
        };

        lines.push(Line::from(Span::styled(
            format!("{}:", mode_label),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "═══════════════════════════════════════════════════════════",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        let sql = match self.code_view_mode {
            CodeViewMode::Compiled => node.get_compiled_sql().unwrap_or_else(|| {
                "Compiled SQL not available. Please run 'dbt compile'.".to_string()
            }),
            CodeViewMode::Original => node
                .get_raw_sql()
                .unwrap_or_else(|| "No raw SQL available".to_string()),
        };
        let trimmed_sql = trim_sql(&sql);
        let highlighted = sql_highlight::highlight_sql(&trimmed_sql);
        lines.extend(highlighted);
    }
}

impl Component for DetailComponent {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // For seeds, delegate key events to TableComponent
        if self.content_type == ContentType::Seed {
            return self.table.handle_key_event(key);
        }

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
        // For seeds, delegate updates to TableComponent
        if self.content_type == ContentType::Seed {
            return self.table.update(action);
        }

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
                self.scroll = (self.scroll + 20).min(max_scroll);
            }
            Action::PageUp => {
                self.scroll = self.scroll.saturating_sub(20);
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // For seeds, split the area: header at top, table below
        if self.content_type == ContentType::Seed {
            // Create outer block
            let block = Block::default()
                .borders(Borders::ALL)
                .title(self.get_title())
                .border_style(Style::default().fg(Color::DarkGray));

            let inner_area = block.inner(area);
            frame.render_widget(block, area);

            // Split inner area: header (3 lines) + table
            let header_height = self.content.len() as u16;
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(header_height),
                    Constraint::Min(0),
                ])
                .split(inner_area);

            // Render header
            let header = Paragraph::new(self.content.clone());
            frame.render_widget(header, chunks[0]);

            // Delegate table rendering to TableComponent
            self.table.draw(frame, chunks[1])?;

            return Ok(());
        }

        // For non-seeds, render as before
        let visible_height = area.height.saturating_sub(2) as usize;

        let paragraph = Paragraph::new(self.content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.get_title())
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

// ═══════════════════════════════════════════════════════════════════════════════
// Helper functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Highlight a YAML line with basic syntax coloring
fn highlight_yaml_line(line: &str) -> Line<'static> {
    let trimmed = line.trim();

    // Check if line contains a key-value pair
    if let Some(colon_pos) = trimmed.find(':') {
        let key = &trimmed[..colon_pos];
        let value = &trimmed[colon_pos + 1..];

        // Calculate leading whitespace
        let indent = line.len() - line.trim_start().len();
        let indent_str = " ".repeat(indent);

        // Handle list items (- key: value)
        if let Some(actual_key) = key.strip_prefix("- ") {
            return Line::from(vec![
                Span::raw(indent_str),
                Span::styled("- ", Style::default().fg(Color::White)),
                Span::styled(actual_key.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(":", Style::default().fg(Color::White)),
                Span::styled(value.to_string(), Style::default().fg(Color::Green)),
            ]);
        }

        return Line::from(vec![
            Span::raw(indent_str),
            Span::styled(key.to_string(), Style::default().fg(Color::Cyan)),
            Span::styled(":", Style::default().fg(Color::White)),
            Span::styled(value.to_string(), Style::default().fg(Color::Green)),
        ]);
    }

    // Handle simple list items (- value)
    if let Some(value) = trimmed.strip_prefix("- ") {
        let indent = line.len() - line.trim_start().len();
        let indent_str = " ".repeat(indent);
        return Line::from(vec![
            Span::raw(indent_str),
            Span::styled("- ", Style::default().fg(Color::White)),
            Span::styled(value.to_string(), Style::default().fg(Color::Yellow)),
        ]);
    }

    // Comments
    if trimmed.starts_with('#') {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Default: return line as-is
    Line::from(line.to_string())
}

/// Extract test type from a dbt test name
fn extract_test_type(test_name: &str) -> String {
    let test_types = [
        "not_null",
        "unique",
        "accepted_values",
        "relationships",
        "dbt_expectations",
        "dbt_utils",
    ];

    for test_type in test_types {
        if test_name.starts_with(test_type) {
            return test_type.to_string();
        }
    }

    test_name
        .split('_')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

/// Trim leading and trailing whitespace from SQL code
pub fn trim_sql(sql: &str) -> String {
    let lines: Vec<&str> = sql.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    let trimmed_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else if line.len() > min_indent {
                line[min_indent..].trim_end().to_string()
            } else {
                line.trim_end().to_string()
            }
        })
        .collect();

    let result = trimmed_lines.join("\n");
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_test_type_not_null() {
        assert_eq!(
            extract_test_type("not_null_customers_customer_id"),
            "not_null"
        );
    }

    #[test]
    fn test_extract_test_type_unique() {
        assert_eq!(extract_test_type("unique_orders_order_id"), "unique");
    }

    #[test]
    fn test_extract_test_type_accepted_values() {
        assert_eq!(
            extract_test_type("accepted_values_status__active__inactive"),
            "accepted_values"
        );
    }

    #[test]
    fn test_trim_sql_removes_leading_whitespace() {
        let sql = "    SELECT *\n    FROM table";
        assert_eq!(trim_sql(sql), "SELECT *\nFROM table");
    }

    #[test]
    fn test_highlight_yaml_line_key_value() {
        let line = "  name: customers";
        let result = highlight_yaml_line(line);
        assert!(!result.spans.is_empty());
    }
}
