//! Domain state - business/data state separate from UI concerns

use super::history::RunHistoryEntry;
use super::lineage::LineageGraph;
use super::node::Node;
use super::run::RunOutput;
use super::sample_data::SampleDataOutput;
use std::path::PathBuf;

/// Project information for display
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub dbt_version: String,
    pub project_name: String,
    pub profile_name: String,
    pub project_path: String,
    pub target: String,
    pub models_count: usize,
    pub tests_count: usize,
    pub seeds_count: usize,
    pub profile_type: String,
    pub profile_host: String,
    pub profile_port: String,
    pub profile_database: String,
    pub profile_schema: String,
    pub profile_user: String,
    pub profile_threads: String,
}

/// Domain state containing all business data
#[derive(Default)]
pub struct DomainState {
    /// All dbt nodes (models, tests, seeds)
    pub all_nodes: Vec<Node>,

    /// Lineage graph built from node dependencies
    pub lineage_graph: Option<LineageGraph>,

    /// Run history entries
    pub run_history: Vec<RunHistoryEntry>,

    /// Current run output (if any)
    pub run_output: Option<RunOutput>,

    /// Current sample data output (if any)
    pub sample_data_output: Option<SampleDataOutput>,

    /// Cached project information
    pub project_info: Option<ProjectInfo>,

    /// Path to the dbt project
    pub project_path: Option<PathBuf>,

    /// Path to the dbt binary
    pub dbt_binary_path: String,
}

impl DomainState {
    /// Create a new domain state with default values
    pub fn new() -> Self {
        Self {
            all_nodes: Vec::new(),
            lineage_graph: None,
            run_history: Vec::new(),
            run_output: None,
            sample_data_output: None,
            project_info: None,
            project_path: None,
            dbt_binary_path: "dbt".to_string(),
        }
    }
}
