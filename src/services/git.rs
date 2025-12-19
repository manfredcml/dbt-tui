//! Git service for repository operations
//!
//! Provides git functionality for version control integration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git file status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitFileStatus {
    #[default]
    Untracked,
    Modified,
    Staged,
    StagedModified, // Staged with additional modifications
    Deleted,
    StagedDeleted,
    Renamed,
    Copied,
    Ignored,
}

/// Overall git repository status
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    /// Whether the repo has uncommitted changes
    pub is_dirty: bool,
    /// File statuses by relative path
    pub files: HashMap<String, GitFileStatus>,
}

/// A single commit entry
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// Check if a path is inside a git repository
pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the current branch name
pub fn get_branch(project_path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            // Detached HEAD state - get short commit hash
            let hash_output = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .current_dir(project_path)
                .output()
                .map_err(|e| format!("Failed to get HEAD: {}", e))?;

            if hash_output.status.success() {
                let hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();
                Ok(format!("HEAD@{}", hash))
            } else {
                Ok("(unknown)".to_string())
            }
        } else {
            Ok(branch)
        }
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Get the git repository root directory
fn get_git_root(project_path: &Path) -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to get git root: {}", e))?;

    if output.status.success() {
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(root))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Get the overall git status for the repository
pub fn get_status(project_path: &Path) -> Result<GitStatus, String> {
    let branch = get_branch(project_path).unwrap_or_else(|_| "(unknown)".to_string());

    // Get the relative path from git root to project_path
    // This is needed because git status returns paths relative to git root,
    // but we need paths relative to the dbt project root
    let prefix_to_strip = get_git_root(project_path)
        .ok()
        .and_then(|git_root| {
            project_path.strip_prefix(&git_root).ok().map(|p| {
                let s = p.to_string_lossy().to_string();
                if s.is_empty() { s } else { format!("{}/", s) }
            })
        })
        .unwrap_or_default();

    // Use porcelain format for machine-readable output
    let output = Command::new("git")
        .args(["status", "--porcelain=v1"])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = HashMap::new();

    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }

        let index_status = line.chars().next().unwrap_or(' ');
        let work_tree_status = line.chars().nth(1).unwrap_or(' ');
        let file_path = line[3..].to_string();

        // Handle renamed files (format: "R  old -> new")
        let file_path = if file_path.contains(" -> ") {
            file_path.split(" -> ").last().unwrap_or(&file_path).to_string()
        } else {
            file_path
        };

        // Strip the prefix to make paths relative to dbt project root
        let file_path = if !prefix_to_strip.is_empty() && file_path.starts_with(&prefix_to_strip) {
            file_path[prefix_to_strip.len()..].to_string()
        } else {
            file_path
        };

        let status = match (index_status, work_tree_status) {
            ('?', '?') => GitFileStatus::Untracked,
            ('!', '!') => GitFileStatus::Ignored,
            ('A', ' ') | ('M', ' ') | ('D', ' ') => {
                if index_status == 'D' {
                    GitFileStatus::StagedDeleted
                } else {
                    GitFileStatus::Staged
                }
            }
            ('A', 'M') | ('M', 'M') => GitFileStatus::StagedModified,
            (' ', 'M') => GitFileStatus::Modified,
            (' ', 'D') => GitFileStatus::Deleted,
            ('R', _) => GitFileStatus::Renamed,
            ('C', _) => GitFileStatus::Copied,
            _ => GitFileStatus::Modified,
        };

        files.insert(file_path, status);
    }

    let is_dirty = !files.is_empty();

    Ok(GitStatus {
        branch,
        is_dirty,
        files,
    })
}

/// Stage a file for commit
pub fn stage_file(project_path: &Path, file_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", file_path])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git add: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Create a commit with the given message
pub fn commit(project_path: &Path, message: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git commit: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Get combined diff (both staged and unstaged) for a file
pub fn get_file_full_diff(project_path: &Path, file_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "HEAD", "--", file_path])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git diff HEAD: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Get commit log for the repository or a specific file
pub fn get_log(project_path: &Path, file_path: Option<&str>, limit: usize) -> Result<Vec<GitCommit>, String> {
    let mut args = vec![
        "log".to_string(),
        format!("-{}", limit),
        "--pretty=format:%h|%an|%ad|%s".to_string(),
        "--date=short".to_string(),
    ];

    if let Some(path) = file_path {
        args.push("--".to_string());
        args.push(path.to_string());
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run git log: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<GitCommit> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                Some(GitCommit {
                    short_hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    date: parts[2].to_string(),
                    message: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_file_status_default() {
        let status = GitFileStatus::default();
        assert_eq!(status, GitFileStatus::Untracked);
    }

    #[test]
    fn test_git_status_default() {
        let status = GitStatus::default();
        assert!(status.branch.is_empty());
        assert!(!status.is_dirty);
        assert!(status.files.is_empty());
    }
}
