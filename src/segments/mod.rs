use chrono::Local;
use git2::Repository;
use std::env;
use std::process::Command;
use sysinfo::System;

/// Segment data collectors for the prompt
#[allow(dead_code)]
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
