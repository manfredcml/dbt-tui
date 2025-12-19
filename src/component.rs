//! Component trait - Interface for UI components
//!
//! Each component encapsulates its own state, event handling, and rendering logic.
//! Components communicate through Actions rather than direct state mutation.

use crate::action::Action;
use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};

/// Trait for UI components
///
/// Components are self-contained units that:
/// - Handle their own key/mouse events
/// - Maintain local state
/// - Render themselves to a frame
///
/// The pattern follows:
/// 1. `handle_key_event` / `handle_mouse_event` - Convert events to Actions
/// 2. `update` - Process Actions and update state
/// 3. `draw` - Render the component
pub trait Component {
    /// Initialize the component
    ///
    /// Called once when the component is created. Use this to set up
    /// initial state that depends on runtime information.
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Handle a key event, returning an optional Action
    ///
    /// This method converts key events into semantic Actions.
    /// The component should not modify state here - just return
    /// the appropriate Action.
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let _ = key;
        Ok(None)
    }

    /// Handle a mouse event, returning an optional Action
    ///
    /// Similar to handle_key_event but for mouse interactions.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let _ = mouse;
        Ok(None)
    }

    /// Update component state based on an Action
    ///
    /// This is where state changes happen. The method can optionally
    /// return a new Action if the update should trigger another action
    /// (e.g., closing a modal might return Render).
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }

    /// Draw the component to the frame
    ///
    /// This method should be pure rendering - no state changes.
    /// Use the provided `area` to determine where to draw.
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
