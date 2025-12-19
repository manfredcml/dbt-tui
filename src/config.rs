use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project_path: String,
    pub dbt_binary_path: String,
    pub profile: String,
    pub target: String,
    /// Available targets for the project (dev, prod, staging, etc.)
    #[serde(default)]
    pub available_targets: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_path: String::new(),
            dbt_binary_path: String::new(),
            profile: "default".to_string(),
            target: "dev".to_string(),
            available_targets: vec![
                "dev".to_string(),
                "prod".to_string(),
                "staging".to_string(),
                "test".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn config_dir() -> Option<PathBuf> {
        let home = env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".dbt-tui"))
    }

    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.json"))
    }

    pub fn load() -> Option<Config> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&config_path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Save the config to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = Self::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }
}
