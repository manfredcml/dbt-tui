//! dbt CLI interaction services

use crate::model::{DbtCommand, RunFlags};
use std::path::Path;

/// Build a dbt command string for any DbtCommand type
///
/// The command includes:
/// - `--project-dir` to specify the dbt project location
/// - `--profiles-dir` to look for profiles.yml in the project directory first,
///   falling back to the default ~/.dbt/ location
/// - Optional flags like --full-refresh, --vars, --exclude
pub fn build_dbt_command(
    dbt_binary_path: &str,
    project_path: &Path,
    command: DbtCommand,
    selector: Option<&str>,
    flags: &RunFlags,
) -> (String, String) {
    let dbt_cmd = if dbt_binary_path.is_empty() {
        "dbt".to_string()
    } else {
        dbt_binary_path.to_string()
    };

    let project_dir_arg = format!("--project-dir \"{}\"", project_path.display());

    // Check if profiles.yml exists in the project directory
    // If so, use --profiles-dir to point to it
    let profiles_dir_arg = if project_path.join("profiles.yml").exists() {
        format!(" --profiles-dir \"{}\"", project_path.display())
    } else {
        String::new()
    };

    // Build additional flags (only for commands that support them)
    let (extra_flags, display_extra_flags) = build_extra_flags(command, flags);

    // Build select clause if selector is provided and command supports it
    let select_clause = if command.supports_select() {
        selector
            .map(|s| format!(" --select {}", s))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let display_select = if command.supports_select() {
        selector
            .map(|s| format!(" --select {}", s))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let full_command = format!(
        "{} {} {}{}{}{}",
        dbt_cmd,
        command.subcommand(),
        project_dir_arg,
        profiles_dir_arg,
        select_clause,
        extra_flags
    );

    let display_command = format!(
        "dbt {}{}{}",
        command.subcommand(),
        display_select,
        display_extra_flags
    );

    (full_command, display_command)
}

/// Build a dbt show command for previewing model data
///
/// Returns (full_command, display_command) tuple
pub fn build_dbt_show_command(
    dbt_binary_path: &str,
    project_path: &Path,
    model_name: &str,
    limit: usize,
) -> (String, String) {
    let dbt_cmd = if dbt_binary_path.is_empty() {
        "dbt".to_string()
    } else {
        dbt_binary_path.to_string()
    };

    let project_dir_arg = format!("--project-dir \"{}\"", project_path.display());

    // Check if profiles.yml exists in the project directory
    let profiles_dir_arg = if project_path.join("profiles.yml").exists() {
        format!(" --profiles-dir \"{}\"", project_path.display())
    } else {
        String::new()
    };

    let full_command = format!(
        "{} show --select {} --limit {} {}{}",
        dbt_cmd, model_name, limit, project_dir_arg, profiles_dir_arg
    );

    let display_command = format!("dbt show --select {} --limit {}", model_name, limit);

    (full_command, display_command)
}

/// Build a simple dbt compile command (no selector, no flags)
///
/// Returns (full_command, display_command) tuple
pub fn build_dbt_compile_command(dbt_binary_path: &str, project_path: &Path) -> (String, String) {
    let dbt_cmd = if dbt_binary_path.is_empty() {
        "dbt".to_string()
    } else {
        dbt_binary_path.to_string()
    };

    let project_dir_arg = format!("--project-dir \"{}\"", project_path.display());

    // Check if profiles.yml exists in the project directory
    let profiles_dir_arg = if project_path.join("profiles.yml").exists() {
        format!(" --profiles-dir \"{}\"", project_path.display())
    } else {
        String::new()
    };

    let full_command = format!("{} compile {}{}", dbt_cmd, project_dir_arg, profiles_dir_arg);
    let display_command = "dbt compile".to_string();

    (full_command, display_command)
}

/// Build extra flags string based on command type
fn build_extra_flags(command: DbtCommand, flags: &RunFlags) -> (String, String) {
    let mut extra_flags = String::new();
    let mut display_extra_flags = String::new();

    // --full-refresh only applies to run and build
    if flags.full_refresh && matches!(command, DbtCommand::Run | DbtCommand::Build) {
        extra_flags.push_str(" --full-refresh");
        display_extra_flags.push_str(" --full-refresh");
    }

    // --vars applies to run, test, build, compile
    if !flags.vars.is_empty()
        && matches!(
            command,
            DbtCommand::Run | DbtCommand::Test | DbtCommand::Build | DbtCommand::Compile
        )
    {
        extra_flags.push_str(&format!(" --vars '{}'", flags.vars));
        display_extra_flags.push_str(&format!(" --vars '{}'", flags.vars));
    }

    // --exclude applies to run, test, build, compile
    if !flags.exclude.is_empty()
        && matches!(
            command,
            DbtCommand::Run | DbtCommand::Test | DbtCommand::Build | DbtCommand::Compile
        )
    {
        extra_flags.push_str(&format!(" --exclude {}", flags.exclude));
        display_extra_flags.push_str(&format!(" --exclude {}", flags.exclude));
    }

    (extra_flags, display_extra_flags)
}

