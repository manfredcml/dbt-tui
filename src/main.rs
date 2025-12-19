//! dbt-tui - A terminal UI for dbt
//!
//! This is the main entry point for the dbt-tui application.
//! It uses the Component Architecture pattern from ratatui.

mod action;
mod app;
mod component;
mod components;
mod config;
mod model;
mod services;
mod tui;

use crate::action::Action;
use crate::app::App;
use crate::component::Component;
use crate::tui::Tui;
use anyhow::Result;
use crossterm::event::Event;
use std::process::Command;
use std::time::Duration;

fn main() -> Result<()> {
    // Setup terminal
    let mut tui = Tui::new()?.with_tick_rate(Duration::from_millis(100));
    tui.enter()?;

    // Create app state
    let mut app = App::new();
    app.init()?;

    // Main event loop
    let result = run_app(&mut tui, &mut app);

    // Cleanup terminal
    tui.exit()?;

    // Handle any errors
    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}

/// Run the main application loop
fn run_app(tui: &mut Tui, app: &mut App) -> Result<()> {
    while !app.should_quit {
        // Draw the UI
        tui.draw(|frame| {
            if let Err(e) = app.draw(frame, frame.area()) {
                eprintln!("Draw error: {}", e);
            }
        })?;

        // Check for pending external editor
        if let Some(file_path) = app.pending_editor_file.take() {
            launch_external_editor(tui, app, &file_path)?;
            continue; // Redraw after editor closes
        }

        // Poll for events
        if let Some(event) = tui.next_event()? {
            // Convert event to action
            let action = match event {
                Event::Key(key) => app.handle_key_event(key)?,
                Event::Mouse(mouse) => app.handle_mouse_event(mouse)?,
                Event::Resize(w, h) => Some(Action::Resize(w, h)),
                _ => None,
            };

            // Process the action
            if let Some(action) = action {
                // Action might produce a follow-up action
                let mut current_action = Some(action);
                while let Some(a) = current_action {
                    current_action = app.update(a)?;
                }
            }
        } else {
            // No event - send a tick for time-based updates
            app.update(Action::Tick)?;
        }
    }

    Ok(())
}

/// Launch an external editor for the given file
fn launch_external_editor(tui: &mut Tui, app: &mut App, file_path: &str) -> Result<()> {
    // Determine the editor to use: $VISUAL, $EDITOR, or fallback
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vim".to_string());

    // Suspend the TUI
    tui.suspend()?;

    // Launch the editor
    let status = Command::new(&editor)
        .arg(file_path)
        .status();

    // Resume the TUI
    tui.resume()?;

    // Handle result
    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                app.error = Some(format!("Editor exited with status: {}", exit_status));
            }
        }
        Err(e) => {
            app.error = Some(format!("Failed to launch editor '{}': {}", editor, e));
        }
    }

    // Refresh git status after editing
    app.update(Action::RefreshGitStatus)?;

    Ok(())
}
