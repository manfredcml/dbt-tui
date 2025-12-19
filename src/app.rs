//! Root application component
//!
//! The App struct implements the Component trait, acting as the root component
//! that delegates event handling and rendering to child components.
//! App is intentionally lean - it coordinates between components but
//! does not contain business logic itself.

use crate::action::Action;
use crate::component::Component;
use crate::components::{
    draw_home_screen, DetailComponent, DocumentationComponent, HelpDialog, HistoryDialog,
    HomeComponent, HomeRenderContext, LineageComponent, ProjectInfoDialog, QuitDialog,
    RunOptionsDialog, RunOutputDialog, SampleDataDialog, SetupComponent, SplashComponent,
    TagFilterDialog, TargetSelectorDialog,
};
use crate::config::Config;
use crate::model::domain::DomainState;
use crate::model::history::{RunHistory, RunHistoryEntry};
use crate::model::lineage::LineageGraph;
use crate::model::modal::{Modal, ModalStack};
use crate::model::run::{DbtCommand, RunFlags, RunOutput, RunSelectMode, RunStatus};
use crate::model::sample_data::SampleDataOutput;
use crate::model::ui::AppMode;
use crate::services::{self, JobRunner};
use anyhow::Result;
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════════
// Error Message Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate user-friendly error message for missing manifest
fn manifest_not_found_error(manifest_path: &std::path::Path, project_path: &std::path::Path) -> String {
    let dbt_project_exists = project_path.join("dbt_project.yml").exists();
    let target_dir_exists = project_path.join("target").exists();

    let mut msg = format!(
        "manifest.json not found at:\n  {}\n\n",
        manifest_path.display()
    );

    if !dbt_project_exists {
        msg.push_str("This does not appear to be a valid dbt project.\n");
        msg.push_str("Please check your project path configuration.\n\n");
        msg.push_str("Expected to find: dbt_project.yml");
    } else if !target_dir_exists {
        msg.push_str("The 'target' directory does not exist.\n\n");
        msg.push_str("Press 'c' to run 'dbt compile' and generate manifest.json\n");
        msg.push_str("Or press 'e' to edit project settings");
    } else {
        msg.push_str("The 'target' directory exists but manifest.json is missing.\n\n");
        msg.push_str("This can happen if:\n");
        msg.push_str("  1. dbt compile has not been run yet\n");
        msg.push_str("  2. The last compilation failed\n");
        msg.push_str("  3. The target directory was cleaned\n\n");
        msg.push_str("Press 'c' to run 'dbt compile' and generate manifest.json\n");
        msg.push_str("Or press 'e' to edit project settings");
    }

    msg
}

/// Generate error message for manifest parsing errors
fn manifest_parse_error(error: &str) -> String {
    format!(
        "Failed to parse manifest.json\n\n\
         Error: {}\n\n\
         This may indicate:\n\
         • The manifest.json file is corrupted\n\
         • dbt version incompatibility\n\
         • The file is being written to\n\n\
         Try running 'dbt compile' again.",
        error
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// App Struct
// ═══════════════════════════════════════════════════════════════════════════════

/// Main application state - coordinates between components
pub struct App {
    /// Current application mode
    pub mode: AppMode,

    /// Next mode to transition to after splash
    pub next_mode_after_splash: AppMode,

    /// Domain state (business data)
    pub domain: DomainState,

    /// Modal overlay stack
    pub modals: ModalStack,

    /// Background job runner
    pub job_runner: JobRunner,

    /// Background job runner for sample data queries
    pub sample_data_runner: JobRunner,

    /// Flag to indicate the app should quit
    pub should_quit: bool,

    /// Error message to display
    pub error: Option<String>,

    /// Status message to display
    pub status_message: Option<String>,

    /// Git branch name
    pub git_branch: Option<String>,

    /// Whether the git repo has uncommitted changes
    pub git_is_dirty: bool,

    /// Git file statuses by relative path
    pub git_file_statuses: std::collections::HashMap<String, services::GitFileStatus>,

    /// Pending external editor file path (set by OpenEditor action, handled by main loop)
    pub pending_editor_file: Option<String>,

    // ─────────────────────────────────────────────────────────────────────────
    // Child Components
    // ─────────────────────────────────────────────────────────────────────────
    pub splash: SplashComponent,
    pub home: HomeComponent,
    pub detail: DetailComponent,
    pub lineage: LineageComponent,
    pub documentation: DocumentationComponent,
    pub quit_dialog: QuitDialog,
    pub run_options_dialog: RunOptionsDialog,
    pub history_dialog: HistoryDialog,
    pub run_output_dialog: RunOutputDialog,
    pub project_info_dialog: ProjectInfoDialog,
    pub setup: SetupComponent,
    pub target_selector: TargetSelectorDialog,
    pub tag_filter_dialog: TagFilterDialog,
    pub sample_data_dialog: SampleDataDialog,
    pub help_dialog: HelpDialog,

    /// Current config (for saving target changes)
    pub config: Option<Config>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// App Implementation
// ═══════════════════════════════════════════════════════════════════════════════

impl App {
    /// Create a new App instance
    pub fn new() -> App {
        // Try to load existing config
        if let Some(config) = Config::load() {
            let mut app = Self::create_app(AppMode::Running);

            app.domain.dbt_binary_path = config.dbt_binary_path.clone();
            let project_path = PathBuf::from(&config.project_path);
            app.domain.project_path = Some(project_path.clone());

            let manifest_path = project_path.join("target").join("manifest.json");

            if !manifest_path.exists() {
                app.error = Some(manifest_not_found_error(&manifest_path, &project_path));
                return app;
            }

            match services::load_manifest(&manifest_path) {
                Ok(manifest) => {
                    app.domain.all_nodes = services::filter_nodes(&manifest);

                    let root_path_str = project_path.to_string_lossy().to_string();
                    for node in &mut app.domain.all_nodes {
                        node.root_path = Some(root_path_str.clone());
                    }

                    app.domain.lineage_graph = Some(LineageGraph::build(&app.domain.all_nodes));

                    if !app.domain.all_nodes.is_empty() {
                        app.home.select_first(&app.domain.all_nodes);
                    } else {
                        app.error = Some("No models, tests, or seeds found in manifest".to_string());
                    }

                    app.domain.project_info = Some(services::get_project_info(
                        &app.domain.dbt_binary_path,
                        &app.domain.project_path,
                        &app.domain.all_nodes,
                    ));
                }
                Err(e) => {
                    app.error = Some(manifest_parse_error(&e.to_string()));
                }
            }

            // Load git status
            app.refresh_git_status();

            app.domain.run_history = RunHistory::load();
            app
        } else {
            // No config exists, show splash then setup screen
            let mut app = Self::create_app(AppMode::Setup);
            app.domain.run_history = RunHistory::load();
            app
        }
    }

    /// Refresh git status for the current project
    pub fn refresh_git_status(&mut self) {
        if let Some(project_path) = &self.domain.project_path {
            if services::is_git_repo(project_path) {
                if let Ok(status) = services::get_status(project_path) {
                    self.git_branch = Some(status.branch);
                    self.git_is_dirty = status.is_dirty;
                    self.git_file_statuses = status.files;
                }
            } else {
                self.git_branch = None;
                self.git_is_dirty = false;
                self.git_file_statuses.clear();
            }
        }
    }

    fn create_app(next_mode: AppMode) -> App {
        App {
            mode: AppMode::Splash,
            next_mode_after_splash: next_mode,
            domain: DomainState::new(),
            modals: ModalStack::new(),
            job_runner: JobRunner::new(),
            sample_data_runner: JobRunner::new(),
            should_quit: false,
            error: None,
            status_message: None,
            git_branch: None,
            git_is_dirty: false,
            git_file_statuses: std::collections::HashMap::new(),
            pending_editor_file: None,
            // Components
            splash: SplashComponent::new(),
            home: HomeComponent::new(),
            detail: DetailComponent::new(),
            lineage: LineageComponent::new(),
            documentation: DocumentationComponent::new(),
            quit_dialog: QuitDialog,
            run_options_dialog: RunOptionsDialog::default(),
            history_dialog: HistoryDialog::default(),
            run_output_dialog: RunOutputDialog::default(),
            project_info_dialog: ProjectInfoDialog::new(),
            setup: SetupComponent::new(),
            target_selector: TargetSelectorDialog::new(),
            tag_filter_dialog: TagFilterDialog::new(),
            sample_data_dialog: SampleDataDialog::default(),
            help_dialog: HelpDialog::default(),
            config: None,
        }
    }

    /// Get the currently selected node (delegates to home component)
    pub fn get_selected_node(&self) -> Option<&crate::model::node::Node> {
        self.home.get_selected_node(&self.domain.all_nodes)
    }

    /// Execute a dbt command with the given options
    /// Handles both single node and bulk selection
    fn execute_dbt_command(&mut self, command: DbtCommand, mode: RunSelectMode, flags: &RunFlags) {
        let project_path = match &self.domain.project_path {
            Some(p) => p.clone(),
            None => return,
        };

        // For commands that don't require selection (like deps), execute directly
        if !command.requires_selection() {
            let (cmd, display_cmd) = services::build_dbt_command(
                &self.domain.dbt_binary_path,
                &project_path,
                command,
                None,
                flags,
            );
            self.domain.run_output = Some(self.job_runner.spawn(cmd));
            if let Some(ref mut output) = self.domain.run_output {
                output.command = display_cmd;
            }
            self.modals.push(Modal::RunOutput);
            return;
        }

        // Build selector based on bulk selection or single node
        let selector = if !self.home.selected_nodes.is_empty() {
            // Bulk selection: build selector from all selected nodes
            let selected_nodes: Vec<_> = self.domain.all_nodes
                .iter()
                .filter(|n| self.home.selected_nodes.contains(&n.unique_id))
                .collect();

            if selected_nodes.is_empty() {
                return;
            }

            // Build selector with all node names
            let names: Vec<String> = selected_nodes
                .iter()
                .map(|n| mode.selector(&n.name))
                .collect();

            // Clear selection after running
            self.home.clear_selection();

            names.join(" ")
        } else {
            // Single node selection
            let node = match self.get_selected_node() {
                Some(n) => n.clone(),
                None => return,
            };
            mode.selector(&node.name)
        };

        let (cmd, display_cmd) = services::build_dbt_command(
            &self.domain.dbt_binary_path,
            &project_path,
            command,
            Some(&selector),
            flags,
        );

        self.domain.run_output = Some(self.job_runner.spawn(cmd));
        if let Some(ref mut output) = self.domain.run_output {
            output.command = display_cmd;
        }
        self.modals.push(Modal::RunOutput);
    }
    /// Save run to history when complete
    fn save_to_history(&mut self) {
        if let Some(ref run_output) = self.domain.run_output {
            if run_output.status != RunStatus::Running {
                let duration = self.job_runner.start_instant()
                    .map(|i| i.elapsed().as_secs_f64())
                    .unwrap_or(0.0);

                let entry = RunHistoryEntry {
                    timestamp: Local::now(),
                    command: run_output.command.clone(),
                    status: run_output.status,
                    output: run_output.output.clone(),
                    duration_secs: duration,
                };

                self.domain.run_history.insert(0, entry);
                if self.domain.run_history.len() > 100 {
                    self.domain.run_history.truncate(100);
                }
                let _ = RunHistory::save(&self.domain.run_history);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Component Implementation
// ═══════════════════════════════════════════════════════════════════════════════

impl Component for App {
    fn init(&mut self) -> Result<()> {
        self.splash.init()?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match self.mode {
            AppMode::Splash => self.splash.handle_key_event(key),
            AppMode::Setup => self.setup.handle_key_event(key),
            AppMode::Running => {
                // If there's an error (like missing manifest), handle special keys
                if self.error.is_some() && self.modals.is_empty() {
                    return self.handle_error_key_event(key);
                }

                if let Some(modal) = self.modals.top().cloned() {
                    self.handle_modal_key_event(&modal, key)
                } else if self.home.search_mode {
                    self.handle_search_key_event(key)
                } else {
                    self.home.handle_key_event(key)
                }
            }
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            // ─────────────────────────────────────────────────────────────────
            // App Lifecycle
            // ─────────────────────────────────────────────────────────────────
            Action::Tick => {
                if self.mode == AppMode::Splash && self.splash.is_complete() {
                    return Ok(Some(Action::SplashComplete));
                }
                // Poll background jobs
                if let Some(ref mut run_output) = self.domain.run_output {
                    self.job_runner.poll(run_output);
                    // Compute layers based on manifest dependency graph
                    run_output.compute_layers(&self.domain.all_nodes);
                }
                // Poll sample data jobs
                if let Some(ref mut sample_output) = self.domain.sample_data_output {
                    if sample_output.status == RunStatus::Running {
                        // Create a temporary RunOutput to poll
                        let mut temp_output = RunOutput::new(String::new());
                        temp_output.output = sample_output.raw_output.clone();
                        temp_output.status = sample_output.status;

                        self.sample_data_runner.poll(&mut temp_output);

                        // Copy back the results
                        sample_output.raw_output = temp_output.output;
                        sample_output.status = temp_output.status;

                        // Parse when complete
                        if sample_output.status != RunStatus::Running {
                            sample_output.parse_output();
                            if sample_output.status == RunStatus::Failed
                                && sample_output.headers.is_empty()
                            {
                                sample_output.error_message =
                                    Some("dbt show command failed".to_string());
                            }
                        }
                    }
                }
            }
            Action::SplashComplete => {
                self.mode = self.next_mode_after_splash;
            }
            Action::ForceQuit => {
                self.should_quit = true;
            }
            Action::Resize(_, _) => {}

            // ─────────────────────────────────────────────────────────────────
            // Navigation (delegate to HomeComponent)
            // ─────────────────────────────────────────────────────────────────
            Action::NextItem => self.home.next(&self.domain.all_nodes),
            Action::PrevItem => self.home.previous(&self.domain.all_nodes),
            Action::NextTab => self.home.next_tab(&self.domain.all_nodes),
            Action::PrevTab => self.home.previous_tab(&self.domain.all_nodes),
            Action::FirstItem => self.home.select_first(&self.domain.all_nodes),
            Action::LastItem => self.home.select_last(&self.domain.all_nodes),

            // ─────────────────────────────────────────────────────────────────
            // Scrolling (delegate to DetailComponent)
            // ─────────────────────────────────────────────────────────────────
            Action::ScrollUp | Action::ScrollDown | Action::PageUp | Action::PageDown => {
                self.detail.update(action)?;
            }

            // ─────────────────────────────────────────────────────────────────
            // View Toggles (delegate to HomeComponent)
            // ─────────────────────────────────────────────────────────────────
            Action::ToggleCodeView => self.home.toggle_code_view_mode(),
            Action::ToggleLineage => self.home.toggle_lineage(),
            Action::ToggleDocumentation => self.home.toggle_documentation(),
            Action::ToggleOutputView => {
                if let Some(ref mut run_output) = self.domain.run_output {
                    run_output.toggle_view_mode();
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // Modals
            // ─────────────────────────────────────────────────────────────────
            Action::OpenQuitDialog => {
                self.modals.push(Modal::QuitConfirm);
            }
            Action::OpenRunOptions => {
                self.run_options_dialog.reset();
                self.modals.push(Modal::RunOptions { selected_index: 0 });
            }
            Action::OpenProjectInfo => {
                if self.modals.top() == Some(&Modal::ProjectInfo) {
                    self.modals.pop();
                } else {
                    self.modals.push(Modal::ProjectInfo);
                }
            }
            Action::OpenHistory => {
                self.history_dialog.selected_index = 0;
                self.history_dialog.detail_scroll = 0;
                if matches!(self.modals.top(), Some(Modal::History { .. })) {
                    self.modals.pop();
                } else {
                    self.modals.push(Modal::History {
                        selected_index: 0,
                        detail_scroll: 0,
                    });
                }
            }
            Action::OpenTargetSelector => {
                // Get current target from config
                let current_target = if let Some(ref config) = self.config {
                    config.target.clone()
                } else if let Some(config) = Config::load() {
                    let target = config.target.clone();
                    self.config = Some(config);
                    target
                } else {
                    "dev".to_string()
                };

                // Try to parse profiles.yml
                if let Some(ref project_path) = self.domain.project_path {
                    if let Some(profiles_info) = services::parse_profiles(project_path) {
                        self.target_selector
                            .set_targets_with_info(&current_target, profiles_info.targets);
                    } else {
                        self.target_selector.set_no_profiles(&current_target);
                    }
                } else {
                    self.target_selector.set_no_profiles(&current_target);
                }

                self.modals.push(Modal::TargetSelector {
                    selected_index: self.target_selector.selected_index,
                });
            }
            Action::OpenRunOutput => {
                // View selected history entry as run output
                if let Some(entry) = self.domain.run_history.get(self.history_dialog.selected_index) {
                    let mut run_output = RunOutput::new(entry.command.clone());
                    run_output.status = entry.status;
                    run_output.output = entry.output.clone();
                    // Parse output lines to populate model_runs
                    for line in entry.output.lines() {
                        run_output.parse_output_line(line);
                    }
                    // Compute layers based on manifest
                    run_output.compute_layers(&self.domain.all_nodes);
                    self.domain.run_output = Some(run_output);
                    self.modals.pop();
                    self.modals.push(Modal::RunOutput);
                }
            }
            Action::CloseModal => {
                if matches!(self.modals.top(), Some(Modal::RunOutput)) {
                    // Check if this was a compile command that succeeded
                    let should_refresh = self.domain.run_output.as_ref().is_some_and(|o| {
                        o.status == RunStatus::Success && o.command.contains("compile")
                    });

                    self.save_to_history();
                    self.domain.run_output = None;
                    self.job_runner.clear();

                    // Auto-refresh manifest after successful compile
                    if should_refresh {
                        self.refresh_manifest();
                        self.refresh_git_status();
                    }
                }
                if matches!(self.modals.top(), Some(Modal::SampleData { .. })) {
                    self.domain.sample_data_output = None;
                    self.sample_data_runner.clear();
                }
                self.modals.pop();
            }
            Action::ConfirmModal => {
                if let Some(modal) = self.modals.top().cloned() {
                    match modal {
                        Modal::QuitConfirm => {
                            self.should_quit = true;
                        }
                        Modal::RunOptions { .. } => {
                            // Get all options from the dialog
                            let command = self.run_options_dialog.get_command();
                            let mode = self.run_options_dialog.get_mode();
                            let flags = self.run_options_dialog.get_flags();
                            self.modals.pop();
                            // Execute the command directly
                            self.execute_dbt_command(command, mode, &flags);
                        }
                        Modal::TargetSelector { .. } => {
                            let selected_target = self.target_selector.get_selected_target().to_string();
                            self.change_target(&selected_target);
                            self.modals.pop();
                        }
                        _ => {}
                    }
                }
            }
            Action::ModalUp => {
                if matches!(self.modals.top(), Some(Modal::RunOptions { .. })) {
                    // RunOptionsDialog handles its own navigation via handle_key_event
                    self.run_options_dialog.update(Action::ModalUp)?;
                } else if matches!(self.modals.top(), Some(Modal::History { .. })) {
                    self.history_dialog.update(Action::ModalUp)?;
                    if let Some(Modal::History { selected_index, .. }) = self.modals.top_mut() {
                        *selected_index = self.history_dialog.selected_index;
                    }
                } else if let Some(Modal::TargetSelector { selected_index }) = self.modals.top_mut() {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                        self.target_selector.selected_index = *selected_index;
                    }
                }
            }
            Action::ModalDown => {
                if matches!(self.modals.top(), Some(Modal::RunOptions { .. })) {
                    // RunOptionsDialog handles its own navigation via handle_key_event
                    self.run_options_dialog.update(Action::ModalDown)?;
                } else if matches!(self.modals.top(), Some(Modal::History { .. })) {
                    // Clamp before incrementing
                    let max = self.domain.run_history.len().saturating_sub(1);
                    if self.history_dialog.selected_index < max {
                        self.history_dialog.update(Action::ModalDown)?;
                    }
                    if let Some(Modal::History { selected_index, .. }) = self.modals.top_mut() {
                        *selected_index = self.history_dialog.selected_index;
                    }
                } else if let Some(Modal::TargetSelector { selected_index }) = self.modals.top_mut() {
                    let max = self.target_selector.targets.len().saturating_sub(1);
                    if *selected_index < max {
                        *selected_index += 1;
                        self.target_selector.selected_index = *selected_index;
                    }
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // Search (delegate to HomeComponent)
            // ─────────────────────────────────────────────────────────────────
            Action::EnterSearchMode => self.home.enter_search_mode(),
            Action::ExitSearchMode => self.home.exit_search_mode(),
            Action::SearchInput(c) => self.home.search_input(c, &self.domain.all_nodes),
            Action::SearchBackspace => self.home.search_backspace(&self.domain.all_nodes),

            // ─────────────────────────────────────────────────────────────────
            // Selection
            // ─────────────────────────────────────────────────────────────────
            Action::ToggleNodeSelection => {
                self.home.toggle_selection(&self.domain.all_nodes);
            }
            Action::ClearSelection => {
                self.home.clear_selection();
            }
            Action::SelectAllNodes => {
                self.home.select_all(&self.domain.all_nodes);
            }

            // ─────────────────────────────────────────────────────────────────
            // Tag Filter
            // ─────────────────────────────────────────────────────────────────
            Action::OpenTagFilter => {
                let all_tags = HomeComponent::get_all_tags(&self.domain.all_nodes);
                self.tag_filter_dialog
                    .set_tags(all_tags, &self.home.tag_filter);
                self.modals.push(Modal::TagFilter { selected_index: 0 });
            }
            Action::SetTagFilter(tag) => {
                self.home.set_tag_filter(tag, &self.domain.all_nodes);
                self.modals.pop();
            }
            Action::ClearTagFilter => {
                self.home.clear_tag_filter(&self.domain.all_nodes);
                self.modals.pop();
            }

            // ─────────────────────────────────────────────────────────────────
            // Setup
            // ─────────────────────────────────────────────────────────────────
            Action::SetupConfirm => {
                // Setup complete, load the config and switch to Running mode
                if let Some(config) = self.setup.get_config() {
                    self.load_project_from_config(config.clone());
                    self.mode = AppMode::Running;
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // Project Management
            // ─────────────────────────────────────────────────────────────────
            Action::RefreshManifest => {
                self.refresh_manifest();
                self.refresh_git_status();
            }
            Action::CompileManifest => {
                self.compile_manifest();
            }

            // ─────────────────────────────────────────────────────────────────
            // Editor Actions
            // ─────────────────────────────────────────────────────────────────
            Action::OpenEditor => {
                // Set pending_editor_file - the main loop will handle launching $EDITOR
                if let Some(node) = self.get_selected_node() {
                    if let Some(ref path) = node.original_file_path {
                        if let Some(ref root) = self.domain.project_path {
                            let full_path = root.join(path);
                            if full_path.exists() {
                                self.pending_editor_file = Some(full_path.to_string_lossy().to_string());
                            } else {
                                self.error = Some(format!("File not found: {}", full_path.display()));
                            }
                        }
                    }
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // Sample Data Preview
            // ─────────────────────────────────────────────────────────────────
            Action::OpenHelp => {
                self.help_dialog.scroll_offset = 0;
                self.modals.push(Modal::Help { scroll_offset: 0 });
            }

            Action::OpenSampleData => {
                // Clone needed data to avoid borrow issues
                let node_info = self.get_selected_node().map(|n| {
                    (n.name.clone(), n.resource_type.clone())
                });

                if let Some((node_name, resource_type)) = node_info {
                    if resource_type == "model" {
                        if let Some(ref project_path) = self.domain.project_path.clone() {
                            let (cmd, _display_cmd) = services::build_dbt_show_command(
                                &self.domain.dbt_binary_path,
                                project_path,
                                &node_name,
                                100,
                            );

                            // Spawn the command
                            let run_output = self.sample_data_runner.spawn(cmd);

                            // Create SampleDataOutput
                            let mut sample_output = SampleDataOutput::new(node_name.clone());
                            sample_output.raw_output = run_output.output.clone();
                            sample_output.status = run_output.status;

                            self.domain.sample_data_output = Some(sample_output);
                            self.modals.push(Modal::SampleData {
                                model_name: node_name,
                                scroll_offset: 0,
                            });
                        }
                    } else {
                        self.error = Some("Sample data preview is only available for models".to_string());
                    }
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // Git Actions
            // ─────────────────────────────────────────────────────────────────
            Action::OpenGitDiff => {
                if let Some(node) = self.get_selected_node() {
                    if let Some(ref path) = node.original_file_path {
                        self.modals.push(Modal::GitDiff {
                            file_path: path.clone(),
                        });
                    }
                }
            }
            Action::GitStageFile => {
                if let Some(node) = self.get_selected_node() {
                    if let Some(ref path) = node.original_file_path {
                        if let Some(ref project_path) = self.domain.project_path {
                            match services::stage_file(project_path, path) {
                                Ok(()) => {
                                    self.status_message = Some(format!("Staged: {}", path));
                                    self.refresh_git_status();
                                }
                                Err(e) => self.error = Some(e),
                            }
                        }
                    }
                }
            }
            Action::OpenGitCommit => {
                self.modals.push(Modal::GitCommit {
                    message: String::new(),
                });
            }
            Action::GitCommit(msg) => {
                if let Some(ref project_path) = self.domain.project_path {
                    match services::commit(project_path, &msg) {
                        Ok(_) => {
                            self.status_message = Some("Commit successful".to_string());
                            self.refresh_git_status();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                self.modals.pop();
            }
            Action::OpenGitLog => {
                self.modals.push(Modal::GitLog { scroll_offset: 0 });
            }
            Action::RefreshGitStatus => {
                self.refresh_git_status();
            }
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match self.mode {
            AppMode::Splash => self.splash.draw(frame, area)?,
            AppMode::Setup => self.setup.draw(frame, area)?,
            AppMode::Running => {
                // Build render context
                let ctx = HomeRenderContext {
                    all_nodes: &self.domain.all_nodes,
                    project_name: self.domain.project_info.as_ref().map(|i| i.project_name.as_str()),
                    lineage_graph: self.domain.lineage_graph.as_ref(),
                    error: self.error.as_deref(),
                    status_message: self.status_message.as_deref(),
                    git_branch: self.git_branch.as_deref(),
                    git_is_dirty: self.git_is_dirty,
                    git_file_statuses: &self.git_file_statuses,
                };

                // Draw home screen with components
                draw_home_screen(
                    frame,
                    area,
                    &mut self.home,
                    &mut self.detail,
                    &mut self.lineage,
                    &mut self.documentation,
                    &ctx,
                )?;

                // Draw modal overlay if active
                if let Some(modal) = self.modals.top().cloned() {
                    self.draw_modal(frame, area, &modal)?;
                }
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Methods
// ═══════════════════════════════════════════════════════════════════════════════

impl App {
    fn handle_modal_key_event(&mut self, modal: &Modal, key: KeyEvent) -> Result<Option<Action>> {
        match modal {
            Modal::QuitConfirm => self.quit_dialog.handle_key_event(key),
            Modal::RunOptions { .. } => self.run_options_dialog.handle_key_event(key),
            Modal::ProjectInfo => self.project_info_dialog.handle_key_event(key),
            Modal::History { .. } => self.history_dialog.handle_key_event(key),
            Modal::RunOutput => self.run_output_dialog.handle_key_event(key),
            Modal::TargetSelector { .. } => self.target_selector.handle_key_event(key),
            Modal::TagFilter { .. } => self.tag_filter_dialog.handle_key_event(key),
            Modal::GitDiff { .. } => {
                let action = match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => Some(Action::CloseModal),
                    KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollDown),
                    KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollUp),
                    _ => None,
                };
                Ok(action)
            }
            Modal::GitCommit { message } => {
                let action = match key.code {
                    KeyCode::Esc => Some(Action::CloseModal),
                    KeyCode::Enter => Some(Action::GitCommit(message.clone())),
                    KeyCode::Backspace => {
                        if let Some(Modal::GitCommit { message }) = self.modals.top_mut() {
                            message.pop();
                        }
                        None
                    }
                    KeyCode::Char(c) => {
                        if let Some(Modal::GitCommit { message }) = self.modals.top_mut() {
                            message.push(c);
                        }
                        None
                    }
                    _ => None,
                };
                Ok(action)
            }
            Modal::GitLog { .. } => {
                let action = match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => Some(Action::CloseModal),
                    KeyCode::Char('j') | KeyCode::Down => {
                        if let Some(Modal::GitLog { scroll_offset }) = self.modals.top_mut() {
                            *scroll_offset = scroll_offset.saturating_add(1);
                        }
                        None
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if let Some(Modal::GitLog { scroll_offset }) = self.modals.top_mut() {
                            *scroll_offset = scroll_offset.saturating_sub(1);
                        }
                        None
                    }
                    _ => None,
                };
                Ok(action)
            }
            Modal::SampleData { .. } => self.sample_data_dialog.handle_key_event(key),
            Modal::Help { .. } => self.help_dialog.handle_key_event(key),
        }
    }

    fn handle_search_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc | KeyCode::Enter => Some(Action::ExitSearchMode),
            KeyCode::Backspace => Some(Action::SearchBackspace),
            KeyCode::Char(c) => Some(Action::SearchInput(c)),
            _ => None,
        };
        Ok(action)
    }

    fn draw_modal(&mut self, frame: &mut Frame, area: Rect, modal: &Modal) -> Result<()> {
        match modal {
            Modal::QuitConfirm => self.quit_dialog.draw(frame, area)?,
            Modal::RunOptions { .. } => {
                let node_name = self.get_selected_node()
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                self.run_options_dialog.draw_with_node_name(frame, area, &node_name)?;
            }
            Modal::ProjectInfo => {
                self.project_info_dialog.set_project_info(self.domain.project_info.as_ref());
                self.project_info_dialog.draw(frame, area)?;
            }
            Modal::History { .. } => {
                self.history_dialog.draw_with_history(frame, area, &self.domain.run_history)?;
            }
            Modal::RunOutput => {
                if let Some(ref run_output) = self.domain.run_output {
                    self.run_output_dialog.draw_with_output(frame, area, run_output)?;
                }
            }
            Modal::TargetSelector { .. } => {
                self.target_selector.draw(frame, area)?;
            }
            Modal::TagFilter { .. } => {
                self.tag_filter_dialog.draw(frame, area)?;
            }
            Modal::GitDiff { file_path } => {
                self.draw_git_diff(frame, area, file_path)?;
            }
            Modal::GitCommit { message } => {
                self.draw_git_commit(frame, area, message)?;
            }
            Modal::GitLog { scroll_offset } => {
                self.draw_git_log(frame, area, *scroll_offset)?;
            }
            Modal::SampleData { .. } => {
                if let Some(ref output) = self.domain.sample_data_output {
                    self.sample_data_dialog.draw_with_output(frame, area, output)?;
                }
            }
            Modal::Help { .. } => {
                self.help_dialog.draw(frame, area)?;
            }
        }
        Ok(())
    }

    /// Load project from config after setup completes
    fn load_project_from_config(&mut self, config: Config) {
        self.domain.dbt_binary_path = config.dbt_binary_path.clone();
        let project_path = PathBuf::from(&config.project_path);
        self.domain.project_path = Some(project_path.clone());

        let manifest_path = project_path.join("target").join("manifest.json");

        if !manifest_path.exists() {
            self.error = Some(manifest_not_found_error(&manifest_path, &project_path));
            return;
        }

        match services::load_manifest(&manifest_path) {
            Ok(manifest) => {
                self.domain.all_nodes = services::filter_nodes(&manifest);

                let root_path_str = project_path.to_string_lossy().to_string();
                for node in &mut self.domain.all_nodes {
                    node.root_path = Some(root_path_str.clone());
                }

                self.domain.lineage_graph = Some(LineageGraph::build(&self.domain.all_nodes));

                if !self.domain.all_nodes.is_empty() {
                    self.home.select_first(&self.domain.all_nodes);
                } else {
                    self.error = Some("No models, tests, or seeds found in manifest".to_string());
                }

                self.domain.project_info = Some(services::get_project_info(
                    &self.domain.dbt_binary_path,
                    &self.domain.project_path,
                    &self.domain.all_nodes,
                ));
            }
            Err(e) => {
                self.error = Some(manifest_parse_error(&e.to_string()));
            }
        }
    }

    /// Refresh the manifest by reloading from disk
    fn refresh_manifest(&mut self) {
        let project_path = match &self.domain.project_path {
            Some(p) => p.clone(),
            None => {
                self.error = Some("No project path configured".to_string());
                return;
            }
        };

        let manifest_path = project_path.join("target").join("manifest.json");

        if !manifest_path.exists() {
            self.error = Some(manifest_not_found_error(&manifest_path, &project_path));
            return;
        }

        // Clear any existing error
        self.error = None;

        match services::load_manifest(&manifest_path) {
            Ok(manifest) => {
                // Remember current selection
                let current_selection = self.get_selected_node().map(|n| n.unique_id.clone());

                self.domain.all_nodes = services::filter_nodes(&manifest);

                let root_path_str = project_path.to_string_lossy().to_string();
                for node in &mut self.domain.all_nodes {
                    node.root_path = Some(root_path_str.clone());
                }

                self.domain.lineage_graph = Some(LineageGraph::build(&self.domain.all_nodes));

                // Try to restore selection, or select first
                if let Some(unique_id) = current_selection {
                    let idx = self.domain.all_nodes.iter().position(|n| n.unique_id == unique_id);
                    if idx.is_some() {
                        // Selection still exists, rebuild display list
                        self.home.select_first(&self.domain.all_nodes);
                    } else {
                        self.home.select_first(&self.domain.all_nodes);
                    }
                } else {
                    self.home.select_first(&self.domain.all_nodes);
                }

                self.domain.project_info = Some(services::get_project_info(
                    &self.domain.dbt_binary_path,
                    &self.domain.project_path,
                    &self.domain.all_nodes,
                ));

                self.status_message = Some("Manifest refreshed successfully".to_string());
            }
            Err(e) => {
                self.error = Some(manifest_parse_error(&e.to_string()));
            }
        }
    }

    /// Run dbt compile to generate manifest.json, then reload
    fn compile_manifest(&mut self) {
        let project_path = match &self.domain.project_path {
            Some(p) => p.clone(),
            None => {
                self.error = Some("No project path configured".to_string());
                return;
            }
        };

        // Build the compile command
        let (full_command, display_command) =
            services::build_dbt_compile_command(&self.domain.dbt_binary_path, &project_path);

        // Clear error and set status
        self.error = None;
        self.status_message = Some(format!("Running {}...", display_command));

        // Spawn the compile command and track output
        self.domain.run_output = Some(self.job_runner.spawn(full_command));
        if let Some(ref mut output) = self.domain.run_output {
            output.command = display_command;
        }

        // Show the run output modal so user can see progress
        self.modals.push(Modal::RunOutput);
    }

    /// Handle key events when in error state (e.g., missing manifest)
    fn handle_error_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('c') => Ok(Some(Action::CompileManifest)),
            KeyCode::Char('e') => {
                self.mode = AppMode::Setup;
                Ok(None)
            }
            KeyCode::Char('q') | KeyCode::Esc => Ok(Some(Action::ForceQuit)),
            _ => Ok(None),
        }
    }

    /// Change the current target and save to config
    fn change_target(&mut self, target: &str) {
        // Update config
        if let Some(ref mut config) = self.config {
            config.target = target.to_string();
            let _ = config.save();
        } else if let Some(mut config) = Config::load() {
            config.target = target.to_string();
            let _ = config.save();
            self.config = Some(config);
        }

        self.status_message = Some(format!("Target changed to '{}'", target));
    }

    /// Draw git diff modal
    fn draw_git_diff(&self, frame: &mut Frame, area: Rect, file_path: &str) -> Result<()> {
        use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::layout::{Constraint, Direction, Layout};

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

        // Get the diff
        let diff_content = if let Some(ref project_path) = self.domain.project_path {
            services::get_file_full_diff(project_path, file_path)
                .unwrap_or_else(|e| format!("Error getting diff: {}", e))
        } else {
            "No project path".to_string()
        };

        // Color the diff lines
        let lines: Vec<Line> = diff_content
            .lines()
            .map(|line| {
                let style = if line.starts_with('+') && !line.starts_with("+++") {
                    Style::default().fg(Color::Green)
                } else if line.starts_with('-') && !line.starts_with("---") {
                    Style::default().fg(Color::Red)
                } else if line.starts_with("@@") {
                    Style::default().fg(Color::Cyan)
                } else if line.starts_with("diff") || line.starts_with("index") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(line.to_string(), style))
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" Git Diff: {} ", file_path))
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, chunks[0]);

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled(" Esc/q ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Close"),
        ]))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(help, chunks[1]);
        Ok(())
    }

    /// Draw git commit modal
    fn draw_git_commit(&self, frame: &mut Frame, area: Rect, message: &str) -> Result<()> {
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use crate::components::centered_popup;

        let popup_area = centered_popup(area, 60, 10);
        frame.render_widget(Clear, popup_area);

        let content = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Enter commit message:",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("> {}_", message),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Enter ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Commit  "),
                Span::styled(" Esc ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(" Git Commit ")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, popup_area);
        Ok(())
    }

    /// Draw git log modal
    fn draw_git_log(&self, frame: &mut Frame, area: Rect, scroll_offset: usize) -> Result<()> {
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::layout::{Constraint, Direction, Layout};

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

        // Get the log
        let commits = if let Some(ref project_path) = self.domain.project_path {
            services::get_log(project_path, None, 50).unwrap_or_default()
        } else {
            vec![]
        };

        let lines: Vec<Line> = commits
            .iter()
            .map(|commit| {
                Line::from(vec![
                    Span::styled(
                        format!("{} ", commit.short_hash),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("{} ", commit.date),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{}: ", commit.author),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(commit.message.clone()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .title(" Git Log ")
                    .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            )
            .scroll((scroll_offset as u16, 0));

        frame.render_widget(paragraph, chunks[0]);

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled(" Esc/q ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Close  "),
            Span::styled(" j/k ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Scroll"),
        ]))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(help, chunks[1]);
        Ok(())
    }
}
