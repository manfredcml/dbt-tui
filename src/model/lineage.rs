//! Data models for lineage graph and dependency tracking

use super::node::Node;
use std::collections::HashMap;

/// Represents a node in the lineage graph with metadata
#[derive(Debug, Clone)]
pub struct LineageNode {
    pub name: String,
    pub resource_type: String,
}

impl LineageNode {
    pub fn from_node(node: &Node) -> Self {
        LineageNode {
            name: node.name.clone(),
            resource_type: node.resource_type.clone(),
        }
    }

    /// Parse a unique_id from sources or other references
    pub fn from_unique_id(unique_id: &str) -> Self {
        let parts: Vec<&str> = unique_id.split('.').collect();

        let (resource_type, name) = if parts.len() >= 3 {
            let resource_type = parts[0].to_string();
            let name = if resource_type == "source" && parts.len() >= 4 {
                parts[3].to_string()
            } else {
                parts[2..].join(".")
            };
            (resource_type, name)
        } else {
            ("unknown".to_string(), unique_id.to_string())
        };

        LineageNode {
            name,
            resource_type,
        }
    }

    /// Get an icon/prefix for the resource type
    pub fn icon(&self) -> &str {
        match self.resource_type.as_str() {
            "model" => "📊",
            "source" => "🗄️",
            "test" => "✓",
            "seed" => "🌱",
            "snapshot" => "📸",
            _ => "•",
        }
    }
}

/// Lineage graph containing all dependency relationships
#[derive(Clone)]
pub struct LineageGraph {
    upstream: HashMap<String, Vec<LineageNode>>,
    downstream: HashMap<String, Vec<LineageNode>>,
}

impl LineageGraph {
    /// Build a lineage graph from a list of nodes
    pub fn build(nodes: &[Node]) -> Self {
        let mut upstream: HashMap<String, Vec<LineageNode>> = HashMap::new();
        let mut downstream: HashMap<String, Vec<LineageNode>> = HashMap::new();

        // First pass: collect upstream dependencies
        for node in nodes {
            let mut upstream_nodes = Vec::new();
            for dep_id in &node.depends_on.nodes {
                upstream_nodes.push(LineageNode::from_unique_id(dep_id));
            }
            upstream.insert(node.unique_id.clone(), upstream_nodes);
        }

        // Second pass: build downstream dependencies
        for node in nodes {
            for dep_id in &node.depends_on.nodes {
                downstream
                    .entry(dep_id.clone())
                    .or_default()
                    .push(LineageNode::from_node(node));
            }
        }

        LineageGraph {
            upstream,
            downstream,
        }
    }

    /// Get upstream dependencies (what this node depends on)
    pub fn get_upstream(&self, unique_id: &str) -> Vec<LineageNode> {
        self.upstream
            .get(unique_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get downstream dependencies (what depends on this node)
    pub fn get_downstream(&self, unique_id: &str) -> Vec<LineageNode> {
        self.downstream
            .get(unique_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{DependsOn, NodeConfig};

    fn create_test_node(
        unique_id: &str,
        name: &str,
        resource_type: &str,
        depends_on: Vec<String>,
    ) -> Node {
        Node {
            unique_id: unique_id.to_string(),
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            package_name: "test".to_string(),
            schema: "public".to_string(),
            compiled_code: None,
            raw_code: None,
            depends_on: DependsOn { nodes: depends_on },
            root_path: None,
            original_file_path: None,
            config: NodeConfig::default(),
            compiled_path: None,
            description: None,
            columns: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_lineage_node_from_unique_id() {
        let source = LineageNode::from_unique_id("source.analytics.raw.customers");
        assert_eq!(source.resource_type, "source");
        assert_eq!(source.name, "customers");

        let model = LineageNode::from_unique_id("model.analytics.stg_customers");
        assert_eq!(model.resource_type, "model");
        assert_eq!(model.name, "stg_customers");
    }

    #[test]
    fn test_lineage_graph_build() {
        let nodes = vec![
            create_test_node("model.test.a", "a", "model", vec![]),
            create_test_node(
                "model.test.b",
                "b",
                "model",
                vec!["model.test.a".to_string()],
            ),
            create_test_node(
                "model.test.c",
                "c",
                "model",
                vec!["model.test.b".to_string()],
            ),
        ];

        let graph = LineageGraph::build(&nodes);

        assert_eq!(graph.get_upstream("model.test.a").len(), 0);
        assert_eq!(graph.get_upstream("model.test.b").len(), 1);
        assert_eq!(graph.get_downstream("model.test.a").len(), 1);
    }
}
