//! Manifest loading and node filtering services

use crate::model::{Manifest, Node};
use std::fs;
use std::path::Path;

/// Load and parse the manifest.json file
pub fn load_manifest<P: AsRef<Path>>(path: P) -> Result<Manifest, String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("Failed to read manifest.json: {}", e))?;

    let manifest: Manifest = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

    Ok(manifest)
}

/// Filter nodes to only include models, tests, seeds, and snapshots
pub fn filter_nodes(manifest: &Manifest) -> Vec<Node> {
    let mut nodes: Vec<Node> = manifest
        .nodes
        .values()
        .filter(|node| {
            matches!(
                node.resource_type.as_str(),
                "model" | "test" | "seed" | "snapshot"
            )
        })
        .cloned()
        .collect();

    // Sort by resource type, then by schema, then by name
    nodes.sort_by(|a, b| {
        let type_order = |t: &str| match t {
            "model" => 0,
            "test" => 1,
            "seed" => 2,
            "snapshot" => 3,
            _ => 4,
        };

        type_order(&a.resource_type)
            .cmp(&type_order(&b.resource_type))
            .then_with(|| a.group_schema().cmp(&b.group_schema()))
            .then_with(|| a.name.cmp(&b.name))
    });

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{DependsOn, NodeConfig};
    use std::collections::HashMap;

    #[test]
    fn test_filter_nodes_only_includes_relevant_types() {
        let mut nodes = HashMap::new();

        nodes.insert(
            "model.test.example".to_string(),
            Node {
                unique_id: "model.test.example".to_string(),
                name: "example".to_string(),
                resource_type: "model".to_string(),
                package_name: "test".to_string(),
                schema: "public".to_string(),
                compiled_code: None,
                raw_code: None,
                depends_on: DependsOn::default(),
                root_path: None,
                original_file_path: None,
                config: NodeConfig::default(),
                compiled_path: None,
                description: None,
                columns: HashMap::new(),
            },
        );

        nodes.insert(
            "source.test.raw".to_string(),
            Node {
                unique_id: "source.test.raw".to_string(),
                name: "raw".to_string(),
                resource_type: "source".to_string(),
                package_name: "test".to_string(),
                schema: "public".to_string(),
                compiled_code: None,
                raw_code: None,
                depends_on: DependsOn::default(),
                root_path: None,
                original_file_path: None,
                config: NodeConfig::default(),
                compiled_path: None,
                description: None,
                columns: HashMap::new(),
            },
        );

        let manifest = Manifest { nodes };
        let filtered = filter_nodes(&manifest);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].resource_type, "model");
    }

    #[test]
    fn test_load_sample_manifest_parses_tags() {
        let manifest_path = "examples/example_dbt/target/manifest.json";
        if !std::path::Path::new(manifest_path).exists() {
            // Skip if example project doesn't exist
            return;
        }

        let manifest = load_manifest(manifest_path).expect("Failed to load manifest");
        let nodes = filter_nodes(&manifest);

        // Collect all tags from all nodes
        let all_tags: Vec<&str> = nodes
            .iter()
            .flat_map(|n| n.config.tags.iter().map(|s| s.as_str()))
            .collect();

        // Should have staging and marts tags
        assert!(
            all_tags.contains(&"staging") || all_tags.contains(&"marts"),
            "Expected tags 'staging' or 'marts' but found: {:?}",
            all_tags
        );
    }
}
