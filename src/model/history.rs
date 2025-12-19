//! Data models for run history persistence

use super::run::RunStatus;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// A single entry in the run history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHistoryEntry {
    pub timestamp: DateTime<Local>,
    pub command: String,
    pub status: RunStatus,
    pub output: String,
    pub duration_secs: f64,
}

impl RunHistoryEntry {
    pub fn status_icon(&self) -> &str {
        match self.status {
            RunStatus::Running => "⏳",
            RunStatus::Success => "✓",
            RunStatus::Failed => "✗",
        }
    }

    pub fn formatted_time(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }

    pub fn formatted_duration(&self) -> String {
        if self.duration_secs < 60.0 {
            format!("{:.1}s", self.duration_secs)
        } else {
            let mins = (self.duration_secs / 60.0).floor();
            let secs = self.duration_secs % 60.0;
            format!("{}m {:.0}s", mins, secs)
        }
    }
}

/// Wrapper for persisting run history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHistory {
    pub entries: Vec<RunHistoryEntry>,
}

impl RunHistory {
    fn history_dir() -> Option<PathBuf> {
        let home = env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".dbt-tui"))
    }

    fn history_path() -> Option<PathBuf> {
        Self::history_dir().map(|dir| dir.join("history.json"))
    }

    pub fn load() -> Vec<RunHistoryEntry> {
        let history_path = match Self::history_path() {
            Some(p) => p,
            None => return Vec::new(),
        };

        if !history_path.exists() {
            return Vec::new();
        }

        let contents = match fs::read_to_string(&history_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        match serde_json::from_str::<RunHistory>(&contents) {
            Ok(history) => history.entries,
            Err(_) => Vec::new(),
        }
    }

    pub fn save(entries: &[RunHistoryEntry]) -> Result<(), String> {
        let history_dir = Self::history_dir().ok_or("Could not determine home directory")?;

        if !history_dir.exists() {
            fs::create_dir_all(&history_dir)
                .map_err(|e| format!("Failed to create history directory: {}", e))?;
        }

        let history_path = Self::history_path().ok_or("Could not determine history path")?;

        let history = RunHistory {
            entries: entries.to_vec(),
        };

        let json = serde_json::to_string_pretty(&history)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;

        fs::write(&history_path, json)
            .map_err(|e| format!("Failed to write history file: {}", e))?;

        Ok(())
    }
}
