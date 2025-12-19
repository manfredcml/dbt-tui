//! Project information reading services

use crate::model::{Node, ProjectInfo};
use std::path::Path;

/// Get dbt version from the binary
pub fn get_dbt_version(dbt_binary_path: &str) -> String {
    let dbt_cmd = if dbt_binary_path.is_empty() {
        "dbt"
    } else {
        dbt_binary_path
    };

    let output = std::process::Command::new(dbt_cmd)
        .arg("--version")
        .output();

    match output {
        Ok(result) => {
            // Check if command succeeded
            if !result.status.success() {
                return format!("Error running {}", dbt_cmd);
            }

            let stdout = String::from_utf8_lossy(&result.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                // Look for "- installed: X.X.X" pattern (dbt >= 1.x format)
                if trimmed.starts_with("- installed:") {
                    if let Some(version) = trimmed.split(':').nth(1) {
                        let ver = version.trim();
                        if !ver.is_empty() {
                            return ver.to_string();
                        }
                    }
                }
                // Look for "dbt-core==X.X.X" pattern
                if trimmed.to_lowercase().contains("dbt") && trimmed.contains("==") {
                    if let Some(ver) = trimmed.split("==").nth(1) {
                        let ver = ver.trim();
                        if !ver.is_empty() {
                            return ver.to_string();
                        }
                    }
                }
                // Look for version pattern like "1.8.0" or "1.8.0rc1"
                if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    return trimmed.to_string();
                }
            }
            // Fallback: return first non-empty line
            stdout
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("Unknown")
                .to_string()
        }
        Err(_) => "dbt not found".to_string(),
    }
}

/// Read dbt_project.yml
pub fn read_dbt_project_yml(project_path: &Path) -> Option<serde_json::Value> {
    let project_file = project_path.join("dbt_project.yml");
    if !project_file.exists() {
        return None;
    }

    let content = std::fs::read_to_string(project_file).ok()?;
    serde_yaml::from_str(&content).ok()
}

/// Read profiles.yml and find the specific profile
pub fn read_profiles_yml(
    project_path: Option<&Path>,
    profile_name: &str,
) -> Option<serde_json::Value> {
    let try_read_profile = |path: &Path| -> Option<serde_json::Value> {
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        let yaml: serde_json::Value = serde_yaml::from_str(&content).ok()?;

        if yaml.get(profile_name).is_some() {
            return Some(yaml);
        }

        None
    };

    // First try ~/.dbt/profiles.yml
    if let Ok(home) = std::env::var("HOME") {
        let profiles_path = Path::new(&home).join(".dbt").join("profiles.yml");
        if let Some(profiles) = try_read_profile(&profiles_path) {
            return Some(profiles);
        }
    }

    // Fall back to project directory
    if let Some(project_dir) = project_path {
        let local_profiles = project_dir.join("profiles.yml");
        if let Some(profiles) = try_read_profile(&local_profiles) {
            return Some(profiles);
        }
    }

    None
}

/// Get complete project information
pub fn get_project_info(
    dbt_binary_path: &str,
    dbt_project_path: &Option<std::path::PathBuf>,
    all_nodes: &[Node],
) -> ProjectInfo {
    let dbt_version = get_dbt_version(dbt_binary_path);

    let (
        project_name,
        profile_name,
        target,
        profile_type,
        profile_host,
        profile_port,
        profile_database,
        profile_schema,
        profile_user,
        profile_threads,
    ) = if let Some(project_path) = dbt_project_path {
        let project_yml = read_dbt_project_yml(project_path);

        let project_name = project_yml
            .as_ref()
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profile_name = project_yml
            .as_ref()
            .and_then(|v| v.get("profile"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profiles_yml = read_profiles_yml(Some(project_path), &profile_name);

        let target = profiles_yml
            .as_ref()
            .and_then(|v| v.get(&profile_name))
            .and_then(|v| v.get("target"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let target_config = profiles_yml
            .as_ref()
            .and_then(|v| v.get(&profile_name))
            .and_then(|v| v.get("outputs"))
            .and_then(|v| v.get(&target));

        let profile_type = target_config
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profile_host = target_config
            .and_then(|v| v.get("host"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profile_port = target_config
            .and_then(|v| v.get("port"))
            .and_then(|v| {
                if let Some(p) = v.as_i64() {
                    Some(p.to_string())
                } else {
                    v.as_str().map(|s| s.to_string())
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let profile_database = target_config
            .and_then(|v| v.get("dbname").or(v.get("database")))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profile_schema = target_config
            .and_then(|v| v.get("schema"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profile_user = target_config
            .and_then(|v| v.get("user").or(v.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let profile_threads = target_config
            .and_then(|v| v.get("threads"))
            .and_then(|v| {
                if let Some(t) = v.as_i64() {
                    Some(t.to_string())
                } else {
                    v.as_str().map(|s| s.to_string())
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        (
            project_name,
            profile_name,
            target,
            profile_type,
            profile_host,
            profile_port,
            profile_database,
            profile_schema,
            profile_user,
            profile_threads,
        )
    } else {
        (
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
        )
    };

    let project_path = dbt_project_path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let models_count = all_nodes
        .iter()
        .filter(|n| n.resource_type == "model")
        .count();
    let tests_count = all_nodes
        .iter()
        .filter(|n| n.resource_type == "test")
        .count();
    let seeds_count = all_nodes
        .iter()
        .filter(|n| n.resource_type == "seed")
        .count();

    ProjectInfo {
        dbt_version,
        project_name,
        profile_name,
        project_path,
        target,
        models_count,
        tests_count,
        seeds_count,
        profile_type,
        profile_host,
        profile_port,
        profile_database,
        profile_schema,
        profile_user,
        profile_threads,
    }
}
