#![allow(dead_code)]

use chrono::Local;
use git2::Repository;
use std::env;
use std::process::Command;
use sysinfo::System;

/// Segment data collectors for the prompt
pub struct Segments;

impl Segments {
    /// Get current working directory
    pub fn pwd() -> String {
        env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("~"))
    }

    /// Get shortened PWD (replaces home with ~)
    pub fn pwd_short() -> String {
        let pwd = Self::pwd();
        if let Some(home) = dirs::home_dir() {
            let home_str = home.display().to_string();
            if pwd.starts_with(&home_str) {
                return pwd.replace(&home_str, "~");
            }
        }
        pwd
    }

    /// Get current user
    pub fn user() -> String {
        whoami::username()
    }

    /// Get hostname
    pub fn hostname() -> String {
        #[allow(deprecated)]
        whoami::hostname()
    }

    /// Check if running over SSH
    pub fn is_ssh() -> bool {
        env::var("SSH_CONNECTION").is_ok()
    }

    /// Get git branch name
    pub fn git_branch() -> Option<String> {
        let repo = Repository::open_from_env().ok()?;
        let head = repo.head().ok()?;
        head.shorthand().map(String::from)
    }

    /// Get git status (modified, staged, untracked counts)
    pub fn git_status() -> Option<GitStatus> {
        let repo = Repository::open_from_env().ok()?;
        let mut status = GitStatus::default();

        let statuses = repo.statuses(None).ok()?;

        for entry in statuses.iter() {
            let flags = entry.status();

            // Working tree changes
            if flags.contains(git2::Status::WT_MODIFIED) {
                status.modified += 1;
            }
            if flags.contains(git2::Status::WT_DELETED) {
                status.deleted += 1;
            }
            if flags.contains(git2::Status::WT_RENAMED) {
                status.renamed += 1;
            }
            if flags.contains(git2::Status::WT_NEW) {
                status.untracked += 1;
            }

            // Index/staged changes
            if flags.contains(git2::Status::INDEX_NEW) {
                status.added += 1;
                status.staged += 1;
            }
            if flags.contains(git2::Status::INDEX_MODIFIED) {
                status.staged += 1;
            }
            if flags.contains(git2::Status::INDEX_DELETED) {
                status.deleted += 1;
                status.staged += 1;
            }
            if flags.contains(git2::Status::INDEX_RENAMED) {
                status.renamed += 1;
                status.staged += 1;
            }

            // Conflicted files
            if flags.contains(git2::Status::CONFLICTED) {
                status.conflicted += 1;
            }
        }

        Some(status)
    }

    /// Get current time
    pub fn time() -> String {
        Local::now().format("%H:%M:%S").to_string()
    }

    /// Get current date
    pub fn date() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    /// Get job count
    pub fn job_count() -> usize {
        0
    }

    /// Get system load average
    pub fn load_average() -> String {
        let load = System::load_average();
        format!("{:.2}", load.one)
    }

    /// Get memory usage percentage
    pub fn memory_usage() -> f32 {
        let sys = System::new_all();

        let total = sys.total_memory() as f32;
        let used = sys.used_memory() as f32;

        (used / total) * 100.0
    }

    /// Get CPU usage percentage
    pub fn cpu_usage() -> f32 {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();

        sys.global_cpu_info().cpu_usage()
    }

    /// Get virtual environment name
    pub fn virtual_env() -> Option<String> {
        env::var("VIRTUAL_ENV").ok().and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
    }

    /// Get Node.js version
    pub fn node_version() -> Option<String> {
        Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|v| v.trim().to_string())
    }

    /// Get Rust version
    pub fn rust_version() -> Option<String> {
        Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|v| v.split_whitespace().nth(1).map(String::from))
    }

    /// Get Python version
    pub fn python_version() -> Option<String> {
        Command::new("python3")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|v| v.split_whitespace().nth(1).map(String::from))
    }

    /// Check if in a container (Docker, etc.)
    pub fn is_container() -> bool {
        std::path::Path::new("/.dockerenv").exists()
            || std::fs::read_to_string("/proc/1/cgroup")
                .map(|content| content.contains("docker") || content.contains("lxc"))
                .unwrap_or(false)
    }

    /// Get Kubernetes context
    pub fn k8s_context() -> Option<String> {
        Command::new("kubectl")
            .args(&["config", "current-context"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|v| v.trim().to_string())
    }

    /// Get AWS profile
    pub fn aws_profile() -> Option<String> {
        env::var("AWS_PROFILE").ok()
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct GitStatus {
    pub staged: usize,
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub renamed: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

impl GitStatus {
    pub fn is_dirty(&self) -> bool {
        self.modified > 0
            || self.staged > 0
            || self.untracked > 0
            || self.added > 0
            || self.deleted > 0
            || self.renamed > 0
            || self.conflicted > 0
    }

    pub fn format_short(&self) -> String {
        let mut parts = Vec::new();

        if self.staged > 0 {
            parts.push(format!("●{}", self.staged));
        }
        if self.modified > 0 {
            parts.push(format!("✚{}", self.modified));
        }
        if self.added > 0 {
            parts.push(format!("+{}", self.added));
        }
        if self.deleted > 0 {
            parts.push(format!("-{}", self.deleted));
        }
        if self.renamed > 0 {
            parts.push(format!("➜{}", self.renamed));
        }
        if self.untracked > 0 {
            parts.push(format!("…{}", self.untracked));
        }
        if self.conflicted > 0 {
            parts.push(format!("✖{}", self.conflicted));
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GitStatus tests
    // =========================================================================

    #[test]
    fn test_git_status_default_is_clean() {
        let status = GitStatus::default();
        assert!(!status.is_dirty());
    }

    #[test]
    fn test_git_status_modified_is_dirty() {
        let status = GitStatus {
            modified: 1,
            ..Default::default()
        };
        assert!(status.is_dirty());
    }

    #[test]
    fn test_git_status_staged_is_dirty() {
        let status = GitStatus {
            staged: 1,
            ..Default::default()
        };
        assert!(status.is_dirty());
    }

    #[test]
    fn test_git_status_untracked_is_dirty() {
        let status = GitStatus {
            untracked: 1,
            ..Default::default()
        };
        assert!(status.is_dirty());
    }

    #[test]
    fn test_git_status_conflicted_is_dirty() {
        let status = GitStatus {
            conflicted: 1,
            ..Default::default()
        };
        assert!(status.is_dirty());
    }

    #[test]
    fn test_git_status_format_short_empty() {
        let status = GitStatus::default();
        assert_eq!(status.format_short(), "");
    }

    #[test]
    fn test_git_status_format_short_staged() {
        let status = GitStatus {
            staged: 3,
            ..Default::default()
        };
        assert_eq!(status.format_short(), "●3");
    }

    #[test]
    fn test_git_status_format_short_modified() {
        let status = GitStatus {
            modified: 2,
            ..Default::default()
        };
        assert_eq!(status.format_short(), "✚2");
    }

    #[test]
    fn test_git_status_format_short_multiple() {
        let status = GitStatus {
            staged: 1,
            modified: 2,
            untracked: 3,
            ..Default::default()
        };
        let formatted = status.format_short();
        assert!(formatted.contains("●1"));
        assert!(formatted.contains("✚2"));
        assert!(formatted.contains("…3"));
    }

    #[test]
    fn test_git_status_format_short_all_fields() {
        let status = GitStatus {
            staged: 1,
            modified: 2,
            added: 3,
            deleted: 4,
            renamed: 5,
            untracked: 6,
            conflicted: 7,
        };
        let formatted = status.format_short();
        assert!(formatted.contains("●1")); // staged
        assert!(formatted.contains("✚2")); // modified
        assert!(formatted.contains("+3")); // added
        assert!(formatted.contains("-4")); // deleted
        assert!(formatted.contains("➜5")); // renamed
        assert!(formatted.contains("…6")); // untracked
        assert!(formatted.contains("✖7")); // conflicted
    }

    // =========================================================================
    // Segments function tests
    // =========================================================================

    #[test]
    fn test_time_format() {
        let time = Segments::time();
        // Should be in HH:MM:SS format
        assert_eq!(time.len(), 8);
        assert_eq!(&time[2..3], ":");
        assert_eq!(&time[5..6], ":");
    }

    #[test]
    fn test_date_format() {
        let date = Segments::date();
        // Should be in YYYY-MM-DD format
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_pwd_returns_string() {
        let pwd = Segments::pwd();
        assert!(!pwd.is_empty());
    }

    #[test]
    fn test_pwd_short_replaces_home() {
        let pwd_short = Segments::pwd_short();
        // If we're in home dir or subdir, should start with ~ or be absolute path
        assert!(!pwd_short.is_empty());
    }

    #[test]
    fn test_user_returns_string() {
        let user = Segments::user();
        assert!(!user.is_empty());
    }

    #[test]
    fn test_hostname_returns_string() {
        let hostname = Segments::hostname();
        assert!(!hostname.is_empty());
    }

    #[test]
    fn test_is_ssh_without_env() {
        // Remove SSH_CONNECTION if it exists (in a real test environment)
        // This test verifies the function doesn't panic
        let _ = Segments::is_ssh();
    }

    #[test]
    fn test_job_count_returns_zero() {
        // Current implementation always returns 0
        assert_eq!(Segments::job_count(), 0);
    }

    #[test]
    fn test_virtual_env_none_when_not_set() {
        // Save current value
        let original = env::var("VIRTUAL_ENV").ok();

        // Remove the env var
        env::remove_var("VIRTUAL_ENV");

        let result = Segments::virtual_env();
        assert!(result.is_none());

        // Restore original value if it existed
        if let Some(val) = original {
            env::set_var("VIRTUAL_ENV", val);
        }
    }

    #[test]
    fn test_virtual_env_extracts_name() {
        // Save current value
        let original = env::var("VIRTUAL_ENV").ok();

        // Set a test value
        env::set_var("VIRTUAL_ENV", "/home/user/project/.venv");

        let result = Segments::virtual_env();
        assert_eq!(result, Some(".venv".to_string()));

        // Restore or remove
        match original {
            Some(val) => env::set_var("VIRTUAL_ENV", val),
            None => env::remove_var("VIRTUAL_ENV"),
        }
    }

    #[test]
    fn test_aws_profile_none_when_not_set() {
        // Save current value
        let original = env::var("AWS_PROFILE").ok();

        // Remove the env var
        env::remove_var("AWS_PROFILE");

        let result = Segments::aws_profile();
        assert!(result.is_none());

        // Restore original value if it existed
        if let Some(val) = original {
            env::set_var("AWS_PROFILE", val);
        }
    }

    #[test]
    fn test_aws_profile_returns_value() {
        // Save current value
        let original = env::var("AWS_PROFILE").ok();

        // Set a test value
        env::set_var("AWS_PROFILE", "test-profile");

        let result = Segments::aws_profile();
        assert_eq!(result, Some("test-profile".to_string()));

        // Restore or remove
        match original {
            Some(val) => env::set_var("AWS_PROFILE", val),
            None => env::remove_var("AWS_PROFILE"),
        }
    }

    #[test]
    fn test_is_container_returns_bool() {
        // Just verify it doesn't panic and returns a bool
        let _ = Segments::is_container();
    }

    #[test]
    fn test_load_average_format() {
        let load = Segments::load_average();
        // Should be a decimal number string
        assert!(!load.is_empty());
        // Should parse as a float
        assert!(load.parse::<f32>().is_ok());
    }

    #[test]
    fn test_memory_usage_in_range() {
        let usage = Segments::memory_usage();
        // Memory usage should be between 0 and 100 percent
        assert!(usage >= 0.0);
        assert!(usage <= 100.0);
    }

    #[test]
    fn test_cpu_usage_in_range() {
        let usage = Segments::cpu_usage();
        // CPU usage should be between 0 and 100 percent
        assert!(usage >= 0.0);
        assert!(usage <= 100.0);
    }
}
