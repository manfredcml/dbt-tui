//! Data models for dbt run execution and output

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;

use crate::model::node::Node;

/// Status of a dbt run
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum RunStatus {
    #[default]
    Running,
    Success,
    Failed,
}

/// Selection mode for dbt run command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunSelectMode {
    /// Just the model: --select model
    Single,
    /// Downstream: --select model+
    Downstream,
    /// Upstream: --select +model
    Upstream,
    /// Both: --select +model+
    UpstreamAndDownstream,
}

/// Run flags for dbt commands
#[derive(Debug, Clone, Default)]
pub struct RunFlags {
    /// --full-refresh flag
    pub full_refresh: bool,
    /// --vars JSON string
    pub vars: String,
    /// --exclude pattern
    pub exclude: String,
}

/// Available dbt commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbtCommand {
    /// dbt run - Execute models
    Run,
    /// dbt test - Run tests
    Test,
    /// dbt build - Run + test in dependency order
    Build,
    /// dbt compile - Compile SQL without executing
    Compile,
    /// dbt deps - Install packages from packages.yml
    Deps,
}

impl DbtCommand {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            DbtCommand::Run => "dbt run",
            DbtCommand::Test => "dbt test",
            DbtCommand::Build => "dbt build",
            DbtCommand::Compile => "dbt compile",
            DbtCommand::Deps => "dbt deps",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            DbtCommand::Run => "Execute selected model(s)",
            DbtCommand::Test => "Run tests for selected model(s)",
            DbtCommand::Build => "Run + test in dependency order",
            DbtCommand::Compile => "Compile SQL without executing",
            DbtCommand::Deps => "Install packages from packages.yml",
        }
    }

    /// Get the dbt subcommand
    pub fn subcommand(&self) -> &'static str {
        match self {
            DbtCommand::Run => "run",
            DbtCommand::Test => "test",
            DbtCommand::Build => "build",
            DbtCommand::Compile => "compile",
            DbtCommand::Deps => "deps",
        }
    }

    /// Whether this command requires model selection
    pub fn requires_selection(&self) -> bool {
        match self {
            DbtCommand::Run | DbtCommand::Test | DbtCommand::Build => true,
            DbtCommand::Compile | DbtCommand::Deps => false,
        }
    }

    /// Whether this command supports --select flag
    pub fn supports_select(&self) -> bool {
        match self {
            DbtCommand::Run | DbtCommand::Test | DbtCommand::Build | DbtCommand::Compile => true,
            DbtCommand::Deps => false,
        }
    }

    /// Shortcut key for quick access
    pub fn shortcut(&self) -> char {
        match self {
            DbtCommand::Run => 'r',
            DbtCommand::Test => 't',
            DbtCommand::Build => 'b',
            DbtCommand::Compile => 'c',
            DbtCommand::Deps => 'd',
        }
    }
}

impl RunSelectMode {
    pub fn all() -> [RunSelectMode; 4] {
        [
            RunSelectMode::Single,
            RunSelectMode::Downstream,
            RunSelectMode::Upstream,
            RunSelectMode::UpstreamAndDownstream,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            RunSelectMode::Single => "Just this model",
            RunSelectMode::Downstream => "This model + downstream",
            RunSelectMode::Upstream => "Upstream + this model",
            RunSelectMode::UpstreamAndDownstream => "Upstream + this + downstream",
        }
    }

    pub fn selector(&self, model_name: &str) -> String {
        match self {
            RunSelectMode::Single => model_name.to_string(),
            RunSelectMode::Downstream => format!("{}+", model_name),
            RunSelectMode::Upstream => format!("+{}", model_name),
            RunSelectMode::UpstreamAndDownstream => format!("+{}+", model_name),
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            RunSelectMode::Single => '1',
            RunSelectMode::Downstream => '2',
            RunSelectMode::Upstream => '3',
            RunSelectMode::UpstreamAndDownstream => '4',
        }
    }
}

/// View mode for run output display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunOutputViewMode {
    /// Raw text output
    Raw,
    /// Graphical view with model boxes
    Graphical,
}

/// Status of an individual model run
#[derive(Debug, Clone, PartialEq)]
pub enum ModelRunStatus {
    Running,
    Success,
    Failed,
    Skipped,
}

/// Information about a single model execution
#[derive(Debug, Clone)]
pub struct ModelRun {
    pub name: String,
    pub model_type: String,
    pub status: ModelRunStatus,
    pub duration: Option<f64>,
    pub result_info: Option<String>,
    pub step: Option<String>,
    pub upstream_deps: Vec<String>,
    pub layer: usize,
}

/// Output from a dbt run command
#[derive(Debug, Clone)]
pub struct RunOutput {
    pub command: String,
    pub status: RunStatus,
    pub output: String,
    pub view_mode: RunOutputViewMode,
    pub model_runs: Vec<ModelRun>,
}

impl RunOutput {
    pub fn new(command: String) -> Self {
        Self {
            command,
            status: RunStatus::Running,
            output: String::new(),
            view_mode: RunOutputViewMode::Graphical,
            model_runs: Vec::new(),
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            RunOutputViewMode::Raw => RunOutputViewMode::Graphical,
            RunOutputViewMode::Graphical => RunOutputViewMode::Raw,
        };
    }

    /// Parse a line of output and update model runs
    pub fn parse_output_line(&mut self, line: &str) {
        if line.contains(" START ") && line.contains(" model ") {
            if let Some(model_run) = Self::parse_start_line(line) {
                if !self.model_runs.iter().any(|m| m.name == model_run.name) {
                    self.model_runs.push(model_run);
                }
            }
        } else if (line.contains(" OK ") || line.contains(" ERROR ") || line.contains(" SKIP "))
            && line.contains(" model ")
        {
            Self::parse_completion_line(line, &mut self.model_runs);
        }
    }

    fn parse_start_line(line: &str) -> Option<ModelRun> {
        let step = Self::extract_step(line);
        let model_type = Self::extract_model_type(line);
        let name = Self::extract_model_name(line)?;

        Some(ModelRun {
            name,
            model_type,
            status: ModelRunStatus::Running,
            duration: None,
            result_info: None,
            step,
            upstream_deps: Vec::new(),
            layer: 0,
        })
    }

    fn parse_completion_line(line: &str, model_runs: &mut Vec<ModelRun>) {
        let name = match Self::extract_model_name(line) {
            Some(n) => n,
            None => return,
        };

        let model = match model_runs.iter_mut().find(|m| m.name == name) {
            Some(m) => m,
            None => {
                model_runs.push(ModelRun {
                    name: name.clone(),
                    model_type: Self::extract_model_type(line),
                    status: ModelRunStatus::Running,
                    duration: None,
                    result_info: None,
                    step: Self::extract_step(line),
                    upstream_deps: Vec::new(),
                    layer: 0,
                });
                model_runs.last_mut().unwrap()
            }
        };

        if line.contains(" OK ") {
            model.status = ModelRunStatus::Success;
        } else if line.contains(" ERROR ") {
            model.status = ModelRunStatus::Failed;
        } else if line.contains(" SKIP ") {
            model.status = ModelRunStatus::Skipped;
        }

        if let Some(bracket_start) = line.rfind('[') {
            if let Some(bracket_end) = line.rfind(']') {
                let bracket_content = &line[bracket_start + 1..bracket_end];

                if let Some(in_pos) = bracket_content.rfind(" in ") {
                    let duration_str = &bracket_content[in_pos + 4..];
                    if let Some(duration) = Self::parse_duration(duration_str) {
                        model.duration = Some(duration);
                    }
                    model.result_info = Some(bracket_content[..in_pos].to_string());
                } else {
                    model.result_info = Some(bracket_content.to_string());
                }
            }
        }
    }

    fn extract_step(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for i in 0..parts.len().saturating_sub(2) {
            if parts.get(i + 1) == Some(&"of") {
                if let (Ok(_), Ok(_)) = (parts[i].parse::<u32>(), parts[i + 2].parse::<u32>()) {
                    return Some(format!("{} of {}", parts[i], parts[i + 2]));
                }
            }
        }
        None
    }

    fn extract_model_type(line: &str) -> String {
        if line.contains(" view ") {
            "view".to_string()
        } else if line.contains(" table ") {
            "table".to_string()
        } else if line.contains(" incremental ") {
            "incremental".to_string()
        } else if line.contains(" seed ") {
            "seed".to_string()
        } else if line.contains(" test ") {
            "test".to_string()
        } else {
            "model".to_string()
        }
    }

    fn extract_model_name(line: &str) -> Option<String> {
        let model_marker = " model ";
        let model_pos = line.find(model_marker)?;
        let after_model = &line[model_pos + model_marker.len()..];

        let end_pos = after_model
            .find(" ...")
            .or_else(|| after_model.find(" ["))
            .unwrap_or(after_model.len());

        let name = after_model[..end_pos].trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }

    fn parse_duration(s: &str) -> Option<f64> {
        let s = s.trim();
        if let Some(stripped) = s.strip_suffix('s') {
            stripped.parse().ok()
        } else {
            s.parse().ok()
        }
    }

    /// Get models organized by layer for rendering
    pub fn get_models_by_layer(&self) -> Vec<Vec<&ModelRun>> {
        if self.model_runs.is_empty() {
            return vec![];
        }

        let max_layer = self.model_runs.iter().map(|m| m.layer).max().unwrap_or(0);
        let mut layers: Vec<Vec<&ModelRun>> = vec![vec![]; max_layer + 1];

        for model in &self.model_runs {
            if model.layer <= max_layer {
                layers[model.layer].push(model);
            }
        }

        layers
    }

    /// Compute layers for model runs based on the manifest dependency graph.
    /// Models with no upstream dependencies (within the run) are layer 0.
    /// Models depending only on layer N are layer N+1.
    pub fn compute_layers(&mut self, all_nodes: &[Node]) {
        if self.model_runs.is_empty() {
            return;
        }

        // Build a mapping from run output model name (schema.name) to Node
        // Run output names are like "staging.stg_campaigns"
        // Node has schema and name fields
        let node_by_display_name: HashMap<String, &Node> = all_nodes
            .iter()
            .map(|n| {
                let display_name = format!("{}.{}", n.schema, n.name);
                (display_name, n)
            })
            .collect();

        // Get the set of model names in this run
        let run_model_names: HashSet<String> =
            self.model_runs.iter().map(|m| m.name.clone()).collect();

        // Build dependency graph: model_name -> upstream model names (only those in this run)
        let mut deps_in_run: HashMap<String, Vec<String>> = HashMap::new();

        for model in &self.model_runs {
            let mut upstream_deps = Vec::new();

            if let Some(node) = node_by_display_name.get(&model.name) {
                // For each dependency in the manifest
                for dep_unique_id in &node.depends_on.nodes {
                    // Find the node for this dependency
                    if let Some(dep_node) = all_nodes.iter().find(|n| &n.unique_id == dep_unique_id)
                    {
                        let dep_display_name = format!("{}.{}", dep_node.schema, dep_node.name);
                        // Only include if this dependency is in the current run
                        if run_model_names.contains(&dep_display_name) {
                            upstream_deps.push(dep_display_name);
                        }
                    }
                }
            }

            deps_in_run.insert(model.name.clone(), upstream_deps);
        }

        // Compute layers using iterative approach
        // Layer 0: models with no dependencies in this run
        // Layer N: models whose all dependencies are in layer < N
        let mut model_layers: HashMap<String, usize> = HashMap::new();
        let mut remaining: HashSet<String> = run_model_names.clone();
        let mut current_layer = 0;

        while !remaining.is_empty() {
            let mut assigned_this_layer = Vec::new();

            for model_name in &remaining {
                let deps = deps_in_run.get(model_name).map(|v| v.as_slice()).unwrap_or(&[]);

                // Check if all dependencies have been assigned a layer
                let all_deps_assigned = deps
                    .iter()
                    .all(|dep| model_layers.contains_key(dep));

                if all_deps_assigned {
                    assigned_this_layer.push(model_name.clone());
                }
            }

            // If no models could be assigned, we have a cycle - assign remaining to current layer
            if assigned_this_layer.is_empty() {
                for model_name in &remaining {
                    model_layers.insert(model_name.clone(), current_layer);
                }
                break;
            }

            for model_name in &assigned_this_layer {
                model_layers.insert(model_name.clone(), current_layer);
                remaining.remove(model_name);
            }

            current_layer += 1;
        }

        // Update model_runs with computed layers and upstream_deps
        for model in &mut self.model_runs {
            if let Some(&layer) = model_layers.get(&model.name) {
                model.layer = layer;
            }
            if let Some(deps) = deps_in_run.get(&model.name) {
                model.upstream_deps = deps.clone();
            }
        }
    }
}

/// Message types sent from background job threads
pub enum JobMessage {
    Output(String),
    Completed(Option<i32>),
    Error(String),
}

/// A background job running a dbt command
pub struct BackgroundJob {
    pub receiver: Receiver<JobMessage>,
    pub start_instant: std::time::Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{DependsOn, NodeConfig};
    use std::collections::HashMap;

    /// Helper to create a test Node
    fn create_test_node(name: &str, schema: &str, depends_on: Vec<String>) -> Node {
        Node {
            unique_id: format!("model.test_project.{}", name),
            name: name.to_string(),
            resource_type: "model".to_string(),
            package_name: "test_project".to_string(),
            schema: schema.to_string(),
            compiled_code: None,
            raw_code: None,
            depends_on: DependsOn { nodes: depends_on },
            root_path: None,
            original_file_path: None,
            config: NodeConfig::default(),
            compiled_path: None,
            description: None,
            columns: HashMap::new(),
        }
    }

    #[test]
    fn test_model_run_parse_start_line() {
        let mut run_output = RunOutput::new("dbt run".to_string());
        run_output.parse_output_line(
            "22:09:24 40  1 of 3 START sql view model analytics_staging.stg_customers ... [RUN]",
        );

        assert_eq!(run_output.model_runs.len(), 1);
        let model = &run_output.model_runs[0];
        assert_eq!(model.name, "analytics_staging.stg_customers");
        assert_eq!(model.model_type, "view");
        assert_eq!(model.status, ModelRunStatus::Running);
    }

    #[test]
    fn test_run_output_view_mode_toggle() {
        let mut run_output = RunOutput::new("dbt run".to_string());
        assert_eq!(run_output.view_mode, RunOutputViewMode::Graphical);

        run_output.toggle_view_mode();
        assert_eq!(run_output.view_mode, RunOutputViewMode::Raw);

        run_output.toggle_view_mode();
        assert_eq!(run_output.view_mode, RunOutputViewMode::Graphical);
    }

    #[test]
    fn test_compute_layers_single_model_no_deps() {
        let mut run_output = RunOutput::new("dbt run".to_string());
        run_output.parse_output_line(
            "22:09:24  1 of 1 START sql view model staging.stg_customers ... [RUN]",
        );

        let nodes = vec![create_test_node("stg_customers", "staging", vec![])];

        run_output.compute_layers(&nodes);

        assert_eq!(run_output.model_runs.len(), 1);
        assert_eq!(run_output.model_runs[0].layer, 0);
        assert!(run_output.model_runs[0].upstream_deps.is_empty());
    }

    #[test]
    fn test_compute_layers_linear_chain() {
        // stg_customers -> int_customers -> fct_orders
        let mut run_output = RunOutput::new("dbt run".to_string());
        run_output.parse_output_line(
            "22:09:24  1 of 3 START sql view model staging.stg_customers ... [RUN]",
        );
        run_output.parse_output_line(
            "22:09:25  2 of 3 START sql view model analytics.int_customers ... [RUN]",
        );
        run_output.parse_output_line(
            "22:09:26  3 of 3 START sql table model marts.fct_orders ... [RUN]",
        );

        let nodes = vec![
            create_test_node("stg_customers", "staging", vec![]),
            create_test_node(
                "int_customers",
                "analytics",
                vec!["model.test_project.stg_customers".to_string()],
            ),
            create_test_node(
                "fct_orders",
                "marts",
                vec!["model.test_project.int_customers".to_string()],
            ),
        ];

        run_output.compute_layers(&nodes);

        // Find each model and check its layer
        let stg = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "staging.stg_customers")
            .unwrap();
        let int = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "analytics.int_customers")
            .unwrap();
        let fct = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "marts.fct_orders")
            .unwrap();

        assert_eq!(stg.layer, 0);
        assert_eq!(int.layer, 1);
        assert_eq!(fct.layer, 2);

        // Check upstream_deps
        assert!(stg.upstream_deps.is_empty());
        assert_eq!(int.upstream_deps, vec!["staging.stg_customers"]);
        assert_eq!(fct.upstream_deps, vec!["analytics.int_customers"]);
    }

    #[test]
    fn test_compute_layers_diamond_pattern() {
        // stg_a and stg_b both feed into int_combined
        //     stg_a
        //         \
        //          int_combined
        //         /
        //     stg_b
        let mut run_output = RunOutput::new("dbt run".to_string());
        run_output.parse_output_line(
            "22:09:24  1 of 3 START sql view model staging.stg_a ... [RUN]",
        );
        run_output.parse_output_line(
            "22:09:24  2 of 3 START sql view model staging.stg_b ... [RUN]",
        );
        run_output.parse_output_line(
            "22:09:25  3 of 3 START sql table model analytics.int_combined ... [RUN]",
        );

        let nodes = vec![
            create_test_node("stg_a", "staging", vec![]),
            create_test_node("stg_b", "staging", vec![]),
            create_test_node(
                "int_combined",
                "analytics",
                vec![
                    "model.test_project.stg_a".to_string(),
                    "model.test_project.stg_b".to_string(),
                ],
            ),
        ];

        run_output.compute_layers(&nodes);

        let stg_a = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "staging.stg_a")
            .unwrap();
        let stg_b = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "staging.stg_b")
            .unwrap();
        let int = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "analytics.int_combined")
            .unwrap();

        // Both staging models should be layer 0
        assert_eq!(stg_a.layer, 0);
        assert_eq!(stg_b.layer, 0);
        // int_combined depends on both, so it's layer 1
        assert_eq!(int.layer, 1);
        assert_eq!(int.upstream_deps.len(), 2);
    }

    #[test]
    fn test_compute_layers_partial_run() {
        // Full graph: stg_a -> int_a -> fct_a
        // But only running int_a and fct_a (not stg_a)
        let mut run_output = RunOutput::new("dbt run".to_string());
        run_output.parse_output_line(
            "22:09:24  1 of 2 START sql view model analytics.int_a ... [RUN]",
        );
        run_output.parse_output_line(
            "22:09:25  2 of 2 START sql table model marts.fct_a ... [RUN]",
        );

        let nodes = vec![
            create_test_node("stg_a", "staging", vec![]),
            create_test_node(
                "int_a",
                "analytics",
                vec!["model.test_project.stg_a".to_string()],
            ),
            create_test_node(
                "fct_a",
                "marts",
                vec!["model.test_project.int_a".to_string()],
            ),
        ];

        run_output.compute_layers(&nodes);

        let int = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "analytics.int_a")
            .unwrap();
        let fct = run_output
            .model_runs
            .iter()
            .find(|m| m.name == "marts.fct_a")
            .unwrap();

        // int_a's dep (stg_a) is not in the run, so int_a is layer 0
        assert_eq!(int.layer, 0);
        // fct_a depends on int_a which is in the run, so it's layer 1
        assert_eq!(fct.layer, 1);

        // int_a has no upstream deps in the run
        assert!(int.upstream_deps.is_empty());
        // fct_a has int_a as upstream dep
        assert_eq!(fct.upstream_deps, vec!["analytics.int_a"]);
    }

    #[test]
    fn test_compute_layers_empty_run() {
        let mut run_output = RunOutput::new("dbt run".to_string());
        let nodes = vec![create_test_node("stg_a", "staging", vec![])];

        // Should not panic
        run_output.compute_layers(&nodes);

        assert!(run_output.model_runs.is_empty());
    }
}
