//! Action enum - All possible application actions
//!
//! Actions are discrete operations that the application can perform.
//! Components emit Actions in response to events, and the App processes
//! them to update state.

use std::fmt;

/// All possible actions in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // ─────────────────────────────────────────────────────────────────────────
    // App Lifecycle
    // ─────────────────────────────────────────────────────────────────────────
    /// Regular tick for animations/updates
    Tick,
    /// Terminal was resized
    Resize(u16, u16),
    /// Force quit without confirmation
    ForceQuit,
    /// Transition from splash to main app
    SplashComplete,

    // ─────────────────────────────────────────────────────────────────────────
    // Navigation
    // ─────────────────────────────────────────────────────────────────────────
    /// Move to next item in list
    NextItem,
    /// Move to previous item in list
    PrevItem,
    /// Move to next tab
    NextTab,
    /// Move to previous tab
    PrevTab,
    /// Jump to first item
    FirstItem,
    /// Jump to last item
    LastItem,

    // ─────────────────────────────────────────────────────────────────────────
    // Scrolling
    // ─────────────────────────────────────────────────────────────────────────
    /// Scroll detail panel up one line
    ScrollUp,
    /// Scroll detail panel down one line
    ScrollDown,
    /// Scroll detail panel up one page
    PageUp,
    /// Scroll detail panel down one page
    PageDown,

    // ─────────────────────────────────────────────────────────────────────────
    // Modals
    // ─────────────────────────────────────────────────────────────────────────
    /// Open quit confirmation dialog
    OpenQuitDialog,
    /// Open run options dialog
    OpenRunOptions,
    /// Open project info overlay
    OpenProjectInfo,
    /// Open run history overlay
    OpenHistory,
    /// Open run output overlay
    OpenRunOutput,
    /// Close the current modal
    CloseModal,
    /// Confirm the current modal action
    ConfirmModal,
    /// Navigate up in modal (e.g., previous option)
    ModalUp,
    /// Navigate down in modal (e.g., next option)
    ModalDown,

    // ─────────────────────────────────────────────────────────────────────────
    // View Toggles
    // ─────────────────────────────────────────────────────────────────────────
    /// Toggle between compiled and original SQL
    ToggleCodeView,
    /// Toggle lineage panel visibility
    ToggleLineage,
    /// Toggle documentation panel visibility
    ToggleDocumentation,
    /// Toggle run output view mode (raw/graphical)
    ToggleOutputView,

    // ─────────────────────────────────────────────────────────────────────────
    // Search
    // ─────────────────────────────────────────────────────────────────────────
    /// Enter search mode
    EnterSearchMode,
    /// Exit search mode
    ExitSearchMode,
    /// Add character to search query
    SearchInput(char),
    /// Remove last character from search query
    SearchBackspace,

    // ─────────────────────────────────────────────────────────────────────────
    // Selection
    // ─────────────────────────────────────────────────────────────────────────
    /// Toggle selection of current node for bulk operations
    ToggleNodeSelection,
    /// Clear all node selections
    ClearSelection,
    /// Select all visible nodes
    SelectAllNodes,

    // ─────────────────────────────────────────────────────────────────────────
    // Tag Filter
    // ─────────────────────────────────────────────────────────────────────────
    /// Open tag filter dialog
    OpenTagFilter,
    /// Set tag filter
    SetTagFilter(String),
    /// Clear tag filter
    ClearTagFilter,

    // ─────────────────────────────────────────────────────────────────────────
    // Project Management
    // ─────────────────────────────────────────────────────────────────────────
    /// Refresh/reload the manifest.json
    RefreshManifest,
    /// Run dbt compile to generate manifest.json
    CompileManifest,
    /// Open target selection dialog
    OpenTargetSelector,

    // ─────────────────────────────────────────────────────────────────────────
    // Setup Wizard
    // ─────────────────────────────────────────────────────────────────────────
    /// Confirm setup configuration
    SetupConfirm,

    // ─────────────────────────────────────────────────────────────────────────
    // Editor
    // ─────────────────────────────────────────────────────────────────────────
    /// Open current file in external $EDITOR
    OpenEditor,

    // ─────────────────────────────────────────────────────────────────────────
    // Sample Data
    // ─────────────────────────────────────────────────────────────────────────
    /// Open sample data preview (dbt show)
    OpenSampleData,
    /// Open help dialog showing all keyboard shortcuts
    OpenHelp,

    // ─────────────────────────────────────────────────────────────────────────
    // Git Operations
    // ─────────────────────────────────────────────────────────────────────────
    /// Open git diff view for current file
    OpenGitDiff,
    /// Stage current file
    GitStageFile,
    /// Open git commit dialog
    OpenGitCommit,
    /// Confirm git commit with message
    GitCommit(String),
    /// Open git log view
    OpenGitLog,
    /// Refresh git status
    RefreshGitStatus,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Tick => write!(f, "Tick"),
            Action::Resize(w, h) => write!(f, "Resize({}, {})", w, h),
            Action::ForceQuit => write!(f, "ForceQuit"),
            Action::SplashComplete => write!(f, "SplashComplete"),
            Action::NextItem => write!(f, "NextItem"),
            Action::PrevItem => write!(f, "PrevItem"),
            Action::NextTab => write!(f, "NextTab"),
            Action::PrevTab => write!(f, "PrevTab"),
            Action::FirstItem => write!(f, "FirstItem"),
            Action::LastItem => write!(f, "LastItem"),
            Action::ScrollUp => write!(f, "ScrollUp"),
            Action::ScrollDown => write!(f, "ScrollDown"),
            Action::PageUp => write!(f, "PageUp"),
            Action::PageDown => write!(f, "PageDown"),
            Action::OpenQuitDialog => write!(f, "OpenQuitDialog"),
            Action::OpenRunOptions => write!(f, "OpenRunOptions"),
            Action::OpenProjectInfo => write!(f, "OpenProjectInfo"),
            Action::OpenHistory => write!(f, "OpenHistory"),
            Action::OpenRunOutput => write!(f, "OpenRunOutput"),
            Action::CloseModal => write!(f, "CloseModal"),
            Action::ConfirmModal => write!(f, "ConfirmModal"),
            Action::ModalUp => write!(f, "ModalUp"),
            Action::ModalDown => write!(f, "ModalDown"),
            Action::ToggleCodeView => write!(f, "ToggleCodeView"),
            Action::ToggleLineage => write!(f, "ToggleLineage"),
            Action::ToggleDocumentation => write!(f, "ToggleDocumentation"),
            Action::ToggleOutputView => write!(f, "ToggleOutputView"),
            Action::EnterSearchMode => write!(f, "EnterSearchMode"),
            Action::ExitSearchMode => write!(f, "ExitSearchMode"),
            Action::SearchInput(c) => write!(f, "SearchInput('{}')", c),
            Action::SearchBackspace => write!(f, "SearchBackspace"),
            Action::ToggleNodeSelection => write!(f, "ToggleNodeSelection"),
            Action::ClearSelection => write!(f, "ClearSelection"),
            Action::SelectAllNodes => write!(f, "SelectAllNodes"),
            Action::OpenTagFilter => write!(f, "OpenTagFilter"),
            Action::SetTagFilter(tag) => write!(f, "SetTagFilter({})", tag),
            Action::ClearTagFilter => write!(f, "ClearTagFilter"),
            Action::RefreshManifest => write!(f, "RefreshManifest"),
            Action::CompileManifest => write!(f, "CompileManifest"),
            Action::OpenTargetSelector => write!(f, "OpenTargetSelector"),
            Action::SetupConfirm => write!(f, "SetupConfirm"),
            Action::OpenEditor => write!(f, "OpenEditor"),
            Action::OpenSampleData => write!(f, "OpenSampleData"),
            Action::OpenHelp => write!(f, "OpenHelp"),
            Action::OpenGitDiff => write!(f, "OpenGitDiff"),
            Action::GitStageFile => write!(f, "GitStageFile"),
            Action::OpenGitCommit => write!(f, "OpenGitCommit"),
            Action::GitCommit(msg) => write!(f, "GitCommit({})", msg),
            Action::OpenGitLog => write!(f, "OpenGitLog"),
            Action::RefreshGitStatus => write!(f, "RefreshGitStatus"),
        }
    }
}
