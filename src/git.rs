use git2::{Repository, StatusOptions, StatusShow};
use serde_json::{json, Value};
use std::fs;
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

/// Cache TTL - 5 seconds balances responsiveness with performance in large repos
/// Most commands complete within 5s, so this avoids redundant git status calls
const CACHE_TTL: Duration = Duration::from_secs(5);

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

/// Fast path: Read branch name directly from .git/HEAD (no libgit2)
/// This is much faster than using Repository::head(), especially in huge repos
fn read_branch_fast(git_dir: &Path) -> Option<String> {
    let head_path = git_dir.join("HEAD");
    let contents = fs::read_to_string(head_path).ok()?;

    // Parse HEAD file
    // Format is either:
    // - "ref: refs/heads/branch-name" (normal branch)
    // - "commit-hash" (detached HEAD)
    let trimmed = contents.trim();
    if let Some(ref_path) = trimmed.strip_prefix("ref: ") {
        // Extract branch name from "refs/heads/branch-name"
        ref_path
            .strip_prefix("refs/heads/")
            .map(|s| s.to_string())
            .or_else(|| Some(ref_path.to_string()))
    } else {
        // Detached HEAD - return short hash
        Some(trimmed.chars().take(7).collect())
    }
}

/// Find .git directory from current path
fn find_git_dir(mut path: &Path) -> Option<PathBuf> {
    loop {
        let git_dir = path.join(".git");

        // Check if it's a directory
        if git_dir.is_dir() {
            return Some(git_dir);
        }

        // Check if it's a file (git worktree)
        if git_dir.is_file() {
            // Read the file to get actual git dir path
            if let Ok(contents) = fs::read_to_string(&git_dir) {
                if let Some(gitdir) = contents.trim().strip_prefix("gitdir: ") {
                    return Some(PathBuf::from(gitdir));
                }
            }
        }

        // Move up to parent directory
        path = path.parent()?;
    }
}

/// Actually compute git status (internal, not cached)
fn compute_git_status(path: &Path) -> Option<GitStatus> {
    let mut status = GitStatus::default();

    // Fast path: Try to read branch name directly without libgit2
    // This is much faster, especially for huge repositories
    let git_dir = find_git_dir(path);
    if let Some(ref dir) = git_dir {
        if let Some(branch) = read_branch_fast(dir) {
            status.branch = branch;
        }
    }

    // If branch name failed, fall back to libgit2
    if status.branch.is_empty() {
        if let Ok(repo) = Repository::discover(path) {
            if let Ok(head) = repo.head() {
                if let Some(branch_name) = head.shorthand() {
                    status.branch = branch_name.to_string();
                }
            }
        }
    }

    // If still no branch name, we're not in a git repo
    if status.branch.is_empty() {
        return None;
    }

    // Check if minimal mode is enabled (for huge repos like 1GB+)
    // Set ZUSH_GIT_MINIMAL=1 to only show branch name, skip all status checks
    // This is much faster for very large repositories
    let minimal_mode = std::env::var("ZUSH_GIT_MINIMAL")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    if minimal_mode {
        // Skip status checking entirely - just return branch name
        return Some(status);
    }

    // Need to open repository for status checking
    let repo = Repository::discover(path).ok()?;

    // Get status - this reads .git/index directly
    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir);

    // Check if untracked file scanning should be disabled (for large repos)
    // Set ZUSH_GIT_DISABLE_UNTRACKED=1 to skip untracked file counting
    let include_untracked = std::env::var("ZUSH_GIT_DISABLE_UNTRACKED")
        .map(|v| v != "1" && v.to_lowercase() != "true")
        .unwrap_or(true);

    opts.include_untracked(include_untracked);
    opts.recurse_untracked_dirs(false); // Don't recurse to improve performance

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
