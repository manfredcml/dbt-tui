//! Background job runner service
//!
//! Handles spawning and monitoring dbt commands in the background.

use crate::model::run::{BackgroundJob, JobMessage, RunOutput, RunStatus};
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::sync::LazyLock;
use std::thread;
use std::time::Instant;

/// Regex to match ANSI escape codes
static ANSI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap()
});

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

/// Job runner service for executing dbt commands
pub struct JobRunner {
    /// Current background job (if any)
    job: Option<BackgroundJob>,
}

impl Default for JobRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl JobRunner {
    pub fn new() -> Self {
        Self { job: None }
    }

    /// Get the start instant of the current job
    pub fn start_instant(&self) -> Option<Instant> {
        self.job.as_ref().map(|j| j.start_instant)
    }

    /// Spawn a new background job
    pub fn spawn(&mut self, command: String) -> RunOutput {
        let (tx, rx) = mpsc::channel();
        let display_command = command.clone();

        thread::spawn(move || {
            Self::run_command(&command, tx);
        });

        self.job = Some(BackgroundJob {
            receiver: rx,
            start_instant: Instant::now(),
        });

        RunOutput::new(display_command)
    }

    /// Poll for job updates, returns true if there were updates
    pub fn poll(&self, run_output: &mut RunOutput) -> bool {
        let Some(ref job) = self.job else {
            return false;
        };

        let mut had_updates = false;

        loop {
            match job.receiver.try_recv() {
                Ok(JobMessage::Output(line)) => {
                    had_updates = true;
                    // Strip ANSI escape codes from output
                    let clean_line = strip_ansi_codes(&line);
                    run_output.output.push_str(&clean_line);
                    run_output.output.push('\n');
                    run_output.parse_output_line(&clean_line);
                }
                Ok(JobMessage::Completed(exit_code)) => {
                    had_updates = true;
                    run_output.status = if exit_code == Some(0) {
                        RunStatus::Success
                    } else {
                        RunStatus::Failed
                    };
                }
                Ok(JobMessage::Error(err)) => {
                    had_updates = true;
                    run_output.output.push_str(&format!("\nError: {}\n", err));
                    run_output.status = RunStatus::Failed;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    if run_output.status == RunStatus::Running {
                        run_output.status = RunStatus::Failed;
                    }
                    break;
                }
            }
        }

        had_updates
    }

    /// Clear the current job
    pub fn clear(&mut self) {
        self.job = None;
    }

    /// Run a shell command and send output through the channel
    fn run_command(command: &str, tx: Sender<JobMessage>) {
        #[cfg(target_os = "windows")]
        let result = Command::new("cmd")
            .args(["/C", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        #[cfg(not(target_os = "windows"))]
        let result = Command::new("sh")
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(JobMessage::Error(e.to_string()));
                return;
            }
        };

        // Read stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if tx.send(JobMessage::Output(line)).is_err() {
                    break;
                }
            }
        }

        // Wait for completion and send exit code
        let exit_code = child.wait().ok().and_then(|s| s.code());
        let _ = tx.send(JobMessage::Completed(exit_code));
    }
}
