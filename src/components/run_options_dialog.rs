//! Run options dialog component
//!
//! Three-section dialog:
//! 1. Command type (run, test, build, compile, deps)
//! 2. Selection mode (just this, upstream, downstream, etc.)
//! 3. Run flags (--full-refresh, --vars, --exclude)

use crate::action::Action;
use crate::component::Component;
use crate::components::centered_popup;
use crate::model::{DbtCommand, RunFlags, RunSelectMode};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Focus section in the run options dialog
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RunOptionsFocus {
    #[default]
    Command,
    SelectMode,
    Flags,
}

/// Run options dialog
pub struct RunOptionsDialog {
    /// Selected command (run, test, build, etc.)
    pub command: DbtCommand,
    /// Selected selection mode index
    pub mode_index: usize,
    /// Current focus section
    pub focus: RunOptionsFocus,
    /// Current run flags
    pub flags: RunFlags,
    /// Which flag is focused (0=full-refresh, 1=vars, 2=exclude)
    pub flag_index: usize,
    /// Whether editing vars input
    pub editing_vars: bool,
    /// Whether editing exclude input
    pub editing_exclude: bool,
}

impl Default for RunOptionsDialog {
    fn default() -> Self {
        Self {
            command: DbtCommand::Run,
            mode_index: 0,
            focus: RunOptionsFocus::Command,
            flags: RunFlags::default(),
            flag_index: 0,
            editing_vars: false,
            editing_exclude: false,
        }
    }
}

impl RunOptionsDialog {
    /// Reset dialog state for a new invocation
    pub fn reset(&mut self) {
        self.command = DbtCommand::Run;
        self.mode_index = 0;
        self.focus = RunOptionsFocus::Command;
        self.flags = RunFlags::default();
        self.flag_index = 0;
        self.editing_vars = false;
        self.editing_exclude = false;
    }

    fn handle_vars_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.editing_vars = false;
            }
            KeyCode::Backspace => {
                self.flags.vars.pop();
            }
            KeyCode::Char(c) => {
                self.flags.vars.push(c);
            }
            _ => {}
        }
        None
    }

    fn handle_exclude_input(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.editing_exclude = false;
            }
            KeyCode::Backspace => {
                self.flags.exclude.pop();
            }
            KeyCode::Char(c) => {
                self.flags.exclude.push(c);
            }
            _ => {}
        }
        None
    }

    /// Get available commands for display
    fn commands() -> &'static [DbtCommand] {
        &[
            DbtCommand::Run,
            DbtCommand::Test,
            DbtCommand::Build,
            DbtCommand::Compile,
            DbtCommand::Deps,
        ]
    }

    fn command_index(&self) -> usize {
        Self::commands()
            .iter()
            .position(|c| *c == self.command)
            .unwrap_or(0)
    }

    fn set_command_by_index(&mut self, index: usize) {
        if let Some(cmd) = Self::commands().get(index) {
            self.command = *cmd;
        }
    }

    pub fn draw_with_node_name(
        &self,
        frame: &mut Frame,
        area: Rect,
        node_name: &str,
    ) -> Result<()> {
        // Determine if selection mode should be shown
        let show_select_mode = self.command.supports_select();
        let height = if show_select_mode { 24u16 } else { 18u16 };
        let popup_area = centered_popup(area, 58, height);

        frame.render_widget(Clear, popup_area);

        let mut content = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Target: {}", node_name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // Command section
        let cmd_header_style = if self.focus == RunOptionsFocus::Command {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        content.push(Line::from(Span::styled(
            "─ Command ─",
            cmd_header_style,
        )));

        for (i, cmd) in Self::commands().iter().enumerate() {
            let is_selected = *cmd == self.command;
            let prefix = if is_selected && self.focus == RunOptionsFocus::Command {
                "▶ "
            } else if is_selected {
                "● "
            } else {
                "  "
            };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            content.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", cmd.shortcut()),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{}{:<12}", prefix, cmd.label()), style),
                Span::styled(cmd.description(), Style::default().fg(Color::DarkGray)),
            ]));

            // Add visual separator after compile (before deps which is project-wide)
            if i == 3 {
                content.push(Line::from(""));
            }
        }

        content.push(Line::from(""));

        // Selection mode section (only for commands that support it)
        if show_select_mode {
            let mode_header_style = if self.focus == RunOptionsFocus::SelectMode {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            content.push(Line::from(Span::styled(
                "─ Selection Mode ─",
                mode_header_style,
            )));

            let modes = RunSelectMode::all();
            for (i, mode) in modes.iter().enumerate() {
                let is_selected = i == self.mode_index;
                let prefix = if is_selected && self.focus == RunOptionsFocus::SelectMode {
                    "▶ "
                } else if is_selected {
                    "● "
                } else {
                    "  "
                };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                content.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", mode.shortcut()),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("{}{}", prefix, mode.label()), style),
                ]));
            }

            content.push(Line::from(""));
        }

        // Flags section (only for commands that support flags)
        if self.command != DbtCommand::Deps {
            let flags_header_style = if self.focus == RunOptionsFocus::Flags {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            content.push(Line::from(Span::styled(
                "─ Flags (Tab to switch) ─",
                flags_header_style,
            )));

            // Full refresh flag (only for run/build)
            if matches!(self.command, DbtCommand::Run | DbtCommand::Build) {
                let fr_prefix = if self.focus == RunOptionsFocus::Flags && self.flag_index == 0 {
                    "▶ "
                } else {
                    "  "
                };
                let fr_checkbox = if self.flags.full_refresh {
                    "[x]"
                } else {
                    "[ ]"
                };
                let fr_style = if self.flags.full_refresh {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                content.push(Line::from(vec![
                    Span::styled(
                        " F ",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(fr_prefix),
                    Span::styled(fr_checkbox, fr_style),
                    Span::raw(" --full-refresh"),
                ]));
            }

            // Vars input
            let vars_prefix = if self.focus == RunOptionsFocus::Flags && self.flag_index == 1 {
                "▶ "
            } else {
                "  "
            };
            let vars_display = if self.flags.vars.is_empty() {
                if self.editing_vars {
                    "_".to_string()
                } else {
                    "(none)".to_string()
                }
            } else if self.editing_vars {
                format!("{}_", self.flags.vars)
            } else {
                self.flags.vars.clone()
            };
            let vars_style = if self.editing_vars {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if !self.flags.vars.is_empty() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            content.push(Line::from(vec![
                Span::raw("   "),
                Span::raw(vars_prefix),
                Span::raw("--vars "),
                Span::styled(vars_display, vars_style),
            ]));

            // Exclude input
            let excl_prefix = if self.focus == RunOptionsFocus::Flags && self.flag_index == 2 {
                "▶ "
            } else {
                "  "
            };
            let excl_display = if self.flags.exclude.is_empty() {
                if self.editing_exclude {
                    "_".to_string()
                } else {
                    "(none)".to_string()
                }
            } else if self.editing_exclude {
                format!("{}_", self.flags.exclude)
            } else {
                self.flags.exclude.clone()
            };
            let excl_style = if self.editing_exclude {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if !self.flags.exclude.is_empty() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            content.push(Line::from(vec![
                Span::raw("   "),
                Span::raw(excl_prefix),
                Span::raw("--exclude "),
                Span::styled(excl_display, excl_style),
            ]));

            content.push(Line::from(""));
        }

        // Help bar
        let help_spans = if self.editing_vars || self.editing_exclude {
            vec![
                Span::styled(
                    " Enter/Esc ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Done editing"),
            ]
        } else {
            vec![
                Span::styled(
                    " Tab ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Switch  "),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Execute  "),
                Span::styled(
                    " Esc ",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Cancel"),
            ]
        };
        content.push(Line::from(help_spans));

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Run Options ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        );

        frame.render_widget(paragraph, popup_area);
        Ok(())
    }

    /// Get the selected command
    pub fn get_command(&self) -> DbtCommand {
        self.command
    }

    /// Get the selected mode
    pub fn get_mode(&self) -> RunSelectMode {
        RunSelectMode::all()[self.mode_index]
    }

    /// Get a copy of the current flags
    pub fn get_flags(&self) -> RunFlags {
        self.flags.clone()
    }
}

impl Component for RunOptionsDialog {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // Handle text input modes
        if self.editing_vars {
            return Ok(self.handle_vars_input(key));
        }
        if self.editing_exclude {
            return Ok(self.handle_exclude_input(key));
        }

        let action = match key.code {
            KeyCode::Tab => {
                // Cycle through sections
                self.focus = match self.focus {
                    RunOptionsFocus::Command => {
                        if self.command.supports_select() {
                            RunOptionsFocus::SelectMode
                        } else if self.command != DbtCommand::Deps {
                            RunOptionsFocus::Flags
                        } else {
                            RunOptionsFocus::Command
                        }
                    }
                    RunOptionsFocus::SelectMode => {
                        if self.command != DbtCommand::Deps {
                            RunOptionsFocus::Flags
                        } else {
                            RunOptionsFocus::Command
                        }
                    }
                    RunOptionsFocus::Flags => RunOptionsFocus::Command,
                };
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.focus {
                    RunOptionsFocus::Command => {
                        let idx = self.command_index();
                        if idx < Self::commands().len() - 1 {
                            self.set_command_by_index(idx + 1);
                        }
                    }
                    RunOptionsFocus::SelectMode => {
                        if self.mode_index < 3 {
                            self.mode_index += 1;
                        }
                    }
                    RunOptionsFocus::Flags => {
                        if self.flag_index < 2 {
                            self.flag_index += 1;
                        }
                    }
                }
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.focus {
                    RunOptionsFocus::Command => {
                        let idx = self.command_index();
                        if idx > 0 {
                            self.set_command_by_index(idx - 1);
                        }
                    }
                    RunOptionsFocus::SelectMode => {
                        if self.mode_index > 0 {
                            self.mode_index -= 1;
                        }
                    }
                    RunOptionsFocus::Flags => {
                        if self.flag_index > 0 {
                            self.flag_index -= 1;
                        }
                    }
                }
                None
            }
            KeyCode::Char(' ') if self.focus == RunOptionsFocus::Flags => {
                match self.flag_index {
                    0 => self.flags.full_refresh = !self.flags.full_refresh,
                    1 => self.editing_vars = true,
                    2 => self.editing_exclude = true,
                    _ => {}
                }
                None
            }
            KeyCode::Enter => {
                if self.focus == RunOptionsFocus::Flags {
                    match self.flag_index {
                        1 => {
                            self.editing_vars = true;
                            return Ok(None);
                        }
                        2 => {
                            self.editing_exclude = true;
                            return Ok(None);
                        }
                        _ => {}
                    }
                }
                Some(Action::ConfirmModal)
            }
            KeyCode::Esc => Some(Action::CloseModal),

            // Command shortcuts
            KeyCode::Char('r') => {
                self.command = DbtCommand::Run;
                None
            }
            KeyCode::Char('t') => {
                self.command = DbtCommand::Test;
                None
            }
            KeyCode::Char('b') => {
                self.command = DbtCommand::Build;
                None
            }
            KeyCode::Char('c') => {
                self.command = DbtCommand::Compile;
                None
            }
            KeyCode::Char('d') => {
                self.command = DbtCommand::Deps;
                None
            }

            // Selection mode shortcuts (when in that section)
            KeyCode::Char('1') if self.focus == RunOptionsFocus::SelectMode => {
                self.mode_index = 0;
                None
            }
            KeyCode::Char('2') if self.focus == RunOptionsFocus::SelectMode => {
                self.mode_index = 1;
                None
            }
            KeyCode::Char('3') if self.focus == RunOptionsFocus::SelectMode => {
                self.mode_index = 2;
                None
            }
            KeyCode::Char('4') if self.focus == RunOptionsFocus::SelectMode => {
                self.mode_index = 3;
                None
            }

            // Flag shortcuts
            KeyCode::Char('F') => {
                if matches!(self.command, DbtCommand::Run | DbtCommand::Build) {
                    self.flags.full_refresh = !self.flags.full_refresh;
                }
                None
            }

            _ => None,
        };
        Ok(action)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ModalUp => {
                match self.focus {
                    RunOptionsFocus::Command => {
                        let idx = self.command_index();
                        if idx > 0 {
                            self.set_command_by_index(idx - 1);
                        }
                    }
                    RunOptionsFocus::SelectMode => {
                        if self.mode_index > 0 {
                            self.mode_index -= 1;
                        }
                    }
                    RunOptionsFocus::Flags => {
                        if self.flag_index > 0 {
                            self.flag_index -= 1;
                        }
                    }
                }
            }
            Action::ModalDown => {
                match self.focus {
                    RunOptionsFocus::Command => {
                        let idx = self.command_index();
                        if idx < Self::commands().len() - 1 {
                            self.set_command_by_index(idx + 1);
                        }
                    }
                    RunOptionsFocus::SelectMode => {
                        if self.mode_index < 3 {
                            self.mode_index += 1;
                        }
                    }
                    RunOptionsFocus::Flags => {
                        if self.flag_index < 2 {
                            self.flag_index += 1;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.draw_with_node_name(frame, area, "selected")
    }
}
