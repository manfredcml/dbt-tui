//! External service interactions
//!
//! This module contains services for interacting with external systems:
//! - dbt CLI commands
//! - Manifest loading and parsing
//! - Project information reading
//! - Profile parsing
//! - Background job execution
//! - Git repository operations

pub mod dbt;
pub mod git;
pub mod job_runner;
pub mod manifest;
pub mod profile;
pub mod project;

pub use dbt::{build_dbt_command, build_dbt_compile_command, build_dbt_show_command};
pub use git::{
    commit, get_file_full_diff, get_log, get_status, is_git_repo, stage_file, GitFileStatus,
};
pub use job_runner::JobRunner;
pub use manifest::{filter_nodes, load_manifest};
pub use profile::{parse_profiles, TargetInfo};
pub use project::get_project_info;
