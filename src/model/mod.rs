//! Model layer - centralized state management
//!
//! This module contains all state-related types:
//! - `DomainState` - Business/data state (nodes, lineage, history)
//! - `UiState` - Presentation state (tabs, scroll, selections)
//! - `ModalStack` - Modal overlay management

pub mod domain;
pub mod history;
pub mod lineage;
pub mod modal;
pub mod node;
pub mod run;
pub mod sample_data;
pub mod ui;

// Re-export commonly used types
pub use domain::ProjectInfo;
pub use history::RunHistoryEntry;
pub use node::{Manifest, Node};
pub use run::{
    DbtCommand, ModelRun, ModelRunStatus, RunFlags, RunOutput, RunOutputViewMode,
    RunSelectMode, RunStatus,
};
pub use sample_data::SampleDataOutput;
pub use ui::CodeViewMode;
