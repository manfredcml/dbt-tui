//! Data models for dbt nodes (models, tests, seeds, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Column metadata from dbt manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub data_type: Option<String>,
}

/// A dbt node (model, test, seed, snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub unique_id: String,
    pub name: String,
    pub resource_type: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub compiled_code: Option<String>,
    #[serde(default)]
    pub raw_code: Option<String>,
    #[serde(default)]
    pub depends_on: DependsOn,
    #[serde(default)]
    pub root_path: Option<String>,
    #[serde(default)]
    pub original_file_path: Option<String>,
    #[serde(default)]
    pub config: NodeConfig,
    #[serde(default)]
    pub compiled_path: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub columns: HashMap<String, ColumnInfo>,
}

/// Node configuration from dbt_project.yml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeConfig {
    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Dependency information for a node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependsOn {
    #[serde(default)]
    pub nodes: Vec<String>,
}

/// The dbt manifest.json structure
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub nodes: HashMap<String, Node>,
}

impl Node {
    /// Get an icon for the node type
    pub fn icon(&self) -> &str {
        match self.resource_type.as_str() {
            "model" => "󰕮",
            "test" => "󰙨",
            "seed" => "󰔛",
            "snapshot" => "󰸎",
            "source" => "󱃗",
            _ => "󰈔",
        }
    }

    /// Get a display name for the node (just the name)
    pub fn display_name(&self) -> String {
        self.name.clone()
    }

    /// Get the schema to use for grouping (prefers config.schema over schema)
    pub fn group_schema(&self) -> String {
        self.config
            .schema
            .clone()
            .unwrap_or_else(|| self.schema.clone())
    }

    /// Get the compiled SQL by reading from the compiled file
    pub fn get_compiled_sql(&self) -> Option<String> {
        if let (Some(root), Some(original_path)) = (&self.root_path, &self.original_file_path) {
            let mut path = PathBuf::from(root);
            path.push("target");
            path.push("compiled");
            path.push(&self.package_name);
            path.push(original_path);

            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return Some(content);
                }
            }
        }
        None
    }

    /// Get the raw SQL by reading from the source file directly (for live updates)
    pub fn get_raw_sql(&self) -> Option<String> {
        if let (Some(root), Some(original_path)) = (&self.root_path, &self.original_file_path) {
            let mut path = PathBuf::from(root);
            path.push(original_path);

            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return Some(content);
                }
            }
        }
        // Fall back to cached raw_code if file not found
        self.raw_code.clone()
    }

    /// Get the full path to the seed CSV file
    pub fn get_seed_path(&self) -> Option<PathBuf> {
        if self.resource_type != "seed" {
            return None;
        }

        match (&self.root_path, &self.original_file_path) {
            (Some(root), Some(file_path)) => {
                let mut path = PathBuf::from(root);
                path.push(file_path);
                Some(path)
            }
            _ => None,
        }
    }

    /// Read and parse the seed CSV file
    pub fn read_seed_data(&self) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        let path = self.get_seed_path().ok_or("No seed path available")?;

        if !path.exists() {
            return Err(format!("Seed file not found: {}", path.display()));
        }

        let file = fs::File::open(&path).map_err(|e| format!("Failed to open seed file: {}", e))?;
        let mut reader = csv::Reader::from_reader(file);

        let headers = reader
            .headers()
            .map_err(|e| format!("Failed to read CSV headers: {}", e))?
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| format!("Failed to read CSV record: {}", e))?;
            let row = record.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }

        Ok((headers, rows))
    }

    /// Get the YAML test definition for schema tests
    /// Returns the relevant YAML snippet showing the test configuration
    pub fn get_test_yaml_definition(&self, model_name: &str) -> Option<String> {
        if self.resource_type != "test" {
            return None;
        }

        // Get the schema file path from original_file_path
        let (root, original_path) = match (&self.root_path, &self.original_file_path) {
            (Some(r), Some(p)) => (r, p),
            _ => return None,
        };

        // Check if this is a YAML-defined test (schema test)
        if !original_path.ends_with(".yml") && !original_path.ends_with(".yaml") {
            return None;
        }

        let mut path = PathBuf::from(root);
        path.push(original_path);

        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;

        // Extract test type from the test name (e.g., "not_null" from "not_null_customers_id")
        let test_type = self.extract_test_type_from_name();

        // Try to find the model section and the specific test
        Self::extract_yaml_test_section(&content, model_name, &test_type, &self.name)
    }

    /// Extract test type from test name
    fn extract_test_type_from_name(&self) -> String {
        let known_types = [
            "not_null",
            "unique",
            "accepted_values",
            "relationships",
            "dbt_expectations",
            "dbt_utils",
        ];

        for test_type in known_types {
            if self.name.starts_with(test_type) {
                return test_type.to_string();
            }
        }

        // Fallback: first word before underscore
        self.name.split('_').next().unwrap_or("").to_string()
    }

    /// Extract the relevant YAML section for a test
    /// Only extracts the specific column definition that contains the test
    fn extract_yaml_test_section(
        yaml_content: &str,
        model_name: &str,
        test_type: &str,
        full_test_name: &str,
    ) -> Option<String> {
        // Try to extract column name from test name
        // Pattern: {test_type}_{model_name}_{column_name} or {test_type}_{column_name}
        let column_name = Self::extract_column_name_from_test(full_test_name, test_type, model_name);

        let lines: Vec<&str> = yaml_content.lines().collect();
        let mut in_model_section = false;
        let mut in_target_column = false;
        let mut model_indent = 0;
        let mut column_indent = 0;
        let mut column_start = 0;
        let mut result = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let current_indent = line.len() - line.trim_start().len();

            // Look for model name definition
            if trimmed.starts_with("- name:") {
                let name_value = trimmed.trim_start_matches("- name:").trim();
                if name_value == model_name {
                    in_model_section = true;
                    model_indent = current_indent;
                    continue;
                } else if in_model_section && current_indent <= model_indent {
                    // Exited model section
                    break;
                }
            }

            if in_model_section {
                // Look for column definition
                if trimmed.starts_with("- name:") && current_indent > model_indent {
                    let col_name = trimmed.trim_start_matches("- name:").trim();

                    // If we were in a target column, we're done
                    if in_target_column {
                        break;
                    }

                    // Check if this is the column we're looking for
                    if let Some(ref target_col) = column_name {
                        if col_name == target_col {
                            in_target_column = true;
                            column_indent = current_indent;
                            column_start = i;
                            result.push(line.to_string());
                            continue;
                        }
                    }
                }

                // Collect lines for the target column
                if in_target_column {
                    // Check if we've exited the column section
                    if !trimmed.is_empty() && current_indent <= column_indent && i > column_start {
                        break;
                    }
                    result.push(line.to_string());
                }
            }
        }

        // If we didn't find a specific column, try to find just the test line
        if result.is_empty() {
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed == format!("- {}", test_type)
                    || trimmed.starts_with(&format!("- {}:", test_type))
                {
                    // Include context: a few lines before and after
                    let start = i.saturating_sub(5);
                    let end = (i + 5).min(lines.len());
                    for line in lines.iter().take(end).skip(start) {
                        result.push(line.to_string());
                    }
                    break;
                }
            }
        }

        if !result.is_empty() {
            Some(result.join("\n"))
        } else {
            None
        }
    }

    /// Extract column name from test name
    /// e.g., "not_null_customers_customer_id" -> Some("customer_id")
    fn extract_column_name_from_test(
        test_name: &str,
        test_type: &str,
        model_name: &str,
    ) -> Option<String> {
        // Remove test type prefix
        let without_type = test_name.strip_prefix(test_type)?.trim_start_matches('_');

        // Remove model name if present
        let without_model = if without_type.starts_with(model_name) {
            without_type
                .strip_prefix(model_name)?
                .trim_start_matches('_')
        } else {
            without_type
        };

        if without_model.is_empty() {
            None
        } else {
            Some(without_model.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_display_name() {
        let node = Node {
            unique_id: "model.analytics.customers".to_string(),
            name: "customers".to_string(),
            resource_type: "model".to_string(),
            package_name: "analytics".to_string(),
            schema: "analytics".to_string(),
            compiled_code: None,
            raw_code: None,
            depends_on: DependsOn::default(),
            root_path: None,
            original_file_path: None,
            config: NodeConfig::default(),
            compiled_path: None,
            description: None,
            columns: HashMap::new(),
        };

        assert_eq!(node.display_name(), "customers");
    }

    #[test]
    fn test_extract_column_name_from_test() {
        // Test with model name in test name
        assert_eq!(
            Node::extract_column_name_from_test("not_null_customers_customer_id", "not_null", "customers"),
            Some("customer_id".to_string())
        );

        // Test with unique test
        assert_eq!(
            Node::extract_column_name_from_test("unique_orders_order_id", "unique", "orders"),
            Some("order_id".to_string())
        );

        // Test without model name prefix
        assert_eq!(
            Node::extract_column_name_from_test("not_null_email", "not_null", "users"),
            Some("email".to_string())
        );
    }

    #[test]
    fn test_extract_yaml_test_section() {
        let yaml = r#"
version: 2

models:
  - name: customers
    description: Customer table
    columns:
      - name: customer_id
        description: Primary key
        tests:
          - unique
          - not_null
      - name: email
        tests:
          - not_null
"#;

        let result = Node::extract_yaml_test_section(yaml, "customers", "not_null", "not_null_customers_customer_id");
        assert!(result.is_some());
        let content = result.unwrap();
        assert!(content.contains("customer_id"));
        assert!(content.contains("not_null"));
    }
}
