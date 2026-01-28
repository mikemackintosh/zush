use git2::{Repository, StatusOptions, StatusShow};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Cache entry for git status
struct CacheEntry {
    path: PathBuf,
    status: GitStatus,
    timestamp: Instant,
}

/// Global cache for git status (avoids recalculating on rapid prompt renders)
static GIT_CACHE: Mutex<Option<CacheEntry>> = Mutex::new(None);

/// Cache TTL - 1 second is short enough to be responsive but helps with rapid Enter presses
const CACHE_TTL: Duration = Duration::from_secs(1);

#[derive(Debug, Default, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub renamed: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

/// Get git status for the current directory (with caching)
/// This is much faster than calling `git` commands because:
/// 1. No process spawning
/// 2. Direct file reading from .git directory
/// 3. No shell overhead
/// 4. Results cached for 1 second to avoid redundant work
pub fn get_git_status(path: &Path) -> Option<GitStatus> {
    // Check cache first
    if let Ok(cache) = GIT_CACHE.lock() {
        if let Some(ref entry) = *cache {
            if entry.path == path && entry.timestamp.elapsed() < CACHE_TTL {
                return Some(entry.status.clone());
            }
        }
    }

    // Cache miss - compute status
    let status = compute_git_status(path)?;

    // Update cache
    if let Ok(mut cache) = GIT_CACHE.lock() {
        *cache = Some(CacheEntry {
            path: path.to_path_buf(),
            status: status.clone(),
            timestamp: Instant::now(),
        });
    }

    Some(status)
}

/// Actually compute git status (internal, not cached)
fn compute_git_status(path: &Path) -> Option<GitStatus> {
    // Try to open repository - this is fast, just checks for .git
    let repo = Repository::discover(path).ok()?;

    let mut status = GitStatus::default();

    // Get current branch name
    if let Ok(head) = repo.head() {
        if let Some(branch_name) = head.shorthand() {
            status.branch = branch_name.to_string();
        }
    }

    // Get status - this reads .git/index directly
    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(false);

    if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
        for entry in statuses.iter() {
            let status_flags = entry.status();

            // Index (staged) changes
            if status_flags.is_index_new() {
                status.added += 1;
                status.staged += 1;
            }
            if status_flags.is_index_modified() {
                status.staged += 1;
            }
            if status_flags.is_index_deleted() {
                status.deleted += 1;
                status.staged += 1;
            }
            if status_flags.is_index_renamed() {
                status.renamed += 1;
                status.staged += 1;
            }

            // Working tree changes
            if status_flags.is_wt_modified() {
                status.modified += 1;
            }
            if status_flags.is_wt_deleted() {
                status.deleted += 1;
            }

            // Untracked
            if status_flags.is_wt_new() {
                status.untracked += 1;
            }

            // Conflicted
            if status_flags.is_conflicted() {
                status.conflicted += 1;
            }
        }
    }

    Some(status)
}

/// Convert GitStatus to JSON for template context
pub fn git_status_to_json(status: &GitStatus) -> Value {
    json!({
        "git_branch": status.branch,
        "git_staged": status.staged,
        "git_modified": status.modified,
        "git_added": status.added,
        "git_deleted": status.deleted,
        "git_renamed": status.renamed,
        "git_untracked": status.untracked,
        "git_conflicted": status.conflicted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_status() {
        // Test in current directory (should be a git repo during development)
        let cwd = env::current_dir().unwrap();
        if let Some(status) = get_git_status(&cwd) {
            println!("Branch: {}", status.branch);
            println!("Staged: {}", status.staged);
            println!("Modified: {}", status.modified);
            assert!(!status.branch.is_empty());
        }
    }
}
