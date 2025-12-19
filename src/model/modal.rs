//! Modal stack for managing overlays
//!
//! Replaces the multiple boolean flags (show_quit_confirm, show_run_options, etc.)
//! with a proper state machine using an enum-based modal stack.

/// Represents a modal overlay that can be displayed on top of the main UI
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    /// Quit confirmation dialog
    QuitConfirm,
    /// Run options selection dialog (legacy, will be replaced by CommandMenu)
    RunOptions { selected_index: usize },
    /// Project information overlay
    ProjectInfo,
    /// dbt run output display
    RunOutput,
    /// Run history list and detail view
    History {
        selected_index: usize,
        detail_scroll: usize,
    },
    /// Target selection dialog
    TargetSelector { selected_index: usize },
    /// Tag filter dialog
    TagFilter { selected_index: usize },
    /// Git diff view
    GitDiff { file_path: String },
    /// Git commit dialog
    GitCommit { message: String },
    /// Git log view
    GitLog { scroll_offset: usize },
    /// Sample data preview dialog (dbt show)
    SampleData {
        model_name: String,
        scroll_offset: usize,
    },
    /// Help dialog showing all keyboard shortcuts
    Help { scroll_offset: usize },
}

/// A stack of modal overlays
///
/// Modals are rendered from bottom to top, with only the top modal
/// receiving input events.
#[derive(Debug, Default)]
pub struct ModalStack {
    stack: Vec<Modal>,
}

impl ModalStack {
    /// Create a new empty modal stack
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Push a modal onto the stack
    pub fn push(&mut self, modal: Modal) {
        self.stack.push(modal);
    }

    /// Pop the top modal from the stack
    pub fn pop(&mut self) -> Option<Modal> {
        self.stack.pop()
    }

    /// Get a reference to the top modal without removing it
    pub fn top(&self) -> Option<&Modal> {
        self.stack.last()
    }

    /// Get a mutable reference to the top modal
    pub fn top_mut(&mut self) -> Option<&mut Modal> {
        self.stack.last_mut()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_stack_push_pop() {
        let mut stack = ModalStack::new();
        assert!(stack.top().is_none());

        stack.push(Modal::QuitConfirm);
        assert!(stack.top().is_some());

        stack.push(Modal::ProjectInfo);

        let top = stack.pop();
        assert_eq!(top, Some(Modal::ProjectInfo));

        let top = stack.pop();
        assert_eq!(top, Some(Modal::QuitConfirm));
        assert!(stack.top().is_none());
    }

    #[test]
    fn test_modal_stack_top() {
        let mut stack = ModalStack::new();
        assert!(stack.top().is_none());

        stack.push(Modal::QuitConfirm);
        assert_eq!(stack.top(), Some(&Modal::QuitConfirm));

        stack.push(Modal::RunOptions { selected_index: 0 });
        assert_eq!(stack.top(), Some(&Modal::RunOptions { selected_index: 0 }));
    }

    #[test]
    fn test_modal_stack_top_mut() {
        let mut stack = ModalStack::new();
        stack.push(Modal::RunOptions { selected_index: 0 });

        if let Some(Modal::RunOptions { selected_index }) = stack.top_mut() {
            *selected_index = 2;
        }

        assert_eq!(stack.top(), Some(&Modal::RunOptions { selected_index: 2 }));
    }
}
