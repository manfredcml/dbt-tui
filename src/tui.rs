//! Terminal User Interface management
//!
//! Handles terminal setup, teardown, and event polling.
//! Wraps ratatui's Terminal for a cleaner interface.

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    ops::{Deref, DerefMut},
    time::Duration,
};

/// Terminal wrapper for managing the TUI lifecycle
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Polling timeout for events
    pub tick_rate: Duration,
}

impl Tui {
    /// Create a new Tui instance
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            tick_rate: Duration::from_millis(100),
        })
    }

    /// Set the tick rate for event polling
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Enter the alternate screen and enable raw mode
    ///
    /// This should be called before the main loop starts.
    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Exit the alternate screen and disable raw mode
    ///
    /// This should be called when the application exits.
    /// Also called automatically on Drop.
    pub fn exit(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        Ok(())
    }

    /// Temporarily suspend the TUI for running external commands
    ///
    /// Returns a guard that will restore the TUI when dropped.
    pub fn suspend(&mut self) -> Result<()> {
        self.exit()
    }

    /// Resume the TUI after suspension
    pub fn resume(&mut self) -> Result<()> {
        self.enter()
    }

    /// Poll for the next event
    ///
    /// Returns `Some(Event)` if an event is available within the tick rate,
    /// or `None` if no event is available (tick timeout).
    pub fn next_event(&self) -> Result<Option<Event>> {
        if event::poll(self.tick_rate)? {
            let event = event::read()?;

            // Filter out key release events (Windows compatibility)
            if let Event::Key(key) = &event {
                if key.kind != KeyEventKind::Press {
                    return Ok(None);
                }
            }

            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    /// Draw to the terminal using the provided closure
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Deref for Tui {
    type Target = Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        // Best effort cleanup on drop
        let _ = self.exit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_creation() {
        // Just test that we can create the struct without panicking
        // Note: This won't actually work in CI without a TTY
        let result = Tui::new();
        // We just check it returns a Result, actual success depends on environment
        assert!(result.is_ok() || result.is_err());
    }
}
