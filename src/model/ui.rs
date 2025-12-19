//! UI state - presentation state separate from domain data
//!
//! Note: Most UI state has been moved to HomeComponent which owns presentation state.

/// Tab selection in the main UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Models,
    Tests,
    Seeds,
}

impl Tab {
    pub fn all() -> Vec<Tab> {
        vec![Tab::Models, Tab::Tests, Tab::Seeds]
    }

    pub fn name(&self) -> &str {
        match self {
            Tab::Models => "Models",
            Tab::Tests => "Tests",
            Tab::Seeds => "Seeds",
        }
    }

    pub fn resource_type(&self) -> Option<&str> {
        match self {
            Tab::Models => Some("model"),
            Tab::Tests => Some("test"),
            Tab::Seeds => Some("seed"),
        }
    }
}

/// Main application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Splash,
    Setup,
    Running,
}

/// SQL code view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeViewMode {
    Compiled,
    #[default]
    Original,
}

// UiState has been moved to HomeComponent which owns presentation state
