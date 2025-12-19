//! Home component - Main application screen
//!
//! Displays tabs, node list, detail panel, lineage, and documentation.
//! Owns navigation state and logic.

use crate::action::Action;
use crate::component::Component;
use crate::components::calculate_main_layout;
use crate::model::node::Node;
use crate::model::ui::{CodeViewMode, Tab};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};
use std::collections::{BTreeMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════════
// Display List Item
// ═══════════════════════════════════════════════════════════════════════════════

/// Display list item for grouped node display
#[derive(Debug, Clone)]
pub enum DisplayListItem {
    /// Schema/group header (not selectable)
    Header(String),
    /// Node reference by index in the filtered nodes list
    Node(usize),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Home Component
// ═══════════════════════════════════════════════════════════════════════════════

/// Home component for the main application view
/// Owns navigation state and handles node list interactions
pub struct HomeComponent {
    /// Current active tab
    pub active_tab: Tab,

    /// List selection state
    pub list_state: ListState,

    /// Search query string
    pub search_query: String,

    /// Whether search mode is active
    pub search_mode: bool,

    /// Whether lineage panel is visible
    pub show_lineage: bool,

    /// Whether documentation panel is visible
    pub show_documentation: bool,

    /// Code view mode (compiled vs original)
    pub code_view_mode: CodeViewMode,

    /// Selected nodes for bulk operations (by unique_id)
    pub selected_nodes: HashSet<String>,

    /// Current tag filter (empty means no filter)
    pub tag_filter: String,
}

impl Default for HomeComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeComponent {
    pub fn new() -> Self {
        Self {
            active_tab: Tab::Models,
            list_state: ListState::default(),
            search_query: String::new(),
            search_mode: false,
            show_lineage: false,
            show_documentation: false,
            code_view_mode: CodeViewMode::Original,
            selected_nodes: HashSet::new(),
            tag_filter: String::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Node Filtering & Display
    // ─────────────────────────────────────────────────────────────────────────

    /// Get nodes filtered by the active tab, search query, and tag filter
    pub fn get_filtered_nodes<'a>(&self, all_nodes: &'a [Node]) -> Vec<&'a Node> {
        let mut nodes: Vec<&Node> = match self.active_tab.resource_type() {
            Some(resource_type) => all_nodes
                .iter()
                .filter(|node| node.resource_type == resource_type)
                .collect(),
            None => vec![],
        };

        // Apply search query filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            nodes.retain(|node| node.name.to_lowercase().contains(&query));
        }

        // Apply tag filter
        if !self.tag_filter.is_empty() {
            let tag = self.tag_filter.to_lowercase();
            nodes.retain(|node| {
                node.config
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&tag))
            });
        }

        nodes
    }

    /// Get nodes grouped for display
    /// - Models/Seeds: grouped by schema
    /// - Tests: grouped by the model being tested
    pub fn get_nodes_grouped<'a>(&self, all_nodes: &'a [Node]) -> Vec<(String, Vec<&'a Node>)> {
        let nodes = self.get_filtered_nodes(all_nodes);
        let mut grouped: BTreeMap<String, Vec<&Node>> = BTreeMap::new();

        for node in nodes {
            let group_key = if self.active_tab == Tab::Tests {
                // Group tests by the model they test
                Self::get_test_model_name(node)
            } else {
                // Group models/seeds by schema
                if node.schema.is_empty() {
                    "default".to_string()
                } else {
                    node.schema.clone()
                }
            };
            grouped.entry(group_key).or_default().push(node);
        }

        grouped.into_iter().collect()
    }

    /// Extract the model name that a test is testing
    fn get_test_model_name(test_node: &Node) -> String {
        // Tests depend on the model they test via depends_on.nodes
        if let Some(dep) = test_node.depends_on.nodes.first() {
            if dep.starts_with("model.") {
                if let Some(model_name) = dep.split('.').next_back() {
                    return model_name.to_string();
                }
            }
            return dep.clone();
        }
        "unknown".to_string()
    }

    /// Build display items list
    /// Returns the items and a list of selectable indices
    pub fn build_display_list(&self, all_nodes: &[Node]) -> (Vec<DisplayListItem>, Vec<usize>) {
        let grouped = self.get_nodes_grouped(all_nodes);
        let mut items = Vec::new();
        let mut selectable_indices = Vec::new();
        let mut node_index = 0;

        for (group_name, nodes) in grouped {
            // Add group header
            items.push(DisplayListItem::Header(group_name));

            // Add nodes under this group
            for _node in nodes {
                selectable_indices.push(items.len());
                items.push(DisplayListItem::Node(node_index));
                node_index += 1;
            }
        }

        (items, selectable_indices)
    }

    /// Get node by its index in the flattened filtered list
    pub fn get_node_by_index<'a>(&self, all_nodes: &'a [Node], index: usize) -> Option<&'a Node> {
        self.get_filtered_nodes(all_nodes).get(index).copied()
    }

    /// Get the currently selected node
    pub fn get_selected_node<'a>(&self, all_nodes: &'a [Node]) -> Option<&'a Node> {
        let (display_items, _) = self.build_display_list(all_nodes);
        let display_idx = self.list_state.selected()?;

        match display_items.get(display_idx)? {
            DisplayListItem::Node(node_idx) => self.get_node_by_index(all_nodes, *node_idx),
            DisplayListItem::Header(_) => None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Navigation
    // ─────────────────────────────────────────────────────────────────────────

    /// Switch to the next tab
    pub fn next_tab(&mut self, all_nodes: &[Node]) {
        let tabs = Tab::all();
        let current_index = tabs.iter().position(|t| *t == self.active_tab).unwrap();
        let next_index = (current_index + 1) % tabs.len();
        self.active_tab = tabs[next_index];
        self.select_first(all_nodes);
    }

    /// Switch to the previous tab
    pub fn previous_tab(&mut self, all_nodes: &[Node]) {
        let tabs = Tab::all();
        let current_index = tabs.iter().position(|t| *t == self.active_tab).unwrap();
        let prev_index = if current_index == 0 {
            tabs.len() - 1
        } else {
            current_index - 1
        };
        self.active_tab = tabs[prev_index];
        self.select_first(all_nodes);
    }

    /// Select next item in the list (skipping headers)
    pub fn next(&mut self, all_nodes: &[Node]) {
        let (_, selectable_indices) = self.build_display_list(all_nodes);
        if selectable_indices.is_empty() {
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);

        let next_idx = selectable_indices
            .iter()
            .find(|&&idx| idx > current)
            .copied()
            .unwrap_or(selectable_indices[0]); // Wrap to first

        self.list_state.select(Some(next_idx));
    }

    /// Select previous item in the list (skipping headers)
    pub fn previous(&mut self, all_nodes: &[Node]) {
        let (_, selectable_indices) = self.build_display_list(all_nodes);
        if selectable_indices.is_empty() {
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);

        let prev_idx = selectable_indices
            .iter()
            .rev()
            .find(|&&idx| idx < current)
            .copied()
            .unwrap_or(*selectable_indices.last().unwrap()); // Wrap to last

        self.list_state.select(Some(prev_idx));
    }

    /// Select the first selectable item
    pub fn select_first(&mut self, all_nodes: &[Node]) {
        let (_, selectable_indices) = self.build_display_list(all_nodes);
        if let Some(&first_idx) = selectable_indices.first() {
            self.list_state.select(Some(first_idx));
        } else {
            self.list_state.select(None);
        }
    }

    /// Select the last selectable item
    pub fn select_last(&mut self, all_nodes: &[Node]) {
        let (_, selectable_indices) = self.build_display_list(all_nodes);
        if let Some(&last_idx) = selectable_indices.last() {
            self.list_state.select(Some(last_idx));
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Toggles
    // ─────────────────────────────────────────────────────────────────────────

    /// Toggle code view mode between compiled and original
    pub fn toggle_code_view_mode(&mut self) {
        self.code_view_mode = match self.code_view_mode {
            CodeViewMode::Compiled => CodeViewMode::Original,
            CodeViewMode::Original => CodeViewMode::Compiled,
        };
    }

    /// Toggle lineage panel visibility
    pub fn toggle_lineage(&mut self) {
        self.show_lineage = !self.show_lineage;
    }

    /// Toggle documentation panel visibility
    pub fn toggle_documentation(&mut self) {
        self.show_documentation = !self.show_documentation;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Search
    // ─────────────────────────────────────────────────────────────────────────

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
    }

    /// Exit search mode
    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }

    /// Add character to search query
    pub fn search_input(&mut self, c: char, all_nodes: &[Node]) {
        self.search_query.push(c);
        self.select_first(all_nodes);
    }

    /// Remove last character from search query
    pub fn search_backspace(&mut self, all_nodes: &[Node]) {
        self.search_query.pop();
        self.select_first(all_nodes);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Selection
    // ─────────────────────────────────────────────────────────────────────────

    /// Toggle selection of the currently focused node
    pub fn toggle_selection(&mut self, all_nodes: &[Node]) {
        if let Some(node) = self.get_selected_node(all_nodes) {
            let unique_id = node.unique_id.clone();
            if self.selected_nodes.contains(&unique_id) {
                self.selected_nodes.remove(&unique_id);
            } else {
                self.selected_nodes.insert(unique_id);
            }
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    /// Select all visible nodes
    pub fn select_all(&mut self, all_nodes: &[Node]) {
        let filtered = self.get_filtered_nodes(all_nodes);
        for node in filtered {
            self.selected_nodes.insert(node.unique_id.clone());
        }
    }

    /// Check if a node is selected
    pub fn is_node_selected(&self, unique_id: &str) -> bool {
        self.selected_nodes.contains(unique_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Tag Filter
    // ─────────────────────────────────────────────────────────────────────────

    /// Set the tag filter
    pub fn set_tag_filter(&mut self, tag: String, all_nodes: &[Node]) {
        self.tag_filter = tag;
        self.select_first(all_nodes);
    }

    /// Clear the tag filter
    pub fn clear_tag_filter(&mut self, all_nodes: &[Node]) {
        self.tag_filter.clear();
        self.select_first(all_nodes);
    }

    /// Get all unique tags from all nodes
    pub fn get_all_tags(all_nodes: &[Node]) -> Vec<String> {
        let mut tags: HashSet<String> = HashSet::new();
        for node in all_nodes {
            for tag in &node.config.tags {
                tags.insert(tag.clone());
            }
        }
        let mut result: Vec<String> = tags.into_iter().collect();
        result.sort();
        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Component Implementation
// ═══════════════════════════════════════════════════════════════════════════════

impl Component for HomeComponent {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => Some(Action::NextItem),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::PrevItem),
            KeyCode::Tab => Some(Action::NextTab),
            KeyCode::BackTab => Some(Action::PrevTab),
            KeyCode::Char('g') => Some(Action::FirstItem),
            KeyCode::Char('G') => Some(Action::LastItem),

            // Scrolling (with Ctrl for detail panel)
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

            // View toggles
            KeyCode::Char('c') => Some(Action::ToggleCodeView),
            KeyCode::Char('l') => Some(Action::ToggleLineage),
            KeyCode::Char('d') => Some(Action::ToggleDocumentation),

            // Modals
            KeyCode::Char('q') => Some(Action::OpenQuitDialog),
            KeyCode::Char('r') | KeyCode::Enter => Some(Action::OpenRunOptions),
            KeyCode::Char('h') => Some(Action::OpenHistory),
            KeyCode::Char('i') => Some(Action::OpenProjectInfo),
            KeyCode::Char('t') => Some(Action::OpenTargetSelector),

            // Project Management
            KeyCode::Char('R') => Some(Action::RefreshManifest),

            // Search
            KeyCode::Char('/') => Some(Action::EnterSearchMode),

            // Selection
            KeyCode::Char(' ') => Some(Action::ToggleNodeSelection),
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::SelectAllNodes)
            }
            KeyCode::Esc if !self.selected_nodes.is_empty() => Some(Action::ClearSelection),

            // Tag filter
            KeyCode::Char('f') => Some(Action::OpenTagFilter),

            // Editor
            KeyCode::Char('e') => Some(Action::OpenEditor),

            // Sample data preview
            KeyCode::Char('p') => Some(Action::OpenSampleData),

            // Help
            KeyCode::Char('?') => Some(Action::OpenHelp),

            // Git operations (Shift+key for git commands)
            KeyCode::Char('D') => Some(Action::OpenGitDiff),
            KeyCode::Char('A') => Some(Action::GitStageFile),  // Add/Stage
            KeyCode::Char('K') => Some(Action::OpenGitCommit), // Commit (K for commit)
            KeyCode::Char('L') => Some(Action::OpenGitLog),    // Log

            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, _action: Action) -> Result<Option<Action>> {
        // Updates are handled by App which has access to all_nodes
        // App calls the navigation methods directly
        Ok(None)
    }

    fn draw(&mut self, _frame: &mut Frame, _area: Rect) -> Result<()> {
        // Drawing is done through draw_home_screen which takes full context
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Context needed for rendering the home screen
pub struct HomeRenderContext<'a> {
    pub all_nodes: &'a [Node],
    pub project_name: Option<&'a str>,
    pub lineage_graph: Option<&'a crate::model::lineage::LineageGraph>,
    pub error: Option<&'a str>,
    pub status_message: Option<&'a str>,
    pub git_branch: Option<&'a str>,
    pub git_is_dirty: bool,
    pub git_file_statuses: &'a std::collections::HashMap<String, crate::services::GitFileStatus>,
}

/// Draw the home screen
pub fn draw_home_screen(
    frame: &mut Frame,
    area: Rect,
    home: &mut HomeComponent,
    detail: &mut crate::components::DetailComponent,
    lineage: &mut crate::components::LineageComponent,
    documentation: &mut crate::components::DocumentationComponent,
    ctx: &HomeRenderContext,
) -> Result<()> {
    let layout = calculate_main_layout(area, true, home.show_lineage, home.show_documentation);

    // Render each section
    render_tabs(frame, layout.tabs, home);
    render_info_box(frame, layout.info, home, ctx.all_nodes);
    render_node_list(frame, layout.list, home, ctx.all_nodes, ctx.git_file_statuses);

    // Update and render detail panel
    let node = home.get_selected_node(ctx.all_nodes).cloned();
    detail.set_node(node.as_ref(), home.code_view_mode);
    detail.draw(frame, layout.detail)?;

    // Update and render lineage panel if visible
    if let Some(lineage_area) = layout.lineage {
        let node_unique_id = node.as_ref().map(|n| n.unique_id.as_str());
        lineage.set_node(node_unique_id, ctx.lineage_graph);
        lineage.draw(frame, lineage_area)?;
    }

    // Update and render documentation panel if visible
    if let Some(doc_area) = layout.documentation {
        documentation.set_node(node.as_ref());
        documentation.draw(frame, doc_area)?;
    }

    if let Some(status_area) = layout.status {
        render_status_bar(frame, status_area, home, ctx);
    }
    render_help_bar(frame, layout.help, home);

    Ok(())
}

fn render_info_box(frame: &mut Frame, area: Rect, home: &HomeComponent, all_nodes: &[Node]) {
    let mut lines = vec![];

    if let Some(node) = home.get_selected_node(all_nodes) {
        // Node name with icon
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", node.icon()),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                node.name.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // File path
        if let Some(ref path) = node.original_file_path {
            lines.push(Line::from(Span::styled(
                path.clone(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Schema
        if !node.schema.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("schema: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&node.schema, Style::default().fg(Color::Cyan)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_tabs(frame: &mut Frame, area: Rect, home: &HomeComponent) {
    let all_tabs = Tab::all();
    let titles: Vec<&str> = all_tabs.iter().map(|t| t.name()).collect();
    let selected = all_tabs
        .iter()
        .position(|t| *t == home.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn render_node_list(
    frame: &mut Frame,
    area: Rect,
    home: &mut HomeComponent,
    all_nodes: &[Node],
    git_file_statuses: &std::collections::HashMap<String, crate::services::GitFileStatus>,
) {
    let nodes = home.get_filtered_nodes(all_nodes);
    let (display_items, _) = home.build_display_list(all_nodes);

    let items: Vec<ListItem> = display_items
        .iter()
        .map(|item| match item {
            DisplayListItem::Header(schema) => {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("── {} ", schema),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "──────────────────────",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            }
            DisplayListItem::Node(node_idx) => {
                if let Some(node) = home.get_node_by_index(all_nodes, *node_idx) {
                    let icon = node.icon();
                    let is_selected = home.is_node_selected(&node.unique_id);
                    let selection_marker = if is_selected { "● " } else { "  " };
                    let selection_style = if is_selected {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };
                    let name_style = if is_selected {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    // Get git status indicator for this file
                    let git_indicator = node
                        .original_file_path
                        .as_ref()
                        .and_then(|path| git_file_statuses.get(path))
                        .map(|status| {
                            let (indicator, color) = match status {
                                crate::services::GitFileStatus::Modified => ("M", Color::Yellow),
                                crate::services::GitFileStatus::Staged => ("S", Color::Green),
                                crate::services::GitFileStatus::StagedModified => ("*", Color::Cyan),
                                crate::services::GitFileStatus::Untracked => ("?", Color::Red),
                                crate::services::GitFileStatus::Deleted => ("D", Color::Red),
                                crate::services::GitFileStatus::StagedDeleted => ("X", Color::Red),
                                crate::services::GitFileStatus::Renamed => ("R", Color::Magenta),
                                crate::services::GitFileStatus::Copied => ("C", Color::Magenta),
                                _ => (" ", Color::DarkGray),
                            };
                            (indicator, color)
                        })
                        .unwrap_or((" ", Color::DarkGray));

                    let spans = vec![
                        Span::styled(selection_marker, selection_style),
                        Span::styled(
                            format!("{} ", git_indicator.0),
                            Style::default().fg(git_indicator.1).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!("{} ", icon), Style::default().fg(Color::Yellow)),
                        Span::styled(node.display_name(), name_style),
                    ];

                    ListItem::new(Line::from(spans))
                } else {
                    ListItem::new(Line::from(Span::raw("")))
                }
            }
        })
        .collect();

    // Build title with selection count and tag filter info
    let mut title = format!(" {} ({}) ", home.active_tab.name(), nodes.len());
    if !home.selected_nodes.is_empty() {
        title = format!(
            " {} ({}) [{}✓] ",
            home.active_tab.name(),
            nodes.len(),
            home.selected_nodes.len()
        );
    }
    if !home.tag_filter.is_empty() {
        title = format!("{} [tag:{}] ", title.trim_end(), home.tag_filter);
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut home.list_state);
}

fn render_status_bar(frame: &mut Frame, area: Rect, home: &HomeComponent, ctx: &HomeRenderContext) {
    let mut spans = vec![];

    // Project name
    if let Some(project_name) = ctx.project_name {
        spans.push(Span::styled(
            format!(" {} ", project_name),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    // Git branch indicator
    if let Some(branch) = ctx.git_branch {
        let dirty_indicator = if ctx.git_is_dirty { "*" } else { "" };
        spans.push(Span::styled(
            format!(" {}{} ", branch, dirty_indicator),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    // Selected node info
    if let Some(node) = home.get_selected_node(ctx.all_nodes) {
        spans.push(Span::styled(
            format!("{} ", node.icon()),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

        if let Some(ref path) = node.original_file_path {
            spans.push(Span::styled(
                format!(" ({})", path),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    // Error message if present
    if let Some(error) = ctx.error {
        spans.clear();
        spans.push(Span::styled(
            format!(" Error: {} ", error),
            Style::default().fg(Color::Red),
        ));
    }

    // Status message if present
    if let Some(status) = ctx.status_message {
        spans.push(Span::styled(
            format!(" {} ", status),
            Style::default().fg(Color::Yellow),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

fn render_help_bar(frame: &mut Frame, area: Rect, home: &HomeComponent) {
    let help_spans = if home.search_mode {
        vec![
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Cancel  "),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Confirm  "),
            Span::styled(
                format!("Search: {}", home.search_query),
                Style::default().fg(Color::Cyan),
            ),
        ]
    } else if !home.selected_nodes.is_empty() {
        // Selection mode help
        vec![
            Span::styled(
                " Space ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Toggle  "),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Clear  "),
            Span::styled(
                " r ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Run Selected  "),
            Span::styled(
                format!("{} selected", home.selected_nodes.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]
    } else {
        vec![
            Span::styled(
                " q ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit "),
            Span::styled(
                " r ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Run "),
            Span::styled(
                " e ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Edit "),
            Span::styled(
                " p ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Preview "),
            Span::styled(
                " / ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Search "),
            Span::styled(
                " h ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("History "),
            Span::styled(
                " ? ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Help "),
            Span::styled(
                "│ Git:",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                " D ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Diff "),
            Span::styled(
                " A ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Stage "),
            Span::styled(
                " K ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Commit "),
            Span::styled(
                " L ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Log"),
        ]
    };

    let paragraph = Paragraph::new(Line::from(help_spans))
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(paragraph, area);
}
