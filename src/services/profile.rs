//! Profile parsing service
//!
//! Parses dbt profiles.yml to extract target information.

use std::path::Path;

/// Information about a single target/output with raw YAML content
#[derive(Debug, Clone, Default)]
pub struct TargetInfo {
    pub name: String,
    /// The raw YAML content for this target (for display)
    pub yaml_content: String,
}

/// Parsed profiles result
#[derive(Debug, Clone)]
pub struct ProfilesInfo {
    pub targets: Vec<TargetInfo>,
}

/// Parse profiles.yml and return target information with raw YAML
pub fn parse_profiles(project_path: &Path) -> Option<ProfilesInfo> {
    // Try project directory first
    let profiles_path = project_path.join("profiles.yml");
    if profiles_path.exists() {
        return parse_profiles_file(&profiles_path);
    }

    // Try ~/.dbt/profiles.yml
    if let Ok(home) = std::env::var("HOME") {
        let home_profiles = Path::new(&home).join(".dbt").join("profiles.yml");
        if home_profiles.exists() {
            return parse_profiles_file(&home_profiles);
        }
    }

    None
}

fn parse_profiles_file(path: &Path) -> Option<ProfilesInfo> {
    let content = std::fs::read_to_string(path).ok()?;

    // Parse YAML to get structure
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    let profiles = yaml.as_mapping()?;

    // Get the first profile
    let (_profile_name, profile_value) = profiles.iter().next()?;
    let profile = profile_value.as_mapping()?;

    // Get default target
    let default_target = profile
        .get(serde_yaml::Value::String("target".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("dev")
        .to_string();

    // Get outputs
    let outputs = profile
        .get(serde_yaml::Value::String("outputs".to_string()))
        .and_then(|v| v.as_mapping())?;

    let mut targets: Vec<TargetInfo> = outputs
        .iter()
        .filter_map(|(name, value)| {
            let name_str = name.as_str()?.to_string();

            // Convert the target config back to YAML for display
            let yaml_content = serde_yaml::to_string(value)
                .ok()
                .unwrap_or_else(|| "# Unable to parse".to_string());

            Some(TargetInfo {
                name: name_str,
                yaml_content,
            })
        })
        .collect();

    // Sort by name, but put default target first
    targets.sort_by(|a, b| {
        if a.name == default_target {
            std::cmp::Ordering::Less
        } else if b.name == default_target {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Some(ProfilesInfo { targets })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_info_default() {
        let info = TargetInfo::default();
        assert!(info.name.is_empty());
        assert!(info.yaml_content.is_empty());
    }
}
