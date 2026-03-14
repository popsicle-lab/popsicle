use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{PopsicleError, Result};

/// Metadata about a single git commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub branch: String,
}

/// A link between a git commit and a Popsicle document/pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLink {
    pub sha: String,
    pub doc_id: Option<String>,
    pub pipeline_run_id: String,
    pub stage: Option<String>,
    pub skill: Option<String>,
    pub review_status: ReviewStatus,
    pub review_summary: Option<String>,
    pub linked_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Passed,
    Failed,
    Skipped,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Passed => write!(f, "passed"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

/// Reads git information from a repository.
pub struct GitTracker;

impl GitTracker {
    /// Get info about a specific commit.
    pub fn commit_info(repo_dir: &Path, sha: &str) -> Result<CommitInfo> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%H%n%h%n%s%n%an%n%aI", sha])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git log failed: {}", e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(PopsicleError::Storage(format!("git log failed: {}", err)));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = text.trim().lines().collect();
        if lines.len() < 5 {
            return Err(PopsicleError::Storage("Unexpected git log output".into()));
        }

        let branch = Self::current_branch(repo_dir).unwrap_or_else(|_| "unknown".into());

        Ok(CommitInfo {
            sha: lines[0].to_string(),
            short_sha: lines[1].to_string(),
            message: lines[2].to_string(),
            author: lines[3].to_string(),
            timestamp: lines[4].to_string(),
            branch,
        })
    }

    /// Get the current branch name.
    pub fn current_branch(repo_dir: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git rev-parse failed: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// List recent commits (up to `count`).
    pub fn recent_commits(repo_dir: &Path, count: usize) -> Result<Vec<CommitInfo>> {
        let output = Command::new("git")
            .args([
                "log",
                &format!("-{}", count),
                "--format=%H|||%h|||%s|||%an|||%aI",
            ])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git log failed: {}", e)))?;

        let text = String::from_utf8_lossy(&output.stdout);
        let branch = Self::current_branch(repo_dir).unwrap_or_else(|_| "unknown".into());

        let commits: Vec<CommitInfo> = text
            .trim()
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(5, "|||").collect();
                if parts.len() >= 5 {
                    Some(CommitInfo {
                        sha: parts[0].to_string(),
                        short_sha: parts[1].to_string(),
                        message: parts[2].to_string(),
                        author: parts[3].to_string(),
                        timestamp: parts[4].to_string(),
                        branch: branch.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(commits)
    }

    /// Get the latest commit SHA.
    pub fn head_sha(repo_dir: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git rev-parse failed: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if the working tree has uncommitted changes.
    pub fn has_uncommitted_changes(repo_dir: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git status failed: {}", e)))?;

        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    /// Get files changed in a specific commit.
    pub fn changed_files(repo_dir: &Path, sha: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["diff-tree", "--no-commit-id", "--name-only", "-r", sha])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git diff-tree failed: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout)
            .trim()
            .lines()
            .map(|l| l.to_string())
            .collect())
    }

    /// Count total lines changed in a file since `since_date` (YYYY-MM-DD).
    /// Returns the sum of insertions + deletions. Returns 0 if the file has no changes.
    pub fn file_changes_since(repo_dir: &Path, file_path: &str, since_date: &str) -> Result<u64> {
        let output = Command::new("git")
            .args([
                "log",
                &format!("--since={}", since_date),
                "--numstat",
                "--pretty=",
                "--",
                file_path,
            ])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| PopsicleError::Storage(format!("git log --numstat failed: {}", e)))?;

        let text = String::from_utf8_lossy(&output.stdout);
        let mut total: u64 = 0;
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let added: u64 = parts[0].parse().unwrap_or(0);
                let deleted: u64 = parts[1].parse().unwrap_or(0);
                total += added + deleted;
            }
        }

        Ok(total)
    }

    /// Install a post-commit git hook that calls `popsicle git on-commit`.
    pub fn install_hook(repo_dir: &Path) -> Result<()> {
        let hooks_dir = repo_dir.join(".git").join("hooks");
        std::fs::create_dir_all(&hooks_dir)?;

        let hook_path = hooks_dir.join("post-commit");
        let hook_content = r#"#!/bin/sh
# Popsicle post-commit hook
# Automatically tracks commits in the active pipeline run
popsicle git on-commit 2>/dev/null || true
"#;

        std::fs::write(&hook_path, hook_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))?;
        }

        Ok(())
    }
}
